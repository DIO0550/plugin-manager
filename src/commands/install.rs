use crate::error::{PlmError, Result};
use crate::github::{GitHubClient, RepoRef};
use crate::marketplace::{MarketplaceRegistry, PluginSource as MpPluginSource};
use crate::plugin::{CachedPlugin, PluginCache, PluginManifest};
use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum ComponentType {
    Skill,
    Agent,
    Prompt,
    Instruction,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum TargetKind {
    Codex,
    Copilot,
}

#[derive(Debug, Parser)]
pub struct Args {
    /// owner/repo 形式、または plugin@marketplace 形式
    pub source: String,

    /// コンポーネント種別を指定（未指定なら自動検出の想定）
    #[arg(long = "type", value_enum)]
    pub component_type: Option<ComponentType>,

    /// 特定ターゲットにだけ入れる
    #[arg(long, value_enum)]
    pub target: Option<TargetKind>,

    /// personal / project
    #[arg(long)]
    pub scope: Option<String>,

    /// キャッシュを無視して再ダウンロード
    #[arg(long)]
    pub force: bool,
}

/// インストール元の指定
#[derive(Debug, Clone)]
pub enum InstallSource {
    /// GitHub直接: owner/repo[@ref]
    GitHub(RepoRef),
    /// Marketplace経由: plugin@marketplace
    Marketplace { plugin: String, marketplace: String },
    /// Marketplace検索: plugin
    Search(String),
}

impl InstallSource {
    /// 引数文字列からパース
    pub fn parse(input: &str) -> Result<Self> {
        // "@" を含む場合
        if let Some((left, right)) = input.split_once('@') {
            // "owner/repo@ref" の場合（GitHubリポジトリ）
            if left.contains('/') {
                let repo = RepoRef::parse(input)?;
                return Ok(InstallSource::GitHub(repo));
            }

            // "plugin@marketplace" の場合
            return Ok(InstallSource::Marketplace {
                plugin: left.to_string(),
                marketplace: right.to_string(),
            });
        }

        // "/" を含む場合はGitHubリポジトリ
        if input.contains('/') {
            let repo = RepoRef::parse(input)?;
            return Ok(InstallSource::GitHub(repo));
        }

        // それ以外はMarketplace検索
        Ok(InstallSource::Search(input.to_string()))
    }
}

/// プラグインをダウンロード
pub fn download_plugin(
    source: &InstallSource,
    force: bool,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<CachedPlugin>> + Send + '_>> {
    Box::pin(async move {
        download_plugin_inner(source, force).await
    })
}

async fn download_plugin_inner(source: &InstallSource, force: bool) -> Result<CachedPlugin> {
    let github = GitHubClient::new();
    let cache = PluginCache::new()?;

    match source {
        InstallSource::GitHub(repo) => {
            let plugin_name = &repo.repo;

            // キャッシュチェック
            if !force && cache.is_cached(plugin_name) {
                println!("Using cached plugin: {}", plugin_name);
                let manifest = cache.load_manifest(plugin_name)?;
                return Ok(CachedPlugin {
                    name: plugin_name.clone(),
                    path: cache.plugin_path(plugin_name),
                    manifest,
                    git_ref: repo.git_ref.clone().unwrap_or_else(|| "main".to_string()),
                    commit_sha: "cached".to_string(),
                });
            }

            // ダウンロード
            println!("Downloading plugin from {}/{}...", repo.owner, repo.repo);
            let (archive, git_ref, commit_sha) = github.download_archive_with_sha(repo).await?;

            // キャッシュに保存
            println!("Extracting to cache...");
            cache.store_from_archive(plugin_name, &archive)?;

            // マニフェスト読み込み
            let manifest = cache.load_manifest(plugin_name)?;

            Ok(CachedPlugin {
                name: manifest.name.clone(),
                path: cache.plugin_path(plugin_name),
                manifest,
                git_ref,
                commit_sha,
            })
        }

        InstallSource::Marketplace { plugin, marketplace } => {
            let registry = MarketplaceRegistry::new()?;

            // マーケットプレイスからプラグイン情報を取得
            let mp_cache = registry.get(marketplace)?.ok_or_else(|| {
                PlmError::MarketplaceNotFound(marketplace.clone())
            })?;

            let plugin_entry = mp_cache
                .plugins
                .iter()
                .find(|p| p.name == *plugin)
                .ok_or_else(|| PlmError::PluginNotFound(plugin.clone()))?;

            // プラグインソースを解決
            let repo = match &plugin_entry.source {
                MpPluginSource::Local(path) => {
                    // マーケットプレイスリポジトリ内の相対パス
                    // source形式: "github:owner/repo"
                    let mp_source = &mp_cache.source;
                    let parts: Vec<&str> = mp_source
                        .strip_prefix("github:")
                        .unwrap_or(mp_source)
                        .split('/')
                        .collect();

                    if parts.len() < 2 {
                        return Err(PlmError::InvalidRepoFormat(mp_source.clone()));
                    }

                    // サブディレクトリ付きでダウンロード
                    // TODO: サブディレクトリのみダウンロードするには追加実装が必要
                    RepoRef {
                        owner: parts[0].to_string(),
                        repo: parts[1].to_string(),
                        git_ref: None,
                    }
                }
                MpPluginSource::External { repo, .. } => {
                    RepoRef::parse(repo)?
                }
            };

            // 再帰的にGitHubからダウンロード
            let github_source = InstallSource::GitHub(repo);
            download_plugin(&github_source, force).await
        }

        InstallSource::Search(plugin_name) => {
            let registry = MarketplaceRegistry::new()?;

            // 全マーケットプレイスからプラグインを検索
            let (marketplace, _plugin_entry) = registry
                .find_plugin(plugin_name)?
                .ok_or_else(|| PlmError::PluginNotFound(plugin_name.clone()))?;

            // 見つかったマーケットプレイスからインストール
            let mp_source = InstallSource::Marketplace {
                plugin: plugin_name.clone(),
                marketplace,
            };
            download_plugin(&mp_source, force).await
        }
    }
}

pub async fn run(args: Args) -> std::result::Result<(), String> {
    // インストール元をパース
    let source = InstallSource::parse(&args.source).map_err(|e| e.to_string())?;

    // ダウンロード
    let cached_plugin = download_plugin(&source, args.force)
        .await
        .map_err(|e| e.to_string())?;

    println!("\nPlugin downloaded successfully!");
    println!("  Name: {}", cached_plugin.name);
    println!("  Version: {}", cached_plugin.manifest.version);
    println!("  Path: {}", cached_plugin.path.display());
    println!("  Ref: {}", cached_plugin.git_ref);
    println!("  SHA: {}", cached_plugin.commit_sha);

    if let Some(desc) = &cached_plugin.manifest.description {
        println!("  Description: {}", desc);
    }

    // コンポーネント情報
    println!("\nComponents:");
    if cached_plugin.manifest.has_skills() {
        println!("  - Skills: {}", cached_plugin.manifest.skills.as_ref().unwrap());
    }
    if cached_plugin.manifest.has_agents() {
        println!("  - Agents: {}", cached_plugin.manifest.agents.as_ref().unwrap());
    }
    if cached_plugin.manifest.has_commands() {
        println!("  - Commands: {}", cached_plugin.manifest.commands.as_ref().unwrap());
    }

    // TODO: ターゲットへのデプロイ処理を実装
    println!("\nNote: Deployment to targets not yet implemented");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_repo() {
        match InstallSource::parse("owner/repo").unwrap() {
            InstallSource::GitHub(repo) => {
                assert_eq!(repo.owner, "owner");
                assert_eq!(repo.repo, "repo");
                assert!(repo.git_ref.is_none());
            }
            _ => panic!("Expected GitHub source"),
        }
    }

    #[test]
    fn test_parse_github_repo_with_ref() {
        match InstallSource::parse("owner/repo@v1.0.0").unwrap() {
            InstallSource::GitHub(repo) => {
                assert_eq!(repo.owner, "owner");
                assert_eq!(repo.repo, "repo");
                assert_eq!(repo.git_ref, Some("v1.0.0".to_string()));
            }
            _ => panic!("Expected GitHub source"),
        }
    }

    #[test]
    fn test_parse_marketplace() {
        match InstallSource::parse("plugin@marketplace").unwrap() {
            InstallSource::Marketplace { plugin, marketplace } => {
                assert_eq!(plugin, "plugin");
                assert_eq!(marketplace, "marketplace");
            }
            _ => panic!("Expected Marketplace source"),
        }
    }

    #[test]
    fn test_parse_search() {
        match InstallSource::parse("plugin-name").unwrap() {
            InstallSource::Search(name) => {
                assert_eq!(name, "plugin-name");
            }
            _ => panic!("Expected Search source"),
        }
    }
}
