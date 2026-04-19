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

/// id フォールバック: id が None なら name を返す
///
/// # Arguments
///
/// * `id` - optional identifier (cache directory name)
/// * `name` - plugin name used as fallback when `id` is `None`
pub(crate) fn resolve_id<'a>(id: Option<&'a str>, name: &'a str) -> &'a str {
    id.unwrap_or(name)
}
pub use marketplace_content::MarketplaceContent;
pub use meta::PluginMeta;
pub(crate) use plugin_content::Plugin;
pub use update::{update_all_plugins, update_plugin, UpdateResult, UpdateStatus};
pub use version::{fetch_remote_versions, UpgradeState};
