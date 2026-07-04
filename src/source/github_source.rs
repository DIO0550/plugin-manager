//! Git リポジトリからのダウンロード

use crate::error::Result;
use crate::host::HostClientFactory;
use crate::plugin::{meta, CachedPackage, GithubCacheId, PackageCacheAccess};
use crate::repo::Repo;
use std::future::Future;
use std::pin::Pin;

use super::PackageSource;

/// ダウンロードの文脈（直接 GitHub か marketplace 経由か）
///
/// 有効な組み合わせを型で表現し、不正状態（marketplace なしの
/// plugin_identifier 等）を表現不能にする。
enum SourceContext {
    /// 直接 GitHub install（cache key は `{owner}--{repo}`）
    Direct,
    /// marketplace 経由 install
    Marketplace {
        /// marketplace 名
        name: String,
        /// marketplace 内でユニークなプラグイン識別子（validated 済み、cache key として使用）
        plugin_identifier: String,
        /// Local プラグインのソースパス（正規化済み）。External プラグインは `None`
        source_path: Option<String>,
    },
}

/// Git リポジトリからプラグインをダウンロードするソース
///
/// GitHub, GitLab, Bitbucket 等のホスティングサービスに対応。
pub struct GitHubSource {
    repo: Repo,
    context: SourceContext,
}

impl GitHubSource {
    /// Create a direct Git repository source with no marketplace association.
    ///
    /// # Arguments
    ///
    /// * `repo` - Repository descriptor identifying the Git host, owner, name, and optional ref.
    pub fn new(repo: Repo) -> Self {
        Self {
            repo,
            context: SourceContext::Direct,
        }
    }

    /// marketplace 経由インストール用コンストラクタ。
    ///
    /// # Arguments
    ///
    /// * `repo` - Repository descriptor for the underlying Git source.
    /// * `marketplace` - Name of the marketplace that surfaced this plugin.
    /// * `source_path` - Optional normalized sub-path for Local plugins, `None` for External.
    /// * `plugin_identifier` - Validated unique plugin name within the marketplace.
    pub fn with_marketplace_plugin(
        repo: Repo,
        marketplace: String,
        source_path: Option<String>,
        plugin_identifier: String,
    ) -> Self {
        Self {
            repo,
            context: SourceContext::Marketplace {
                name: marketplace,
                plugin_identifier,
                source_path,
            },
        }
    }

    /// marketplace 名（直接 GitHub の場合は `None`）
    fn marketplace_name(&self) -> Option<&str> {
        match &self.context {
            SourceContext::Direct => None,
            SourceContext::Marketplace { name, .. } => Some(name),
        }
    }

    /// Local プラグインのソースパス（直接 GitHub / External の場合は `None`）
    fn source_path(&self) -> Option<&str> {
        match &self.context {
            SourceContext::Direct => None,
            SourceContext::Marketplace { source_path, .. } => source_path.as_deref(),
        }
    }

    /// このソースの cache key を算出する
    ///
    /// - marketplace 経由: `plugin_identifier`
    /// - 直接 GitHub: `"{owner}--{repo}"`（`GithubCacheId`）
    fn compute_cache_name(&self) -> String {
        match &self.context {
            SourceContext::Direct => GithubCacheId::from_repo(&self.repo).into_string(),
            SourceContext::Marketplace {
                plugin_identifier, ..
            } => plugin_identifier.clone(),
        }
    }
}

impl PackageSource for GitHubSource {
    fn download<'a>(
        &'a self,
        cache: &'a dyn PackageCacheAccess,
        force: bool,
    ) -> Pin<Box<dyn Future<Output = Result<CachedPackage>> + Send + 'a>> {
        Box::pin(async move {
            let factory = HostClientFactory::with_defaults();
            let client = factory.create(self.repo.host());
            let display_name = self.repo.name();
            let marketplace = self.marketplace_name();

            let cache_name = self.compute_cache_name();
            let log_label = match &self.context {
                SourceContext::Direct => display_name,
                SourceContext::Marketplace {
                    plugin_identifier, ..
                } => plugin_identifier,
            };

            if !force && cache.is_cached(marketplace, &cache_name) {
                println!(
                    "Using cached plugin: {} (cache key: {})",
                    log_label, cache_name
                );
                return cache.load_package(marketplace, &cache_name);
            }

            println!(
                "Downloading plugin from {}/{}...",
                self.repo.owner(),
                self.repo.name()
            );
            let (archive, git_ref, commit_sha) =
                client.download_archive_with_sha(&self.repo).await?;

            println!("Extracting to cache...");
            let plugin_path =
                cache.store_from_archive(marketplace, &cache_name, &archive, self.source_path())?;

            let manifest = cache.load_manifest(marketplace, &cache_name)?;

            // store_from_archive で installedAt は既に書き込まれているので、追加フィールドのみ更新
            let mut plugin_meta = meta::load_meta(&plugin_path).unwrap_or_default();
            plugin_meta.set_source_repo(self.repo.owner(), self.repo.name());
            plugin_meta.set_git_info(&git_ref, &commit_sha);
            plugin_meta.marketplace = Some("github".to_string());
            if let Err(e) = meta::write_meta(&plugin_path, &plugin_meta) {
                eprintln!("Warning: Failed to save plugin metadata: {}", e);
            }

            Ok(CachedPackage {
                name: manifest.name.clone(),
                id: Some(cache_name.clone()),
                marketplace: self.marketplace_name().map(String::from),
                path: plugin_path,
                manifest,
                git_ref,
                commit_sha,
                marketplace_manifest: None,
            })
        })
    }
}
