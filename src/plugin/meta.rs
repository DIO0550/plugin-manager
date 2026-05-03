mod manifest;
mod manifest_resolve;
#[allow(clippy::module_inception)]
mod meta;
pub(crate) mod version;

pub use self::meta::*;

pub use self::manifest::{Author, PluginManifest};
pub use self::manifest_resolve::has_manifest;
pub(crate) use self::manifest_resolve::resolve_manifest_path;
pub use self::version::{fetch_remote_versions, UpgradeState};
