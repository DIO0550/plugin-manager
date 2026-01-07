mod cache;
mod cached_plugin;
mod manifest;
mod manifest_resolve;

pub use cache::{has_manifest, CachedPlugin, PluginCache};
pub use manifest::{Author, PluginManifest};
