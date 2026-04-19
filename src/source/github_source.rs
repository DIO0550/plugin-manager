//! Git リポジトリからのダウンロード

use crate::error::Result;
use crate::host::HostClientFactory;
use crate::plugin::{meta, CachedPackage, PackageCacheAccess};
use crate::repo::Repo;
use std::future::Future;
use std::pin::Pin;

use super::PackageSource;

/// Git リポジトリからプラグインをダウンロードするソース
///
/// GitHub, GitLab, Bitbucket 等のホスティングサービスに対応。
pub struct GitHubSource {
    repo: Repo,
    /// マーケットプレイス経由の場合はその名前
    marketplace: Option<String>,
    /// プラグインのソースパス（正規化済み）
    /// マーケットプレイス内の Local プラグイン用
    source_path: Option<String>,
}

impl GitHubSource {
    /// Create a direct Git repository source with no marketplace association.
    ///
    /// # Arguments
    /// * `repo` - Repository descriptor identifying the Git host, owner, name, and optional ref.
    pub fn new(repo: Repo) -> Self {
        Self {
            repo,
            marketplace: None,
            source_path: None,
        }
    }

    /// マーケットプレイス経由でのソースを作成
    ///
    /// # Arguments
    /// * `repo` - Repository descriptor for the underlying Git source.
    /// * `marketplace` - Name of the marketplace that surfaced this plugin.
    pub fn with_marketplace(repo: Repo, marketplace: String) -> Self {
        Self {
            repo,
            marketplace: Some(marketplace),
            source_path: None,
        }
    }

    /// マーケットプレイス経由 + ソースパス指定でのソース作成
    /// Local プラグイン専用: marketplace と source_path は両方必須
    ///
    /// # Arguments
    /// * `repo` - Repository descriptor for the underlying Git source.
    /// * `marketplace` - Name of the marketplace that surfaced this plugin.
    /// * `source_path` - Normalized sub-path within the repository pointing at the local plugin.
    pub fn with_marketplace_and_source_path(
        repo: Repo,
        marketplace: String,
        source_path: String,
    ) -> Self {
        Self {
            repo,
            marketplace: Some(marketplace),
            source_path: Some(source_path),
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
            let plugin_name = self.repo.name();
            let marketplace = self.marketplace.as_deref();

            // 直接GitHubインストールの場合は owner--repo 形式にする
            let cache_name = if self.marketplace.is_none() {
                format!("{}--{}", self.repo.owner(), self.repo.name())
            } else {
                plugin_name.to_string()
            };

            if !force && cache.is_cached(marketplace, &cache_name) {
                println!(
                    "Using cached plugin: {} (cache key: {})",
                    plugin_name, cache_name
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
            let plugin_path = cache.store_from_archive(
                marketplace,
                &cache_name,
                &archive,
                self.source_path.as_deref(),
            )?;

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
                cache_key: Some(cache_name.clone()),
                marketplace: self.marketplace.clone(),
                path: plugin_path,
                manifest,
                git_ref,
                commit_sha,
                marketplace_manifest: None,
            })
        })
    }
}
