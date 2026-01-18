//! Marketplace 検索によるダウンロード

use crate::error::{PlmError, Result};
use crate::marketplace::MarketplaceRegistry;
use crate::plugin::CachedPlugin;
use std::future::Future;
use std::pin::Pin;

use super::{MarketplaceSource, PluginSource};

/// 全 Marketplace を検索してプラグインをダウンロードするソース
pub struct SearchSource {
    query: String,
}

impl SearchSource {
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
        }
    }
}

impl PluginSource for SearchSource {
    fn download(
        &self,
        force: bool,
    ) -> Pin<Box<dyn Future<Output = Result<CachedPlugin>> + Send + '_>> {
        Box::pin(async move {
            let registry = MarketplaceRegistry::new()?;

            // 全マーケットプレイスからプラグインを検索
            let (marketplace, _plugin_entry) = registry
                .find_plugin(&self.query)?
                .ok_or_else(|| PlmError::PluginNotFound(self.query.clone()))?;

            // MarketplaceSource に委譲
            MarketplaceSource::new(&self.query, &marketplace)
                .download(force)
                .await
        })
    }
}
