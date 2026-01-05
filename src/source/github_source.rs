//! Git リポジトリからのダウンロード

use crate::error::Result;
use crate::host::HostClientFactory;
use crate::plugin::{CachedPlugin, PluginCache};
use crate::repo::Repo;
use std::future::Future;
use std::pin::Pin;

use super::PluginSource;

/// Git リポジトリからプラグインをダウンロードするソース
///
/// GitHub, GitLab, Bitbucket 等のホスティングサービスに対応。
pub struct GitHubSource {
    repo: Repo,
    /// マーケットプレイス経由の場合はその名前
    marketplace: Option<String>,
    /// 抽出するサブディレクトリ（正規化済み）
    /// マーケットプレイス内の Local プラグイン用
    subdir: Option<String>,
}

impl GitHubSource {
    pub fn new(repo: Repo) -> Self {
        Self {
            repo,
            marketplace: None,
            subdir: None,
        }
    }

    /// マーケットプレイス経由でのソースを作成
    pub fn with_marketplace(repo: Repo, marketplace: String) -> Self {
        Self {
            repo,
            marketplace: Some(marketplace),
            subdir: None,
        }
    }

    /// マーケットプレイス経由 + サブディレクトリ指定でのソース作成
    /// Local プラグイン専用: marketplace と subdir は両方必須
    pub fn with_marketplace_and_subdir(repo: Repo, marketplace: String, subdir: String) -> Self {
        Self {
            repo,
            marketplace: Some(marketplace),
            subdir: Some(subdir),
        }
    }
}

impl PluginSource for GitHubSource {
    fn download(
        &self,
        force: bool,
    ) -> Pin<Box<dyn Future<Output = Result<CachedPlugin>> + Send + '_>> {
        Box::pin(async move {
            let factory = HostClientFactory::with_defaults();
            let client = factory.create(self.repo.host());
            let cache = PluginCache::new()?;
            let plugin_name = self.repo.name();
            let marketplace = self.marketplace.as_deref();

            // 直接GitHubインストールの場合は owner--repo 形式にする
            let cache_name = if self.marketplace.is_none() {
                format!("{}--{}", self.repo.owner(), self.repo.name())
            } else {
                plugin_name.to_string()
            };

            // キャッシュチェック
            if !force && cache.is_cached(marketplace, &cache_name) {
                println!("Using cached plugin: {}", plugin_name);
                let manifest = cache.load_manifest(marketplace, &cache_name)?;
                return Ok(CachedPlugin {
                    name: plugin_name.to_string(),
                    marketplace: self.marketplace.clone(),
                    path: cache.plugin_path(marketplace, &cache_name),
                    manifest,
                    git_ref: self
                        .repo
                        .git_ref()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "main".to_string()),
                    commit_sha: "cached".to_string(),
                });
            }

            // ダウンロード
            println!(
                "Downloading plugin from {}/{}...",
                self.repo.owner(),
                self.repo.name()
            );
            let (archive, git_ref, commit_sha) =
                client.download_archive_with_sha(&self.repo).await?;

            // キャッシュに保存
            println!("Extracting to cache...");
            cache.store_from_archive(marketplace, &cache_name, &archive, self.subdir.as_deref())?;

            // マニフェスト読み込み
            let manifest = cache.load_manifest(marketplace, &cache_name)?;

            Ok(CachedPlugin {
                name: manifest.name.clone(),
                marketplace: self.marketplace.clone(),
                path: cache.plugin_path(marketplace, &cache_name),
                manifest,
                git_ref,
                commit_sha,
            })
        })
    }
}
