use chrono::Utc;
use tempfile::TempDir;

use crate::error::PlmError;
use crate::marketplace::{MarketplaceCache, MarketplaceRegistry};
use crate::plugin::PackageCache;

use super::download_marketplace_plugin_with_registry;

#[tokio::test]
async fn test_download_with_registry_marketplace_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let registry = MarketplaceRegistry::with_cache_dir(temp_dir.path().join("mkt")).unwrap();
    let cache = PackageCache::with_cache_dir(temp_dir.path().join("plugins")).unwrap();

    let result = download_marketplace_plugin_with_registry(
        "some-plugin",
        "nonexistent-marketplace",
        false,
        &cache,
        &registry,
    )
    .await;

    assert!(
        matches!(result, Err(PlmError::MarketplaceNotFound(_))),
        "Expected MarketplaceNotFound but got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_download_with_registry_plugin_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let registry = MarketplaceRegistry::with_cache_dir(temp_dir.path().join("mkt")).unwrap();
    registry
        .store(&MarketplaceCache {
            name: "test-marketplace".to_string(),
            fetched_at: Utc::now(),
            source: "github:test/repo".to_string(),
            owner: None,
            plugins: vec![],
            original_manifest: None,
        })
        .unwrap();

    let cache = PackageCache::with_cache_dir(temp_dir.path().join("plugins")).unwrap();

    let result = download_marketplace_plugin_with_registry(
        "nonexistent-plugin",
        "test-marketplace",
        false,
        &cache,
        &registry,
    )
    .await;

    assert!(
        matches!(result, Err(PlmError::PluginNotFound(_))),
        "Expected PluginNotFound but got: {:?}",
        result
    );
}
