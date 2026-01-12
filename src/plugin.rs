mod cache;
mod cached_plugin;
mod manifest;
mod manifest_resolve;
pub mod meta;

pub use cache::{has_manifest, CachedPlugin, PluginCache};
pub use manifest::{Author, PluginManifest};
pub use meta::PluginMeta;
