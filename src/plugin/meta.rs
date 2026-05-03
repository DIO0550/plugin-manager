pub(crate) mod manifest;
pub(crate) mod manifest_resolve;
#[allow(clippy::module_inception)]
mod meta;
pub(crate) mod version;

pub use self::meta::*;

pub(crate) use self::manifest_resolve::{has_manifest, resolve_manifest_path};
