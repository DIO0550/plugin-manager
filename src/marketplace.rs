mod config;
mod download;
mod path;
mod reference;
mod registry;
mod source_ref;

pub use config::{
    normalize_name, normalize_source_path, MarketplaceConfig, MarketplaceRegistration,
};
pub use reference::{MarketplaceRef, DEFAULT_MARKETPLACE};
// Re-exported for tests
#[cfg(test)]
pub use config::validate_name;
pub use download::download_marketplace_plugin_with_cache;
pub use path::PluginSourcePath;
pub use registry::{
    validate_plugin_names, MarketplaceCache, MarketplaceManifest, MarketplaceRegistry, PluginSource,
};
pub use source_ref::MarketplaceSourceRef;
// Re-exported for tests
#[cfg(test)]
pub use registry::MarketplacePlugin;
