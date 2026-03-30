mod cache;
mod cached_plugin;
mod manifest;
mod manifest_resolve;
mod marketplace_package;
pub mod meta;
mod update;
mod version;

pub use cache::{has_manifest, PluginCache, PluginCacheAccess, RemoteMarketplaceData};
pub use manifest::PluginManifest;
pub use marketplace_package::MarketplacePackage;
pub use meta::PluginMeta;
pub use update::{update_all_plugins, update_plugin, UpdateResult, UpdateStatus};
pub use version::{fetch_remote_versions, needs_update, VersionQueryResult};
