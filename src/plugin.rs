mod cache;
mod cached_plugin;
mod manifest;
mod manifest_resolve;
pub mod meta;
mod update;
mod version;

pub use cache::{has_manifest, CachedPlugin, PluginCache};
pub use manifest::PluginManifest;
pub use meta::PluginMeta;
pub use update::{update_all_plugins, update_plugin, UpdateResult, UpdateStatus};
pub use version::{fetch_remote_versions, needs_update, VersionQueryResult};
