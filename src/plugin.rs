mod cache;
mod manifest;

pub use cache::{has_manifest, CachedPlugin, PluginCache};
pub use manifest::{Author, PluginManifest};
