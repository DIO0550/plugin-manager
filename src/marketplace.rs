mod config;
mod fetcher;
mod plugin_source_path;
mod registry;
mod windows_path;

pub use config::{
    normalize_name, normalize_source_path, to_display_source, to_internal_source,
    MarketplaceConfig, MarketplaceRegistration,
};
// Re-exported for tests
#[cfg(test)]
pub use config::validate_name;
pub use fetcher::MarketplaceFetcher;
pub use plugin_source_path::PluginSourcePath;
#[cfg(test)]
pub use registry::MarketplacePlugin;
pub use registry::{MarketplaceCache, MarketplaceManifest, MarketplaceRegistry, PluginSource};
