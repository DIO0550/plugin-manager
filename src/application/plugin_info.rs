//! プラグイン詳細情報取得
//!
//! 特定のプラグインの詳細情報を取得するユースケースを提供する。

use super::plugin_catalog::{list_all_placed, list_installed, InstalledPlugin};
use crate::error::{PlmError, Result};
use crate::plugin::{meta, PackageCacheAccess, Plugin, PluginManifest};
use std::path::{Path, PathBuf};

/// プラグイン詳細情報（composition）
///
/// application 層が返す UseCase 出力。
/// 内部に [`InstalledPlugin`] を保持し、wire format 詳細は commands 層に委ねる。
pub struct PluginInfo {
    pub installed: InstalledPlugin,
    pub installed_at: Option<String>,
    pub source: Source,
}

impl PluginInfo {
    pub fn name(&self) -> &str {
        self.installed.name()
    }

    pub fn enabled(&self) -> bool {
        self.installed.enabled()
    }
}

/// プラグインソース情報
pub enum Source {
    GitHub { repository: String },
    Marketplace { name: String },
}

/// 内部用: プラグイン候補
#[derive(Debug)]
struct PluginCandidate {
    marketplace: String,
    dir_name: String,
    cache_path: PathBuf,
    manifest: PluginManifest,
}

/// プラグイン詳細情報を取得
///
/// # Arguments
/// * `name` - プラグイン名（"plugin" または "marketplace/plugin" 形式）
pub fn get_plugin_info(cache: &dyn PackageCacheAccess, name: &str) -> Result<PluginInfo> {
    let (marketplace_filter, plugin_name) = parse_plugin_name(name)?;
    let candidates = find_plugin_candidates(cache, &plugin_name)?;
    let resolved = resolve_single_plugin(candidates, marketplace_filter.as_deref(), &plugin_name)?;
    build_plugin_detail(resolved)
}

/// 入力名をパースしバリデーション
///
/// # Returns
/// * `Ok((None, "plugin"))` - プラグイン名のみ
/// * `Ok((Some("marketplace"), "plugin"))` - マーケットプレイス指定
/// * `Err` - 不正な形式
fn parse_plugin_name(name: &str) -> Result<(Option<String>, String)> {
    // 空文字チェック
    if name.is_empty() {
        return Err(PlmError::InvalidArgument("plugin name is empty".into()));
    }

    // 先頭がスラッシュの場合
    if name.starts_with('/') {
        return Err(PlmError::InvalidArgument(
            "plugin name cannot start with '/'".into(),
        ));
    }

    // 末尾がスラッシュの場合
    if name.ends_with('/') {
        return Err(PlmError::InvalidArgument(
            "plugin name cannot end with '/'".into(),
        ));
    }

    let slash_count = name.chars().filter(|c| *c == '/').count();

    match slash_count {
        0 => Ok((None, name.to_string())),
        1 => {
            let parts: Vec<&str> = name.split('/').collect();
            let marketplace = parts[0].to_string();
            let plugin = parts[1].to_string();

            // 空のパートがないかチェック
            if marketplace.is_empty() || plugin.is_empty() {
                return Err(PlmError::InvalidArgument(
                    "marketplace and plugin name cannot be empty".into(),
                ));
            }

            Ok((Some(marketplace), plugin))
        }
        _ => Err(PlmError::InvalidArgument(
            "invalid plugin name format: too many '/' separators".into(),
        )),
    }
}

/// キャッシュ全体をスキャンし、manifest.name が一致するプラグインを列挙
fn find_plugin_candidates(
    cache: &dyn PackageCacheAccess,
    name: &str,
) -> Result<Vec<PluginCandidate>> {
    let candidates = list_installed(cache)?
        .into_iter()
        .filter(|pkg| pkg.manifest().name == name)
        .map(|pkg| {
            let dir_name = pkg
                .cache_key()
                .map(str::to_string)
                .unwrap_or_else(|| pkg.name().to_string());
            PluginCandidate {
                marketplace: pkg
                    .marketplace()
                    .map(str::to_string)
                    .unwrap_or_else(|| "github".to_string()),
                dir_name,
                cache_path: pkg.path().to_path_buf(),
                manifest: pkg.manifest().clone(),
            }
        })
        .collect();

    Ok(candidates)
}

/// 候補から単一プラグインを解決
fn resolve_single_plugin(
    candidates: Vec<PluginCandidate>,
    marketplace_filter: Option<&str>,
    name: &str,
) -> Result<PluginCandidate> {
    // マーケットプレイスフィルタがある場合は絞り込み
    let filtered: Vec<_> = if let Some(mp) = marketplace_filter {
        candidates
            .into_iter()
            .filter(|c| c.marketplace == mp)
            .collect()
    } else {
        candidates
    };

    match filtered.len() {
        0 => Err(PlmError::PluginNotFound(name.to_string())),
        1 => Ok(filtered.into_iter().next().unwrap()),
        _ => {
            // 複数候補がある場合
            let candidate_names: Vec<String> = filtered
                .iter()
                .map(|c| format!("{}/{}", c.marketplace, c.manifest.name))
                .collect();
            Err(PlmError::AmbiguousPlugin {
                name: name.to_string(),
                candidates: candidate_names,
            })
        }
    }
}

/// キャッシュパスからソース情報を判定
fn determine_source_from_path(_cache_path: &Path, marketplace: &str, dir_name: &str) -> Source {
    if marketplace == "github" {
        // owner--repo → owner/repo
        let repository = restore_github_repo(dir_name);
        Source::GitHub { repository }
    } else {
        Source::Marketplace {
            name: marketplace.to_string(),
        }
    }
}

/// owner--repo → owner/repo に変換
fn restore_github_repo(dir_name: &str) -> String {
    // "--" を "/" に置換（最初の1つのみ）
    if let Some(pos) = dir_name.find("--") {
        let (owner, repo) = dir_name.split_at(pos);
        format!("{}/{}", owner, &repo[2..])
    } else {
        // "--" がない場合はそのまま返す
        dir_name.to_string()
    }
}

/// PluginInfo を構築
fn build_plugin_detail(candidate: PluginCandidate) -> Result<PluginInfo> {
    let PluginCandidate {
        marketplace,
        dir_name,
        cache_path,
        manifest,
    } = candidate;

    // ソース判定
    let source = determine_source_from_path(&cache_path, &marketplace, &dir_name);

    // インストール時刻
    let installed_at = meta::resolve_installed_at(&cache_path, Some(&manifest));

    // デプロイ状態判定（キャッシュディレクトリ名で判定）
    let enabled = check_deployed_status(&cache_path, &marketplace, &dir_name);

    // InstalledPlugin を組み立てる
    let install_id = Some(dir_name);
    let plugin = Plugin::new(manifest, cache_path);
    let installed = InstalledPlugin::new(plugin, install_id, Some(marketplace), enabled);

    Ok(PluginInfo {
        installed,
        installed_at,
        source,
    })
}

/// デプロイ状態を判定
///
/// `meta::is_enabled()` に委譲する。
fn check_deployed_status(cache_path: &Path, marketplace: &str, plugin_name: &str) -> bool {
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let deployed = list_all_placed(&project_root);
    meta::is_enabled(cache_path, marketplace, plugin_name, &deployed)
}

#[cfg(test)]
#[path = "plugin_info_test.rs"]
mod tests;
