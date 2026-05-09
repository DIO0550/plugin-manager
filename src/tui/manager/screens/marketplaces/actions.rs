//! Marketplaces タブのアクション実行
//!
//! マーケットプレイスの追加・削除・更新操作を実行する。

use super::model::{BrowsePlugin, InstallSummary, PluginInstallResult};
use crate::application::InstalledPlugin;
use crate::component::Scope;
use crate::host::HostClientFactory;
use crate::install::{self, PlaceRequest};
use crate::marketplace::{
    download_marketplace_plugin_with_cache, to_display_source, to_internal_source,
    MarketplaceCache, MarketplaceConfig, MarketplaceRegistration, MarketplaceRegistry,
};
use crate::plugin::{PackageCache, PackageCacheAccess};
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
///
/// # Arguments
///
/// * `source` - Marketplace source URL or `owner/repo` spec.
/// * `name` - Local name used to reference the marketplace.
/// * `source_path` - Optional subdirectory path inside the source repository.
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

    let factory = HostClientFactory::with_defaults();
    let client = factory.create(repo.host());
    let registry = MarketplaceRegistry::new().map_err(|e| e.to_string())?;
    let cache = tokio::task::block_in_place(|| {
        handle.block_on(registry.fetch_cache(&*client, name, &repo, source_path))
    })
    .map_err(|e| e.to_string())?;

    // config.save() を先に実行し、失敗時に孤立キャッシュが残らないようにする
    config.save()?;

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
///
/// # Arguments
///
/// * `name` - Local marketplace name to remove.
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
///
/// # Arguments
///
/// * `name` - Local marketplace name to refresh.
pub fn update_marketplace(name: &str) -> Result<MarketplaceItem, String> {
    let config = MarketplaceConfig::load()?;
    let entry = config
        .get(name)
        .ok_or_else(|| format!("Marketplace '{}' not found", name))?
        .clone();

    update_marketplace_registration(&entry)
}

/// マーケットプレイス登録情報を更新（config再読み込み不要）
///
/// # Arguments
///
/// * `entry` - Registration entry to re-fetch and store.
fn update_marketplace_registration(
    entry: &MarketplaceRegistration,
) -> Result<MarketplaceItem, String> {
    let handle = tokio::runtime::Handle::try_current()
        .map_err(|_| "No Tokio runtime available".to_string())?;

    let display_source = to_display_source(&entry.source);
    let repo = repo::from_url(&display_source).map_err(|e| e.to_string())?;
    let factory = HostClientFactory::with_defaults();
    let client = factory.create(repo.host());
    let registry = MarketplaceRegistry::new().map_err(|e| e.to_string())?;
    let cache = tokio::task::block_in_place(|| {
        handle.block_on(registry.fetch_cache(
            &*client,
            &entry.name,
            &repo,
            entry.source_path.as_deref(),
        ))
    })
    .map_err(|e| e.to_string())?;

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
///
/// # Arguments
///
/// * `name` - Local marketplace name to query.
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
///
/// # Arguments
///
/// * `marketplace_name` - Local marketplace name to browse.
/// * `installed_plugins` - Currently installed plugins used to flag `installed`.
pub(super) fn get_browse_plugins(
    marketplace_name: &str,
    installed_plugins: &[InstalledPlugin],
) -> Vec<BrowsePlugin> {
    let registry = match MarketplaceRegistry::new() {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    get_browse_plugins_with_registry(&registry, marketplace_name, installed_plugins)
}

/// Registry を引数で受ける内部ヘルパー（I/O テスト用）
///
/// # Arguments
///
/// * `registry` - Marketplace registry to query (injected for tests).
/// * `marketplace_name` - Local marketplace name to browse.
/// * `installed_plugins` - Currently installed plugins used to flag `installed`.
fn get_browse_plugins_with_registry(
    registry: &MarketplaceRegistry,
    marketplace_name: &str,
    installed_plugins: &[InstalledPlugin],
) -> Vec<BrowsePlugin> {
    match registry.get(marketplace_name) {
        Ok(Some(cache)) => build_browse_plugins(&cache, installed_plugins),
        _ => Vec::new(),
    }
}

/// 純粋変換: MarketplaceCache -> Vec<BrowsePlugin>
///
/// # Arguments
///
/// * `cache` - Marketplace cache snapshot to convert.
/// * `installed_plugins` - Currently installed plugins used to flag `installed`.
///
/// # Note
///
/// `installed` 判定は `InstalledPlugin::id()`（キャッシュディレクトリ名）と
/// `MarketplacePlugin.name`（marketplace.json 登録名）が一致することを前提とする。
/// GitHub ソースでは marketplace 経由のプラグインは登録名と同じ ID になる一方、
/// 直接 GitHub から入れたプラグインは `owner--repo` 形式の別 ID になる。
/// そのため marketplace の登録名が `owner--repo` と乖離するケースや、
/// 同名プラグインを直接 GitHub からインストールしたケースでは、
/// インストール済みでも `installed=false` となり得る。
fn build_browse_plugins(
    cache: &MarketplaceCache,
    installed_plugins: &[InstalledPlugin],
) -> Vec<BrowsePlugin> {
    cache
        .plugins
        .iter()
        .map(|p| BrowsePlugin {
            name: p.name.clone(),
            description: p.description.clone(),
            version: p.version.clone(),
            source: p.source.clone(),
            installed: installed_plugins.iter().any(|ip| ip.id() == p.name),
        })
        .collect()
}

/// 前提条件エラー時に全プラグインを失敗として記録
///
/// # Arguments
///
/// * `plugin_names` - Plugin names that are all marked as failed.
/// * `error` - Error message shared across every failure entry.
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

/// インストール処理のコンテキスト
struct InstallCtx<'a> {
    handle: &'a tokio::runtime::Handle,
    targets: &'a [Box<dyn crate::target::Target>],
    scope: Scope,
    project_root: &'a Path,
    cache: &'a dyn PackageCacheAccess,
}

/// 個別プラグインの download -> scan -> place パイプライン
///
/// # Arguments
///
/// * `ctx` - Shared install context (runtime handle, targets, scope, cache).
/// * `marketplace_name` - Marketplace the plugin is downloaded from.
/// * `plugin_name` - Plugin to install.
fn install_single_plugin(
    ctx: &InstallCtx<'_>,
    marketplace_name: &str,
    plugin_name: &str,
) -> PluginInstallResult {
    // Download (async -> sync bridge)
    let package = match tokio::task::block_in_place(|| {
        ctx.handle.block_on(download_marketplace_plugin_with_cache(
            plugin_name,
            marketplace_name,
            false,
            ctx.cache,
        ))
    }) {
        Ok(d) => d,
        Err(e) => {
            return PluginInstallResult {
                plugin_name: plugin_name.to_string(),
                success: false,
                error: Some(e.to_string()),
            }
        }
    };

    let scanned = match install::scan_plugin(&package, None) {
        Ok(s) => s,
        Err(e) => {
            return PluginInstallResult {
                plugin_name: plugin_name.to_string(),
                success: false,
                error: Some(e),
            }
        }
    };

    let place_result = install::place_plugin(&PlaceRequest {
        scanned: &scanned,
        targets: ctx.targets,
        scope: ctx.scope,
        project_root: ctx.project_root,
    });

    install::update_meta_after_place(scanned.plugin_root(), &place_result);

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
            error: Some(
                "No components were placed. The plugin may contain no components after scanning, \
                 or none of its components are supported by the selected targets."
                    .to_string(),
            ),
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
///
/// # Arguments
///
/// * `marketplace_name` - Marketplace plugins are downloaded from.
/// * `plugin_names` - Plugins to install in order.
/// * `target_names` - Target environment names to deploy into.
/// * `scope` - Personal/Project scope used by every target.
pub fn install_plugins(
    marketplace_name: &str,
    plugin_names: &[String],
    target_names: &[String],
    scope: Scope,
) -> InstallSummary {
    // プラグイン名の空チェック（Tokio不要で早期リターン）
    if plugin_names.is_empty() {
        return build_install_summary(Vec::new());
    }

    // ターゲット名の空チェック（Tokio不要で早期リターン）
    if target_names.is_empty() {
        return make_all_failed_summary(plugin_names, "No targets specified");
    }

    // ターゲット解決（Tokio不要で早期リターン）
    let targets: Vec<Box<dyn crate::target::Target>> = match target_names
        .iter()
        .map(|name| parse_target(name).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(t) => t,
        Err(e) => return make_all_failed_summary(plugin_names, &e),
    };

    // project_root 取得（他の箇所と同様に失敗時は "." にフォールバック）
    let project_root = std::env::current_dir().unwrap_or_else(|_| ".".into());

    // stdout/stderr 抑制（TUI代替スクリーンの保護）
    let _guard = OutputSuppressGuard::new();

    let handle = match tokio::runtime::Handle::try_current() {
        Ok(h) => h,
        Err(_) => return make_all_failed_summary(plugin_names, "No Tokio runtime available"),
    };

    // PackageCache を1回作成（各プラグインで共有）
    let cache = match PackageCache::new() {
        Ok(c) => c,
        Err(e) => {
            return make_all_failed_summary(plugin_names, &format!("Failed to access cache: {e}"))
        }
    };

    // 各プラグインに対して download -> scan -> place
    let install_ctx = InstallCtx {
        handle: &handle,
        targets: &targets,
        scope,
        project_root: &project_root,
        cache: &cache,
    };
    let results: Vec<PluginInstallResult> = plugin_names
        .iter()
        .map(|plugin_name| install_single_plugin(&install_ctx, marketplace_name, plugin_name))
        .collect();

    build_install_summary(results)
}

/// 純粋変換: Vec<PluginInstallResult> -> InstallSummary
///
/// # Arguments
///
/// * `results` - Individual install results to aggregate.
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
