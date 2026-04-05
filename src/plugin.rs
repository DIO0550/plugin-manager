mod cache;
mod cached_package;
mod manifest;
mod manifest_resolve;
mod marketplace_package;
pub mod meta;
mod update;
mod version;

pub use cache::{CachedPackage, PackageCache, PackageCacheAccess};
pub use cached_package::UNKNOWN_GIT_VALUE;
pub use manifest::PluginManifest;
pub use marketplace_package::MarketplacePackage;
pub use meta::PluginMeta;
pub use update::{update_all_plugins, update_plugin, UpdateResult, UpdateStatus};
pub use version::{fetch_remote_versions, needs_update, VersionQueryResult};
