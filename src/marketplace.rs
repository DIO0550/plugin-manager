mod config;
mod fetcher;
mod plugin_source_path;
mod registry;
mod windows_path;

pub use config::{
    normalize_name, normalize_source_path, to_display_source, to_internal_source, validate_name,
    MarketplaceConfig, MarketplaceEntry,
};
pub use fetcher::MarketplaceFetcher;
pub use plugin_source_path::PluginSourcePath;
pub use registry::{MarketplaceCache, MarketplaceManifest, MarketplaceRegistry, PluginSource};
