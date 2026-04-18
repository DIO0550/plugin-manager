mod cache;
mod cached_package;
mod manifest;
mod manifest_resolve;
mod marketplace_content;
pub mod meta;
mod plugin_content;
mod update;
mod version;

pub use cache::{CachedPackage, PackageCache, PackageCacheAccess};
pub use manifest::{Author, PluginManifest};

/// cache_key フォールバック: cache_key が None なら name を返す
pub(crate) fn resolve_cache_key<'a>(cache_key: Option<&'a str>, name: &'a str) -> &'a str {
    cache_key.unwrap_or(name)
}
pub use marketplace_content::MarketplaceContent;
pub use meta::PluginMeta;
pub(crate) use plugin_content::Plugin;
pub use update::{update_all_plugins, update_plugin, UpdateResult, UpdateStatus};
pub use version::{fetch_remote_versions, UpgradeState};
