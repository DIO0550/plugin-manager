//! プラグイン詳細情報取得
//!
//! 特定のプラグインの詳細情報を取得するユースケースを提供する。

use crate::component::{ComponentKind, Scope};
use crate::error::{PlmError, Result};
use crate::plugin::{has_manifest, meta, PluginCache, PluginManifest};
use crate::scan::scan_components;
use crate::target::{all_targets, PluginOrigin};
use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// プラグイン詳細情報（DTO）
#[derive(Debug, Clone, Serialize)]
pub struct PluginDetail {
    /// プラグイン名
    pub name: String,
    /// バージョン
    pub version: String,
    /// 説明文
    pub description: Option<String>,

    /// 作者情報（未設定の場合は None → JSON/YAMLでフィールド省略）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<AuthorInfo>,

    /// インストール日時（RFC3339形式）
    #[serde(rename = "installedAt")]
    pub installed_at: Option<String>,
    /// ソース情報
    pub source: PluginSource,

    /// コンポーネント一覧
    pub components: ComponentInfo,

    /// 有効状態
    pub enabled: bool,
    /// キャッシュパス（絶対パス）
    #[serde(rename = "cachePath")]
    pub cache_path: String,
}

/// 作者情報
#[derive(Debug, Clone, Serialize)]
pub struct AuthorInfo {
    /// 作者名
    pub name: String,
    /// メールアドレス
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// プラグインソース情報
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PluginSource {
    /// GitHub からインストール
    GitHub { repository: String },
    /// マーケットプレイスからインストール
    Marketplace { name: String },
}

/// コンポーネント一覧
#[derive(Debug, Clone, Serialize)]
pub struct ComponentInfo {
    pub skills: Vec<String>,
    pub agents: Vec<String>,
    pub commands: Vec<String>,
    pub instructions: Vec<String>,
    pub hooks: Vec<String>,
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
fn determine_source_from_path(_cache_path: &Path, marketplace: &str, dir_name: &str) -> PluginSource {
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
    let enabled = check_deployed_status(&candidate.marketplace, &manifest.name);

    // キャッシュパス（絶対パス）
    let cache_path = candidate
        .cache_path
        .to_string_lossy()
        .to_string();

    Ok(PluginDetail {
        name: manifest.name.clone(),
        version: manifest.version.clone(),
        description: manifest.description.clone(),
        author,
        installed_at: meta::resolve_installed_at(&candidate.cache_path, Some(&manifest)),
        source,
        components,
        enabled,
        cache_path,
    })
}

/// デプロイ状態を判定
fn check_deployed_status(marketplace: &str, plugin_name: &str) -> bool {
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let deployed = collect_deployed_plugins(&project_root);

    let origin = PluginOrigin::from_cached_plugin(
        if marketplace == "github" {
            None
        } else {
            Some(marketplace)
        },
        plugin_name,
    );
    deployed.contains(&(origin.marketplace, origin.plugin))
}

/// デプロイ済みプラグインの集合を取得
fn collect_deployed_plugins(project_root: &Path) -> HashSet<(String, String)> {
    let mut deployed = HashSet::new();
    let targets = all_targets();

    for target in &targets {
        for kind in ComponentKind::all() {
            if !target.supports(*kind) {
                continue;
            }
            // エラー時は警告のみで継続
            match target.list_placed(*kind, Scope::Project, project_root) {
                Ok(placed) => {
                    for item in placed {
                        if let Some((mp, plugin)) = parse_placed_item(&item) {
                            deployed.insert((mp, plugin));
                        }
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Warning: failed to list placed components for {:?}: {}",
                        kind, e
                    );
                }
            }
        }
    }
    deployed
}

/// "marketplace/plugin/component" 形式をパース
fn parse_placed_item(item: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = item.split('/').collect();
    if parts.len() >= 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // parse_plugin_name tests
    // ========================================

    #[test]
    fn test_parse_plugin_name_simple() {
        let result = parse_plugin_name("my-plugin").unwrap();
        assert_eq!(result, (None, "my-plugin".to_string()));
    }

    #[test]
    fn test_parse_plugin_name_with_marketplace() {
        let result = parse_plugin_name("marketplace/plugin").unwrap();
        assert_eq!(
            result,
            (Some("marketplace".to_string()), "plugin".to_string())
        );
    }

    #[test]
    fn test_parse_plugin_name_empty() {
        let result = parse_plugin_name("");
        assert!(result.is_err());
        match result.unwrap_err() {
            PlmError::InvalidArgument(msg) => {
                assert!(msg.contains("empty"));
            }
            e => panic!("Expected InvalidArgument, got: {:?}", e),
        }
    }

    #[test]
    fn test_parse_plugin_name_leading_slash() {
        let result = parse_plugin_name("/plugin");
        assert!(result.is_err());
        match result.unwrap_err() {
            PlmError::InvalidArgument(msg) => {
                assert!(msg.contains("start with"));
            }
            e => panic!("Expected InvalidArgument, got: {:?}", e),
        }
    }

    #[test]
    fn test_parse_plugin_name_trailing_slash() {
        let result = parse_plugin_name("plugin/");
        assert!(result.is_err());
        match result.unwrap_err() {
            PlmError::InvalidArgument(msg) => {
                assert!(msg.contains("end with"));
            }
            e => panic!("Expected InvalidArgument, got: {:?}", e),
        }
    }

    #[test]
    fn test_parse_plugin_name_too_many_slashes() {
        let result = parse_plugin_name("a/b/c");
        assert!(result.is_err());
        match result.unwrap_err() {
            PlmError::InvalidArgument(msg) => {
                assert!(msg.contains("too many"));
            }
            e => panic!("Expected InvalidArgument, got: {:?}", e),
        }
    }

    // ========================================
    // restore_github_repo tests
    // ========================================

    #[test]
    fn test_restore_github_repo_normal() {
        assert_eq!(restore_github_repo("owner--repo"), "owner/repo");
    }

    #[test]
    fn test_restore_github_repo_no_separator() {
        assert_eq!(restore_github_repo("simple-name"), "simple-name");
    }

    #[test]
    fn test_restore_github_repo_multiple_dashes() {
        assert_eq!(
            restore_github_repo("owner--repo--extra"),
            "owner/repo--extra"
        );
    }

    // ========================================
    // resolve_single_plugin tests
    // ========================================

    fn create_candidate(marketplace: &str, name: &str) -> PluginCandidate {
        PluginCandidate {
            marketplace: marketplace.to_string(),
            dir_name: name.to_string(),
            cache_path: PathBuf::from(format!("/cache/{}/{}", marketplace, name)),
            manifest: PluginManifest {
                name: name.to_string(),
                version: "1.0.0".to_string(),
                description: None,
                author: None,
                homepage: None,
                repository: None,
                license: None,
                keywords: None,
                commands: None,
                agents: None,
                skills: None,
                instructions: None,
                hooks: None,
                mcp_servers: None,
                lsp_servers: None,
                installed_at: None,
            },
        }
    }

    #[test]
    fn test_resolve_single_plugin_not_found() {
        let result = resolve_single_plugin(vec![], None, "missing");
        assert!(result.is_err());
        match result.unwrap_err() {
            PlmError::PluginNotFound(name) => {
                assert_eq!(name, "missing");
            }
            e => panic!("Expected PluginNotFound, got: {:?}", e),
        }
    }

    #[test]
    fn test_resolve_single_plugin_one_match() {
        let candidates = vec![create_candidate("github", "my-plugin")];
        let result = resolve_single_plugin(candidates, None, "my-plugin");
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert_eq!(resolved.marketplace, "github");
    }

    #[test]
    fn test_resolve_single_plugin_multiple_ambiguous() {
        let candidates = vec![
            create_candidate("marketplace-a", "common"),
            create_candidate("marketplace-b", "common"),
        ];
        let result = resolve_single_plugin(candidates, None, "common");
        assert!(result.is_err());
        match result.unwrap_err() {
            PlmError::AmbiguousPlugin { name, candidates } => {
                assert_eq!(name, "common");
                assert_eq!(candidates.len(), 2);
            }
            e => panic!("Expected AmbiguousPlugin, got: {:?}", e),
        }
    }

    #[test]
    fn test_resolve_single_plugin_filtered_by_marketplace() {
        let candidates = vec![
            create_candidate("marketplace-a", "common"),
            create_candidate("marketplace-b", "common"),
        ];
        let result = resolve_single_plugin(candidates, Some("marketplace-a"), "common");
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert_eq!(resolved.marketplace, "marketplace-a");
    }

    #[test]
    fn test_resolve_single_plugin_filtered_not_found() {
        let candidates = vec![create_candidate("marketplace-a", "common")];
        let result = resolve_single_plugin(candidates, Some("marketplace-b"), "common");
        assert!(result.is_err());
        match result.unwrap_err() {
            PlmError::PluginNotFound(name) => {
                assert_eq!(name, "common");
            }
            e => panic!("Expected PluginNotFound, got: {:?}", e),
        }
    }

    // ========================================
    // determine_source_from_path tests
    // ========================================

    #[test]
    fn test_determine_source_github() {
        let path = PathBuf::from("/cache/github/owner--repo");
        let source = determine_source_from_path(&path, "github", "owner--repo");
        match source {
            PluginSource::GitHub { repository } => {
                assert_eq!(repository, "owner/repo");
            }
            _ => panic!("Expected GitHub source"),
        }
    }

    #[test]
    fn test_determine_source_marketplace() {
        let path = PathBuf::from("/cache/awesome-plugins/my-plugin");
        let source = determine_source_from_path(&path, "awesome-plugins", "my-plugin");
        match source {
            PluginSource::Marketplace { name } => {
                assert_eq!(name, "awesome-plugins");
            }
            _ => panic!("Expected Marketplace source"),
        }
    }
}
