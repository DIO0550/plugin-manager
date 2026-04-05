use std::path::Path;

use chrono::Utc;
use serial_test::serial;
use tempfile::TempDir;

use crate::error::PlmError;
use crate::marketplace::{MarketplaceCache, MarketplaceRegistry};
use crate::plugin::PackageCache;

use super::download_marketplace_plugin_with_cache;

/// HOME 環境変数の保存・復元ガード
struct HomeGuard(Option<String>);

impl HomeGuard {
    fn new(new_home: &Path) -> Self {
        let old = std::env::var("HOME").ok();
        std::env::set_var("HOME", new_home);
        Self(old)
    }
}

impl Drop for HomeGuard {
    fn drop(&mut self) {
        match &self.0 {
            Some(old) => std::env::set_var("HOME", old),
            None => std::env::remove_var("HOME"),
        }
    }
}

#[tokio::test]
#[serial]
async fn test_download_marketplace_plugin_with_cache_invalid_marketplace() {
    let temp_dir = TempDir::new().unwrap();
    let _guard = HomeGuard::new(temp_dir.path());

    let cache = PackageCache::with_cache_dir(temp_dir.path().join("plugins")).unwrap();

    let result = download_marketplace_plugin_with_cache(
        "some-plugin",
        "nonexistent-marketplace",
        false,
        &cache,
    )
    .await;

    assert!(
        matches!(result, Err(PlmError::MarketplaceNotFound(_))),
        "Expected MarketplaceNotFound but got: {:?}",
        result
    );
}

#[tokio::test]
#[serial]
async fn test_download_marketplace_plugin_with_cache_plugin_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let _guard = HomeGuard::new(temp_dir.path());

    let registry = MarketplaceRegistry::new().unwrap();
    registry
        .store(&MarketplaceCache {
            name: "test-marketplace".to_string(),
            fetched_at: Utc::now(),
            source: "github:test/repo".to_string(),
            owner: None,
            plugins: vec![],
        })
        .unwrap();

    let cache = PackageCache::with_cache_dir(temp_dir.path().join("plugins")).unwrap();

    let result = download_marketplace_plugin_with_cache(
        "nonexistent-plugin",
        "test-marketplace",
        false,
        &cache,
    )
    .await;

    assert!(
        matches!(result, Err(PlmError::PluginNotFound(_))),
        "Expected PluginNotFound but got: {:?}",
        result
    );
}
