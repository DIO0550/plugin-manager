// These tests use PLM_HOME env var (not HOME) to isolate the marketplace
// registry path. PLM_HOME is PLM-specific and does not affect system-level
// HOME resolution in other tests. All tests that mutate PLM_HOME are marked
// #[serial] to prevent concurrent env var conflicts within this group.
//
// Ideally MarketplaceSource would accept an injected registry/cache dir,
// but that refactor is out of scope for this PR. PLM_HOME + #[serial] is
// the pragmatic middle ground.

use std::path::Path;

use chrono::Utc;
use serial_test::serial;
use tempfile::TempDir;

use crate::error::PlmError;
use crate::marketplace::{MarketplaceCache, MarketplaceRegistry};
use crate::plugin::PackageCache;

use super::download_marketplace_plugin_with_cache;

/// PLM_HOME 環境変数の保存・復元ガード
///
/// テスト用に PLM_HOME を一時ディレクトリに差し替え、
/// Drop 時に元の値（または未設定状態）に復元する。
/// `#[serial]` と併用して環境変数の競合を防ぐ。
struct PlmHomeGuard(Option<String>);

impl PlmHomeGuard {
    fn new(new_home: &Path) -> Self {
        let old = std::env::var("PLM_HOME").ok();
        std::env::set_var("PLM_HOME", new_home);
        Self(old)
    }
}

impl Drop for PlmHomeGuard {
    fn drop(&mut self) {
        match &self.0 {
            Some(old) => std::env::set_var("PLM_HOME", old),
            None => std::env::remove_var("PLM_HOME"),
        }
    }
}

#[tokio::test]
#[serial]
async fn test_download_marketplace_plugin_with_cache_invalid_marketplace() {
    let temp_dir = TempDir::new().unwrap();
    let _guard = PlmHomeGuard::new(temp_dir.path());

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
    let _guard = PlmHomeGuard::new(temp_dir.path());

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
