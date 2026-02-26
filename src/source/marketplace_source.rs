//! Marketplace 経由のダウンロード

use crate::error::{PlmError, Result};
use crate::marketplace::{MarketplaceRegistry, PluginSource as MpPluginSource, PluginSourcePath};
use crate::plugin::{CachedPlugin, PluginCacheAccess};
use crate::repo;
use std::future::Future;
use std::pin::Pin;

use super::{GitHubSource, PluginSource};

/// 指定した Marketplace からプラグインをダウンロードするソース
pub struct MarketplaceSource {
    plugin: String,
    marketplace: String,
}

impl MarketplaceSource {
    pub fn new(plugin: &str, marketplace: &str) -> Self {
        Self {
            plugin: plugin.to_string(),
            marketplace: marketplace.to_string(),
        }
    }
}

impl PluginSource for MarketplaceSource {
    fn download<'a>(
        &'a self,
        cache: &'a dyn PluginCacheAccess,
        force: bool,
    ) -> Pin<Box<dyn Future<Output = Result<CachedPlugin>> + Send + 'a>> {
        Box::pin(async move {
            let registry = MarketplaceRegistry::new()?;

            // マーケットプレイスからプラグイン情報を取得
            let mp_cache = registry
                .get(&self.marketplace)?
                .ok_or_else(|| PlmError::MarketplaceNotFound(self.marketplace.clone()))?;

            let plugin_entry = mp_cache
                .plugins
                .iter()
                .find(|p| p.name == self.plugin)
                .ok_or_else(|| PlmError::PluginNotFound(self.plugin.clone()))?;

            // プラグインソースをRepoに変換してダウンロード
            match &plugin_entry.source {
                MpPluginSource::Local(path) => {
                    let parts: Vec<&str> = mp_cache
                        .source
                        .strip_prefix("github:")
                        .unwrap_or(&mp_cache.source)
                        .split('/')
                        .collect();

                    if parts.len() < 2 {
                        return Err(PlmError::InvalidRepoFormat(mp_cache.source.clone()));
                    }

                    let owner = parts[0];
                    let repo_name = parts[1];
                    let repo = repo::from_url(&format!("{}/{}", owner, repo_name))?;

                    // path を正規化・検証
                    let source_path: PluginSourcePath = path.parse()?;

                    // Git ソースに委譲（marketplace + source_path 情報を渡す）
                    GitHubSource::with_marketplace_and_source_path(
                        repo,
                        self.marketplace.clone(),
                        source_path.into(),
                    )
                    .download(cache, force)
                    .await
                }
                MpPluginSource::External { repo: repo_url, .. } => {
                    let repo = repo::from_url(repo_url)?;
                    // Git ソースに委譲（marketplace 情報を渡す）
                    GitHubSource::with_marketplace(repo, self.marketplace.clone())
                        .download(cache, force)
                        .await
                }
            }
        })
    }
}
