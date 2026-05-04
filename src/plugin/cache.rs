#[allow(clippy::module_inception)]
mod cache;
mod cached_package;
mod cleanup;
mod legacy_layout_sweeper;

pub(crate) use cache::list_installed;
pub use cache::{PackageCache, PackageCacheAccess};
pub use cached_package::CachedPackage;
pub(crate) use cleanup::{cleanup_legacy_hierarchy, cleanup_plugin_directories};
pub use legacy_layout_sweeper::LegacyLayoutSweeper;
