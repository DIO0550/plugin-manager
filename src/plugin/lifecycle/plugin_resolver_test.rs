use super::find_by_plugin_name;
use crate::error::PlmError;
use crate::plugin::cache::{PackageCache, PackageCacheAccess};
use crate::plugin::meta::{write_meta, PluginMeta};
use std::fs;
use tempfile::TempDir;

const SAMPLE_MANIFEST: &str = r#"{ "name": "plugin-a", "version": "0.0.1", "description": "" }"#;
const SAMPLE_MANIFEST_B: &str = r#"{ "name": "plugin-b", "version": "0.0.1", "description": "" }"#;

fn install_plugin(cache: &PackageCache, marketplace: Option<&str>, cache_id: &str, manifest: &str) {
    let path = cache.plugin_path(marketplace, cache_id);
    fs::create_dir_all(&path).unwrap();
    fs::write(path.join("plugin.json"), manifest).unwrap();
    write_meta(&path, &PluginMeta::default()).unwrap();
}

#[test]
fn test_find_by_plugin_name_returns_none_when_not_found() {
    let temp = TempDir::new().unwrap();
    let cache = PackageCache::with_cache_dir(temp.path().to_path_buf()).unwrap();
    install_plugin(&cache, Some("d-market"), "plugin-a", SAMPLE_MANIFEST);

    let r = find_by_plugin_name(&cache, "plugin-x", None).unwrap();
    assert!(r.is_none());
}

#[test]
fn test_find_by_plugin_name_returns_single_match() {
    let temp = TempDir::new().unwrap();
    let cache = PackageCache::with_cache_dir(temp.path().to_path_buf()).unwrap();
    install_plugin(&cache, Some("d-market"), "plugin-a", SAMPLE_MANIFEST);

    let r = find_by_plugin_name(&cache, "plugin-a", None)
        .unwrap()
        .unwrap();
    assert_eq!(r.display_name, "plugin-a");
    assert_eq!(r.cache_id, "plugin-a");
    assert_eq!(r.marketplace.as_deref(), Some("d-market"));
}

#[test]
fn test_find_by_plugin_name_returns_ambiguous_error_for_multiple_matches() {
    let temp = TempDir::new().unwrap();
    let cache = PackageCache::with_cache_dir(temp.path().to_path_buf()).unwrap();
    install_plugin(&cache, Some("d-market"), "plugin-a", SAMPLE_MANIFEST);
    install_plugin(&cache, Some("x-market"), "plugin-a", SAMPLE_MANIFEST);

    let err = find_by_plugin_name(&cache, "plugin-a", None).unwrap_err();
    match err {
        PlmError::AmbiguousPluginName { name, candidates } => {
            assert_eq!(name, "plugin-a");
            assert_eq!(candidates.len(), 2);
            // sorted
            assert_eq!(candidates[0], "plugin-a@d-market");
            assert_eq!(candidates[1], "plugin-a@x-market");
        }
        other => panic!("unexpected error: {:?}", other),
    }
}

#[test]
fn test_find_by_plugin_name_disambiguates_with_marketplace_hint() {
    let temp = TempDir::new().unwrap();
    let cache = PackageCache::with_cache_dir(temp.path().to_path_buf()).unwrap();
    install_plugin(&cache, Some("d-market"), "plugin-a", SAMPLE_MANIFEST);
    install_plugin(&cache, Some("x-market"), "plugin-a", SAMPLE_MANIFEST);

    let r = find_by_plugin_name(&cache, "plugin-a", Some("d-market"))
        .unwrap()
        .unwrap();
    assert_eq!(r.marketplace.as_deref(), Some("d-market"));
}

#[test]
fn test_find_by_plugin_name_returns_none_when_hint_does_not_match_any() {
    let temp = TempDir::new().unwrap();
    let cache = PackageCache::with_cache_dir(temp.path().to_path_buf()).unwrap();
    install_plugin(&cache, Some("d-market"), "plugin-a", SAMPLE_MANIFEST);

    let r = find_by_plugin_name(&cache, "plugin-a", Some("x-market")).unwrap();
    assert!(r.is_none());
}

#[test]
fn test_find_by_plugin_name_skips_corrupt_entries() {
    let temp = TempDir::new().unwrap();
    let cache = PackageCache::with_cache_dir(temp.path().to_path_buf()).unwrap();
    install_plugin(&cache, Some("d-market"), "plugin-a", SAMPLE_MANIFEST);
    // 壊れたエントリ（plugin.json なし）
    let broken = cache.plugin_path(Some("d-market"), "broken");
    fs::create_dir_all(&broken).unwrap();

    let r = find_by_plugin_name(&cache, "plugin-a", None)
        .unwrap()
        .unwrap();
    assert_eq!(r.cache_id, "plugin-a");
}

#[test]
fn test_find_by_plugin_name_distinguishes_direct_github() {
    let temp = TempDir::new().unwrap();
    let cache = PackageCache::with_cache_dir(temp.path().to_path_buf()).unwrap();
    install_plugin(&cache, Some("d-market"), "plugin-a", SAMPLE_MANIFEST);
    install_plugin(&cache, None, "owner--plugin-a", SAMPLE_MANIFEST_B);

    // direct GitHub plugin の display name は "plugin-b"
    let r = find_by_plugin_name(&cache, "plugin-b", None)
        .unwrap()
        .unwrap();
    assert_eq!(r.marketplace, None);
    assert_eq!(r.cache_id, "owner--plugin-a");
}
