//! Marketplace 検索によるダウンロード

use crate::error::{PlmError, Result};
use crate::marketplace::{MarketplaceConfig, MarketplaceRegistry};
use crate::plugin::{CachedPackage, PackageCacheAccess};
use std::future::Future;
use std::pin::Pin;

use super::{MarketplaceSource, PackageSource};

/// 全 Marketplace を検索してプラグインをダウンロードするソース
pub struct SearchSource {
    query: String,
}

impl SearchSource {
    /// Create a source that searches all registered marketplaces for a plugin name.
    ///
    /// # Arguments
    ///
    /// * `query` - Plugin name to look up across every registered marketplace.
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
        }
    }
}

impl PackageSource for SearchSource {
    fn download<'a>(
        &'a self,
        cache: &'a dyn PackageCacheAccess,
        force: bool,
    ) -> Pin<Box<dyn Future<Output = Result<CachedPackage>> + Send + 'a>> {
        Box::pin(async move {
            let registry = MarketplaceRegistry::new()?;

            let config = MarketplaceConfig::load().map_err(|e| {
                PlmError::Cache(format!("Failed to load marketplace config: {}", e))
            })?;
            let registered_names: Vec<&str> =
                config.list().iter().map(|e| e.name.as_str()).collect();

            if registered_names.is_empty() {
                return Err(PlmError::PluginNotFound(format!(
                    "{} (no marketplaces registered; use 'plm marketplace add <owner/repo>' to add one)",
                    self.query
                )));
            }

            let matches = registry.find_plugins(&self.query)?;
            let matches: Vec<_> = matches
                .into_iter()
                .filter(|m| registered_names.contains(&m.marketplace.as_str()))
                .collect();

            if matches.is_empty() {
                let mut uncached: Vec<&str> = Vec::new();
                for name in &registered_names {
                    if registry.get(name)?.is_none() {
                        uncached.push(name);
                    }
                }

                if !uncached.is_empty() {
                    return Err(PlmError::PluginNotFound(format!(
                        "{}; some marketplaces have no cache: {}. Run 'plm marketplace update' to fetch plugin information.",
                        self.query,
                        uncached.join(", ")
                    )));
                }

                return Err(PlmError::PluginNotFound(self.query.clone()));
            }

            if matches.len() > 1 {
                let marketplace_names: Vec<_> =
                    matches.iter().map(|m| m.marketplace.as_str()).collect();
                return Err(PlmError::InvalidArgument(format!(
                    "Plugin '{}' found in multiple marketplaces: {}. Use '{}@<marketplace>' to specify which one.",
                    self.query,
                    marketplace_names.join(", "),
                    self.query
                )));
            }

            let plugin_match = &matches[0];
            MarketplaceSource::new(&self.query, &plugin_match.marketplace)
                .download(cache, force)
                .await
        })
    }
}
