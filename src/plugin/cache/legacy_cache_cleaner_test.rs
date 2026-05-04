use super::LegacyCacheCleaner;
use crate::marketplace::{MarketplaceCache, MarketplacePlugin, PluginSource};
use crate::plugin::cache::{PackageCache, PackageCacheAccess};
use chrono::Utc;
use std::fs;
use tempfile::TempDir;

fn make_cache(plugins: Vec<&str>) -> MarketplaceCache {
    MarketplaceCache {
        name: "d-market".to_string(),
        fetched_at: Utc::now(),
        source: "github:owner/d-market-git".to_string(),
        owner: None,
        plugins: plugins
            .into_iter()
            .map(|n| MarketplacePlugin {
                name: n.to_string(),
                source: PluginSource::Local(format!("./plugins/{}", n)),
                description: None,
                version: None,
            })
            .collect(),
    }
}

#[test]
fn test_legacy_cleaner_removes_only_repo_name_dir() {
    let temp = TempDir::new().unwrap();
    let cache = PackageCache::with_cache_dir(temp.path().to_path_buf()).unwrap();
    let market = "d-market";

    fs::create_dir_all(temp.path().join(market).join("d-market-git")).unwrap();
    fs::create_dir_all(temp.path().join(market).join("plugin-a")).unwrap();
    fs::create_dir_all(temp.path().join(market).join("plugin-b")).unwrap();

    let mp_cache = make_cache(vec!["plugin-a", "plugin-b"]);

    let removed = LegacyCacheCleaner::clean_if_legacy(&cache, market, &mp_cache).unwrap();
    assert!(removed);
    assert!(!cache.has_marketplace_entry(market, "d-market-git").unwrap());
    assert!(cache.has_marketplace_entry(market, "plugin-a").unwrap());
    assert!(cache.has_marketplace_entry(market, "plugin-b").unwrap());
}

#[test]
fn test_legacy_cleaner_noop_when_plugins_len_le_1() {
    let temp = TempDir::new().unwrap();
    let cache = PackageCache::with_cache_dir(temp.path().to_path_buf()).unwrap();
    let market = "d-market";

    fs::create_dir_all(temp.path().join(market).join("d-market-git")).unwrap();
    let mp_cache = make_cache(vec!["plugin-a"]);

    let removed = LegacyCacheCleaner::clean_if_legacy(&cache, market, &mp_cache).unwrap();
    assert!(!removed);
    assert!(cache.has_marketplace_entry(market, "d-market-git").unwrap());
}

#[test]
fn test_legacy_cleaner_noop_when_plugins_empty() {
    let temp = TempDir::new().unwrap();
    let cache = PackageCache::with_cache_dir(temp.path().to_path_buf()).unwrap();
    let market = "d-market";

    fs::create_dir_all(temp.path().join(market).join("d-market-git")).unwrap();
    let mp_cache = make_cache(vec![]);

    let removed = LegacyCacheCleaner::clean_if_legacy(&cache, market, &mp_cache).unwrap();
    assert!(!removed);
    assert!(cache.has_marketplace_entry(market, "d-market-git").unwrap());
}

#[test]
fn test_legacy_cleaner_noop_when_repo_name_matches_plugin() {
    let temp = TempDir::new().unwrap();
    let cache = PackageCache::with_cache_dir(temp.path().to_path_buf()).unwrap();
    let market = "d-market";

    fs::create_dir_all(temp.path().join(market).join("d-market-git")).unwrap();
    let mp_cache = make_cache(vec!["d-market-git", "plugin-a"]);

    let removed = LegacyCacheCleaner::clean_if_legacy(&cache, market, &mp_cache).unwrap();
    assert!(!removed);
    assert!(cache.has_marketplace_entry(market, "d-market-git").unwrap());
}

#[test]
fn test_legacy_cleaner_noop_when_legacy_dir_missing() {
    let temp = TempDir::new().unwrap();
    let cache = PackageCache::with_cache_dir(temp.path().to_path_buf()).unwrap();
    let market = "d-market";

    fs::create_dir_all(temp.path().join(market).join("plugin-a")).unwrap();
    let mp_cache = make_cache(vec!["plugin-a", "plugin-b"]);

    let removed = LegacyCacheCleaner::clean_if_legacy(&cache, market, &mp_cache).unwrap();
    assert!(!removed);
    assert!(cache.has_marketplace_entry(market, "plugin-a").unwrap());
}

#[test]
fn test_remove_marketplace_entry_removes_only_specified() {
    let temp = TempDir::new().unwrap();
    let cache = PackageCache::with_cache_dir(temp.path().to_path_buf()).unwrap();
    let market = "d-market";

    fs::create_dir_all(temp.path().join(market).join("a")).unwrap();
    fs::create_dir_all(temp.path().join(market).join("b")).unwrap();
    fs::create_dir_all(temp.path().join(market).join("c")).unwrap();

    cache.remove_marketplace_entry(market, "b").unwrap();

    assert!(cache.has_marketplace_entry(market, "a").unwrap());
    assert!(!cache.has_marketplace_entry(market, "b").unwrap());
    assert!(cache.has_marketplace_entry(market, "c").unwrap());
}
