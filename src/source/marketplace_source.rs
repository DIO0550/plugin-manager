//! Marketplace 経由のダウンロード

use crate::error::{PlmError, Result};
use crate::marketplace::{MarketplaceRegistry, PluginSource as MpPluginSource};
use crate::plugin::CachedPlugin;
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
    fn download(
        &self,
        force: bool,
    ) -> Pin<Box<dyn Future<Output = Result<CachedPlugin>> + Send + '_>> {
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

            // プラグインソースをRepoに変換
            let repo = match &plugin_entry.source {
                MpPluginSource::Local(_path) => {
                    let parts: Vec<&str> = mp_cache
                        .source
                        .strip_prefix("github:")
                        .unwrap_or(&mp_cache.source)
                        .split('/')
                        .collect();

                    if parts.len() < 2 {
                        return Err(PlmError::InvalidRepoFormat(mp_cache.source.clone()));
                    }

                    repo::from_url(&format!("{}/{}", parts[0], parts[1]))?
                }
                MpPluginSource::External { repo, .. } => repo::from_url(repo)?,
            };

            // Git ソースに委譲
            GitHubSource::new(repo).download(force).await
        })
    }
}
