mod cache;
mod manifest;

pub use cache::{has_manifest, resolve_manifest_path, CachedPlugin, PluginCache};
pub use manifest::{Author, PluginManifest};
