//! GitHub からのダウンロード

use crate::error::Result;
use crate::github::{GitHubClient, GitRepo};
use crate::plugin::{CachedPlugin, PluginCache};
use std::future::Future;
use std::pin::Pin;

use super::PluginSource;

/// GitHub リポジトリからプラグインをダウンロードするソース
pub struct GitHubSource {
    repo: GitRepo,
}

impl GitHubSource {
    pub fn new(repo: GitRepo) -> Self {
        Self { repo }
    }
}

impl PluginSource for GitHubSource {
    fn download(&self, force: bool) -> Pin<Box<dyn Future<Output = Result<CachedPlugin>> + Send + '_>> {
        Box::pin(async move {
            let github = GitHubClient::new();
            let cache = PluginCache::new()?;
            let plugin_name = &self.repo.repo;

            // キャッシュチェック
            if !force && cache.is_cached(plugin_name) {
                println!("Using cached plugin: {}", plugin_name);
                let manifest = cache.load_manifest(plugin_name)?;
                return Ok(CachedPlugin {
                    name: plugin_name.clone(),
                    path: cache.plugin_path(plugin_name),
                    manifest,
                    git_ref: self.repo.git_ref.clone().unwrap_or_else(|| "main".to_string()),
                    commit_sha: "cached".to_string(),
                });
            }

            // ダウンロード
            println!("Downloading plugin from {}/{}...", self.repo.owner, self.repo.repo);
            let (archive, git_ref, commit_sha) = github.download_archive_with_sha(&self.repo).await?;

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
        })
    }
}

