//! Marketplace 経由のダウンロード

use crate::error::{PlmError, Result};
use crate::marketplace::{
    validate_plugin_names, MarketplaceManifest, MarketplaceRegistry,
    PluginSource as MpPluginSource, PluginSourcePath,
};
use crate::plugin::{CachedPackage, LegacyCacheCleaner, PackageCacheAccess};
use crate::repo;
use std::future::Future;
use std::pin::Pin;

use super::{GitHubSource, PackageSource};

/// 指定した Marketplace からプラグインをダウンロードするソース
pub struct MarketplaceSource {
    plugin: String,
    marketplace: String,
}

impl MarketplaceSource {
    /// Create a source that resolves a plugin through a specific marketplace.
    ///
    /// # Arguments
    ///
    /// * `plugin` - Name of the plugin to resolve inside the marketplace.
    /// * `marketplace` - Name of the registered marketplace to query.
    pub fn new(plugin: &str, marketplace: &str) -> Self {
        Self {
            plugin: plugin.to_string(),
            marketplace: marketplace.to_string(),
        }
    }
}

impl PackageSource for MarketplaceSource {
    fn download<'a>(
        &'a self,
        cache: &'a dyn PackageCacheAccess,
        force: bool,
    ) -> Pin<Box<dyn Future<Output = Result<CachedPackage>> + Send + 'a>> {
        Box::pin(async move {
            let registry = MarketplaceRegistry::new()?;

            let mp_cache = registry
                .get(&self.marketplace)?
                .ok_or_else(|| PlmError::MarketplaceNotFound(self.marketplace.clone()))?;

            // 取得した manifest の plugin name 群を再検証（保険）
            validate_plugin_names(&mp_cache.plugins)?;

            // 旧レイアウト残骸の自動掃除（ピンポイント。空 plugins / rename / 別形式は no-op）
            LegacyCacheCleaner::clean_if_legacy(cache, &self.marketplace, &mp_cache)?;

            let plugin_entry = mp_cache
                .plugins
                .iter()
                .find(|p| p.name == self.plugin)
                .ok_or_else(|| PlmError::PluginNotFound(self.plugin.clone()))?;

            let marketplace_manifest = Some(MarketplaceManifest {
                name: mp_cache.name.clone(),
                owner: mp_cache.owner.clone(),
                plugins: mp_cache.plugins.clone(),
            });

            // plugin_identifier は MarketplacePlugin.name (registry 側で validated 済み)
            let plugin_identifier = plugin_entry.name.clone();

            let mut cached = match &plugin_entry.source {
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

                    let source_path: PluginSourcePath = path.parse()?;

                    GitHubSource::with_marketplace_plugin(
                        repo,
                        self.marketplace.clone(),
                        Some(source_path.into()),
                        plugin_identifier.clone(),
                    )
                    .download(cache, force)
                    .await?
                }
                MpPluginSource::External { repo: repo_url, .. } => {
                    let repo = repo::from_url(repo_url)?;
                    GitHubSource::with_marketplace_plugin(
                        repo,
                        self.marketplace.clone(),
                        None,
                        plugin_identifier.clone(),
                    )
                    .download(cache, force)
                    .await?
                }
            };

            cached.marketplace_manifest = marketplace_manifest;
            Ok(cached)
        })
    }
}
