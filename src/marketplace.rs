mod config;
mod download;
mod path;
mod registry;

pub use config::{
    normalize_name, normalize_source_path, to_display_source, to_internal_source,
    MarketplaceConfig, MarketplaceRegistration,
};
// Re-exported for tests
#[cfg(test)]
pub use config::validate_name;
pub use download::download_marketplace_plugin_with_cache;
pub use path::PluginSourcePath;
pub use registry::{
    validate_plugin_name, validate_plugin_names, MarketplaceCache, MarketplaceManifest,
    MarketplacePlugin, MarketplaceRegistry, PluginSource,
};
