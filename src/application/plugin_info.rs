//! プラグイン詳細情報取得
//!
//! 特定のプラグインの詳細情報を取得するユースケースを提供する。

use super::plugin_catalog::list_all_placed;
pub(super) use super::plugin_info_types::{AuthorInfo, ComponentInfo, PluginDetail, PluginSource};
use crate::error::{PlmError, Result};
use crate::plugin::{has_manifest, meta, PluginCache, PluginManifest};
use crate::scan::scan_components;
use std::path::{Path, PathBuf};

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
pub fn get_plugin_info(name: &str) -> Result<PluginDetail> {
    let (marketplace_filter, plugin_name) = parse_plugin_name(name)?;
    let candidates = find_plugin_candidates(&plugin_name)?;
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
fn find_plugin_candidates(name: &str) -> Result<Vec<PluginCandidate>> {
    let cache = PluginCache::new()?;
    let plugin_list = cache.list()?;

    let mut candidates = Vec::new();

    for (marketplace, dir_name) in plugin_list {
        // 隠しディレクトリは除外
        if dir_name.starts_with('.') {
            continue;
        }

        let marketplace_name = marketplace.clone().unwrap_or_else(|| "github".to_string());
        let plugin_path = cache.plugin_path(marketplace.as_deref(), &dir_name);

        // plugin.json が存在するもののみ
        if !has_manifest(&plugin_path) {
            continue;
        }

        let manifest = match cache.load_manifest(marketplace.as_deref(), &dir_name) {
            Ok(m) => m,
            Err(_) => continue,
        };

        // manifest.name が一致するもののみ
        if manifest.name == name {
            candidates.push(PluginCandidate {
                marketplace: marketplace_name,
                dir_name,
                cache_path: plugin_path,
                manifest,
            });
        }
    }

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
fn determine_source_from_path(
    _cache_path: &Path,
    marketplace: &str,
    dir_name: &str,
) -> PluginSource {
    if marketplace == "github" {
        // owner--repo → owner/repo
        let repository = restore_github_repo(dir_name);
        PluginSource::GitHub { repository }
    } else {
        PluginSource::Marketplace {
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

/// PluginDetail を構築
fn build_plugin_detail(candidate: PluginCandidate) -> Result<PluginDetail> {
    let manifest = &candidate.manifest;

    // 作者情報の変換
    let author = manifest.author.as_ref().and_then(|a| {
        // name が空または未設定の場合は None
        if a.name.is_empty() {
            None
        } else {
            Some(AuthorInfo {
                name: a.name.clone(),
                email: a.email.clone(),
                url: a.url.clone(),
            })
        }
    });

    // ソース判定
    let source = determine_source_from_path(
        &candidate.cache_path,
        &candidate.marketplace,
        &candidate.dir_name,
    );

    // コンポーネント走査
    let scan = scan_components(&candidate.cache_path, manifest);
    let components = ComponentInfo {
        skills: scan.skills,
        agents: scan.agents,
        commands: scan.commands,
        instructions: scan.instructions,
        hooks: scan.hooks,
    };

    // デプロイ状態判定
    let enabled = check_deployed_status(
        &candidate.cache_path,
        &candidate.marketplace,
        &manifest.name,
    );

    // キャッシュパス（絶対パス）
    let cache_path = candidate.cache_path.to_string_lossy().to_string();

    Ok(PluginDetail {
        name: manifest.name.clone(),
        version: manifest.version.clone(),
        description: manifest.description.clone(),
        author,
        installed_at: meta::resolve_installed_at(&candidate.cache_path, Some(manifest)),
        source,
        components,
        enabled,
        cache_path,
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
