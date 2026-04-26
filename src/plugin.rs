mod action;
mod cache;
mod cached_package;
mod cleanup;
mod installed;
mod intent;
mod loader;
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
pub(crate) use cleanup::{cleanup_legacy_hierarchy, cleanup_plugin_directories};
pub use installed::InstalledPlugin;
pub use intent::PluginIntent;
pub(crate) use loader::load_plugin;
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
