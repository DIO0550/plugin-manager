//! Marketplaces タブのアクション実行
//!
//! マーケットプレイスの追加・削除・更新操作を実行する。

use super::model::{BrowsePlugin, InstallSummary, PluginInstallResult};
use crate::application::PluginSummary;
use crate::component::Scope;
use crate::install::{self, PlaceRequest};
use crate::marketplace::{
    to_display_source, to_internal_source, MarketplaceCache, MarketplaceConfig, MarketplaceFetcher,
    MarketplaceRegistration, MarketplaceRegistry,
};
use crate::repo;
use crate::target::parse_target;
use crate::tui::manager::core::MarketplaceItem;
use crate::tui::output_suppress::OutputSuppressGuard;
use std::path::Path;

/// マーケットプレイス追加の結果
pub struct AddResult {
    pub marketplace: MarketplaceItem,
}

/// マーケットプレイスを追加
pub fn add_marketplace(
    source: &str,
    name: &str,
    source_path: Option<&str>,
) -> Result<AddResult, String> {
    let handle = tokio::runtime::Handle::try_current()
        .map_err(|_| "No Tokio runtime available".to_string())?;

    let repo = repo::from_url(source).map_err(|e| e.to_string())?;
    let internal_source = to_internal_source(&repo.full_name());

    let mut config = MarketplaceConfig::load()?;

    if config.exists(name) {
        return Err(format!("Marketplace '{}' already exists", name));
    }

    let entry = MarketplaceRegistration {
        name: name.to_string(),
        source: internal_source.clone(),
        source_path: source_path.map(|s| s.to_string()),
    };

    config.add(entry)?;

    // フェッチしてキャッシュ
    let fetcher = MarketplaceFetcher::new();
    let cache = tokio::task::block_in_place(|| {
        handle.block_on(fetcher.fetch_as_cache(&repo, name, source_path))
    })
    .map_err(|e| e.to_string())?;

    // config.save() を先に実行し、失敗時に孤立キャッシュが残らないようにする
    config.save()?;

    let registry = MarketplaceRegistry::new().map_err(|e| e.to_string())?;
    registry.store(&cache).map_err(|e| e.to_string())?;

    Ok(AddResult {
        marketplace: MarketplaceItem {
            name: name.to_string(),
            source: to_display_source(&internal_source),
            source_path: source_path.map(|s| s.to_string()),
            plugin_count: Some(cache.plugins.len()),
            last_updated: Some(cache.fetched_at.format("%Y-%m-%d %H:%M").to_string()),
        },
    })
}

/// マーケットプレイスを削除
pub fn remove_marketplace(name: &str) -> Result<(), String> {
    let mut config = MarketplaceConfig::load()?;
    config.remove(name)?;
    config.save()?;

    // キャッシュ削除（失敗しても続行）
    if let Ok(registry) = MarketplaceRegistry::new() {
        let _ = registry.remove(name);
    }

    Ok(())
}

/// マーケットプレイスを更新
pub fn update_marketplace(name: &str) -> Result<MarketplaceItem, String> {
    let config = MarketplaceConfig::load()?;
    let entry = config
        .get(name)
        .ok_or_else(|| format!("Marketplace '{}' not found", name))?
        .clone();

    update_marketplace_registration(&entry)
}

/// マーケットプレイス登録情報を更新（config再読み込み不要）
fn update_marketplace_registration(
    entry: &MarketplaceRegistration,
) -> Result<MarketplaceItem, String> {
    let handle = tokio::runtime::Handle::try_current()
        .map_err(|_| "No Tokio runtime available".to_string())?;

    let display_source = to_display_source(&entry.source);
    let repo = repo::from_url(&display_source).map_err(|e| e.to_string())?;
    let fetcher = MarketplaceFetcher::new();
    let cache = tokio::task::block_in_place(|| {
        handle.block_on(fetcher.fetch_as_cache(&repo, &entry.name, entry.source_path.as_deref()))
    })
    .map_err(|e| e.to_string())?;

    let registry = MarketplaceRegistry::new().map_err(|e| e.to_string())?;
    registry.store(&cache).map_err(|e| e.to_string())?;

    Ok(MarketplaceItem {
        name: entry.name.clone(),
        source: display_source,
        source_path: entry.source_path.clone(),
        plugin_count: Some(cache.plugins.len()),
        last_updated: Some(cache.fetched_at.format("%Y-%m-%d %H:%M").to_string()),
    })
}

/// 全マーケットプレイスを更新
pub fn update_all_marketplaces() -> Vec<(String, Result<MarketplaceItem, String>)> {
    let config = match MarketplaceConfig::load() {
        Ok(c) => c,
        Err(e) => return vec![("(config)".to_string(), Err(e))],
    };

    config
        .list()
        .iter()
        .map(|entry| {
            let result = update_marketplace_registration(entry);
            (entry.name.clone(), result)
        })
        .collect()
}

/// マーケットプレイスのプラグイン一覧を取得
pub fn get_marketplace_plugins(name: &str) -> Vec<(String, Option<String>)> {
    let registry = match MarketplaceRegistry::new() {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    match registry.get(name) {
        Ok(Some(cache)) => cache
            .plugins
            .iter()
            .map(|p| (p.name.clone(), p.description.clone()))
            .collect(),
        _ => Vec::new(),
    }
}

/// マーケットプレイスのブラウズ用プラグイン一覧を取得
pub(super) fn get_browse_plugins(
    marketplace_name: &str,
    installed_plugins: &[PluginSummary],
) -> Vec<BrowsePlugin> {
    let registry = match MarketplaceRegistry::new() {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    get_browse_plugins_with_registry(&registry, marketplace_name, installed_plugins)
}

/// Registry を引数で受ける内部ヘルパー（I/O テスト用）
fn get_browse_plugins_with_registry(
    registry: &MarketplaceRegistry,
    marketplace_name: &str,
    installed_plugins: &[PluginSummary],
) -> Vec<BrowsePlugin> {
    match registry.get(marketplace_name) {
        Ok(Some(cache)) => build_browse_plugins(&cache, installed_plugins),
        _ => Vec::new(),
    }
}

/// 純粋変換: MarketplaceCache -> Vec<BrowsePlugin>
fn build_browse_plugins(
    cache: &MarketplaceCache,
    installed_plugins: &[PluginSummary],
) -> Vec<BrowsePlugin> {
    cache
        .plugins
        .iter()
        .map(|p| BrowsePlugin {
            name: p.name.clone(),
            description: p.description.clone(),
            version: p.version.clone(),
            source: p.source.clone(),
            installed: installed_plugins.iter().any(|ip| ip.name == p.name),
        })
        .collect()
}

/// 前提条件エラー時に全プラグインを失敗として記録
fn make_all_failed_summary(plugin_names: &[String], error: &str) -> InstallSummary {
    let results: Vec<PluginInstallResult> = plugin_names
        .iter()
        .map(|name| PluginInstallResult {
            plugin_name: name.clone(),
            success: false,
            error: Some(error.to_string()),
        })
        .collect();
    build_install_summary(results)
}

/// 個別プラグインの download -> scan -> place パイプライン
fn install_single_plugin(
    handle: &tokio::runtime::Handle,
    marketplace_name: &str,
    plugin_name: &str,
    targets: &[Box<dyn crate::target::Target>],
    scope: Scope,
    project_root: &Path,
) -> PluginInstallResult {
    // Download (async -> sync bridge)
    let downloaded = match tokio::task::block_in_place(|| {
        handle.block_on(install::download_marketplace_plugin(
            plugin_name,
            marketplace_name,
            false,
        ))
    }) {
        Ok(d) => d,
        Err(e) => {
            return PluginInstallResult {
                plugin_name: plugin_name.to_string(),
                success: false,
                error: Some(e),
            }
        }
    };

    // Scan
    let scanned = match install::scan_plugin(&downloaded, None) {
        Ok(s) => s,
        Err(e) => {
            return PluginInstallResult {
                plugin_name: plugin_name.to_string(),
                success: false,
                error: Some(e),
            }
        }
    };

    // Place
    let place_result = install::place_plugin(&PlaceRequest {
        scanned: &scanned,
        targets,
        scope,
        project_root,
    });

    if !place_result.failures.is_empty() {
        let errors: Vec<String> = place_result
            .failures
            .iter()
            .map(|f| format!("{}/{}: {}", f.target, f.component_name, f.error))
            .collect();
        PluginInstallResult {
            plugin_name: plugin_name.to_string(),
            success: false,
            error: Some(errors.join("; ")),
        }
    } else if place_result.successes.is_empty() {
        PluginInstallResult {
            plugin_name: plugin_name.to_string(),
            success: false,
            error: Some("No components were placed".to_string()),
        }
    } else {
        PluginInstallResult {
            plugin_name: plugin_name.to_string(),
            success: true,
            error: None,
        }
    }
}

/// マーケットプレイスから複数プラグインを一括インストール
pub fn install_plugins(
    marketplace_name: &str,
    plugin_names: &[String],
    target_names: &[String],
    scope: Scope,
) -> InstallSummary {
    // stdout/stderr 抑制（TUI代替スクリーンの保護）
    let _guard = OutputSuppressGuard::new();

    // Tokio runtime handle 取得
    let handle = match tokio::runtime::Handle::try_current() {
        Ok(h) => h,
        Err(_) => return make_all_failed_summary(plugin_names, "No Tokio runtime available"),
    };

    // プラグイン名の空チェック
    if plugin_names.is_empty() {
        return build_install_summary(Vec::new());
    }

    // ターゲット名の空チェック
    if target_names.is_empty() {
        return make_all_failed_summary(plugin_names, "No targets specified");
    }

    // ターゲット解決（全ターゲットまとめて先に解決）
    let targets: Vec<Box<dyn crate::target::Target>> = match target_names
        .iter()
        .map(|name| parse_target(name).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(t) => t,
        Err(e) => return make_all_failed_summary(plugin_names, &e),
    };

    // project_root 取得（ループ外で1回）
    let project_root = match std::env::current_dir() {
        Ok(p) => p,
        Err(e) => return make_all_failed_summary(plugin_names, &e.to_string()),
    };

    // 各プラグインに対して download -> scan -> place
    let results: Vec<PluginInstallResult> = plugin_names
        .iter()
        .map(|plugin_name| {
            install_single_plugin(
                &handle,
                marketplace_name,
                plugin_name,
                &targets,
                scope,
                &project_root,
            )
        })
        .collect();

    build_install_summary(results)
}

/// 純粋変換: Vec<PluginInstallResult> -> InstallSummary
fn build_install_summary(results: Vec<PluginInstallResult>) -> InstallSummary {
    let total = results.len();
    let succeeded = results.iter().filter(|r| r.success).count();
    let failed = total - succeeded;
    InstallSummary {
        results,
        total,
        succeeded,
        failed,
    }
}

#[cfg(test)]
#[path = "actions_test.rs"]
mod actions_test;
