mod cache;
mod content;
mod lifecycle;
pub mod meta;

pub(crate) use cache::{cleanup_legacy_hierarchy, cleanup_plugin_directories, list_installed};
pub use cache::{CachedPackage, PackageCache, PackageCacheAccess};
pub(crate) use content::{load_plugin, Plugin};
pub use content::{InstalledPlugin, MarketplaceContent};
pub use lifecycle::{
    update_all_plugins, update_plugin, PluginAction, PluginIntent, UpdateResult, UpdateStatus,
};
pub use meta::manifest::{Author, PluginManifest};

/// id フォールバック: id が None なら name を返す
///
/// # Arguments
///
/// * `id` - optional identifier (cache directory name)
/// * `name` - plugin name used as fallback when `id` is `None`
pub(crate) fn resolve_id<'a>(id: Option<&'a str>, name: &'a str) -> &'a str {
    id.unwrap_or(name)
}
pub(crate) use meta::version;
pub use meta::version::{fetch_remote_versions, UpgradeState};
pub use meta::PluginMeta;
