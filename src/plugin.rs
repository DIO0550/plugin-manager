mod action;
mod cache;
mod cached_package;
mod deployment;
mod installed;
mod intent;
mod manifest;
mod manifest_resolve;
mod marketplace_content;
pub mod meta;
mod plugin_content;
mod update;
mod version;

pub use action::PluginAction;
pub(crate) use cache::list_installed;
pub use cache::{CachedPackage, PackageCache, PackageCacheAccess};
pub(crate) use deployment::{cleanup_plugin_directories, load_plugin_deployment};
pub use installed::InstalledPlugin;
pub use intent::PluginIntent;
pub use manifest::{Author, PluginManifest};

/// cache_key フォールバック: cache_key が None なら name を返す
///
/// # Arguments
///
/// * `cache_key` - optional cache directory key
/// * `name` - plugin name used as fallback when `cache_key` is `None`
pub(crate) fn resolve_cache_key<'a>(cache_key: Option<&'a str>, name: &'a str) -> &'a str {
    cache_key.unwrap_or(name)
}
pub use marketplace_content::MarketplaceContent;
pub use meta::PluginMeta;
pub(crate) use plugin_content::Plugin;
pub use update::{update_all_plugins, update_plugin, UpdateResult, UpdateStatus};
pub use version::{fetch_remote_versions, UpgradeState};
