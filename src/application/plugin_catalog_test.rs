use super::*;
use crate::plugin::PackageCache;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn create_test_cache() -> (TempDir, PackageCache) {
    let temp_dir = TempDir::new().unwrap();
    let cache = PackageCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();
    (temp_dir, cache)
}

/// tempdir cache 内にプラグインフィクスチャを作成
fn setup_plugin_fixture(cache_dir: &Path, marketplace: &str, name: &str, version: &str) {
    let plugin_dir = cache_dir.join(marketplace).join(name);
    fs::create_dir_all(&plugin_dir).unwrap();

    let manifest = format!(r#"{{"name":"{}","version":"{}"}}"#, name, version);
    fs::write(plugin_dir.join("plugin.json"), manifest).unwrap();
}

// ========================================
// list_installed_plugins tests (cache-based)
// ========================================

#[test]
fn test_list_installed_plugins_empty_cache() {
    let (_temp_dir, cache) = create_test_cache();
    let result = list_installed_plugins(&cache).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_list_installed_plugins_one_plugin() {
    let (temp_dir, cache) = create_test_cache();
    setup_plugin_fixture(temp_dir.path(), "github", "my-plugin", "1.0.0");

    let result = list_installed_plugins(&cache).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name(), "my-plugin");
    assert_eq!(result[0].version(), "1.0.0");
    assert!(result[0].marketplace().is_none());
}

#[test]
fn test_list_installed_plugins_multiple() {
    let (temp_dir, cache) = create_test_cache();
    setup_plugin_fixture(temp_dir.path(), "github", "plugin-a", "1.0.0");
    setup_plugin_fixture(temp_dir.path(), "github", "plugin-b", "2.0.0");
    setup_plugin_fixture(temp_dir.path(), "my-marketplace", "plugin-c", "3.0.0");

    let result = list_installed_plugins(&cache).unwrap();
    assert_eq!(result.len(), 3);
}

#[test]
fn test_list_installed_plugins_hidden_dir_excluded() {
    let (temp_dir, cache) = create_test_cache();
    setup_plugin_fixture(temp_dir.path(), "github", "good-plugin", "1.0.0");
    setup_plugin_fixture(temp_dir.path(), "github", ".hidden-plugin", "1.0.0");

    let result = list_installed_plugins(&cache).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name(), "good-plugin");
}

#[test]
fn test_list_installed_plugins_no_manifest_excluded() {
    let (temp_dir, cache) = create_test_cache();
    setup_plugin_fixture(temp_dir.path(), "github", "valid-plugin", "1.0.0");
    let no_manifest_dir = temp_dir.path().join("github").join("no-manifest");
    fs::create_dir_all(&no_manifest_dir).unwrap();

    let result = list_installed_plugins(&cache).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name(), "valid-plugin");
}
