use super::*;
use crate::plugin::PluginCache;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn create_test_cache() -> (TempDir, PluginCache) {
    let temp_dir = TempDir::new().unwrap();
    let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();
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
// get_uninstall_info tests (正常系)
// ========================================

#[test]
fn test_get_uninstall_info_found() {
    // キャッシュにプラグインが存在する場合は成功
    let (temp_dir, cache) = create_test_cache();
    setup_plugin_fixture(temp_dir.path(), "github", "my-plugin", "1.0.0");

    let result = get_uninstall_info(&cache, "my-plugin", Some("github"));
    assert!(result.is_ok());
    let info = result.unwrap();
    assert_eq!(info.plugin_name, "my-plugin");
}

// ========================================
// uninstall_plugin tests (正常系)
// ========================================

#[test]
fn test_uninstall_plugin_success() {
    // キャッシュにプラグインが存在する場合はアンインストール成功
    let (temp_dir, cache) = create_test_cache();
    let project_root = TempDir::new().unwrap();
    setup_plugin_fixture(temp_dir.path(), "github", "my-plugin", "1.0.0");

    let result = uninstall_plugin(&cache, "my-plugin", Some("github"), project_root.path());
    assert!(result.success);
    assert!(result.error.is_none());
}

// ========================================
// disable_plugin tests (正常系)
// ========================================

#[test]
fn test_disable_plugin_success() {
    // キャッシュにプラグインが存在する場合は disable 成功
    let (temp_dir, cache) = create_test_cache();
    let project_root = TempDir::new().unwrap();
    setup_plugin_fixture(temp_dir.path(), "github", "my-plugin", "1.0.0");

    let result = disable_plugin(
        &cache,
        "my-plugin",
        Some("github"),
        project_root.path(),
        None,
    );
    assert!(result.success);
    assert!(result.error.is_none());
}

// ========================================
// enable_plugin tests (正常系)
// ========================================

#[test]
fn test_enable_plugin_success() {
    // キャッシュにプラグインが存在する場合は enable 成功
    let (temp_dir, cache) = create_test_cache();
    let project_root = TempDir::new().unwrap();
    setup_plugin_fixture(temp_dir.path(), "github", "my-plugin", "1.0.0");

    let result = enable_plugin(
        &cache,
        "my-plugin",
        Some("github"),
        project_root.path(),
        None,
    );
    assert!(result.success);
    assert!(result.error.is_none());
}

// ========================================
// Error tests
// ========================================

#[test]
fn test_get_uninstall_info_not_found() {
    // 存在しないプラグインの場合はエラーを返す
    let (_temp_dir, cache) = create_test_cache();
    let result = get_uninstall_info(&cache, "nonexistent-plugin-12345", Some("github"));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("not found"));
    assert!(err.contains("nonexistent-plugin-12345"));
}

#[test]
fn test_get_uninstall_info_default_marketplace() {
    // マーケットプレイス未指定時は "github" がデフォルト
    let (_temp_dir, cache) = create_test_cache();
    let result = get_uninstall_info(&cache, "nonexistent-plugin-12345", None);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("github"));
}

#[test]
fn test_uninstall_plugin_not_found() {
    // 存在しないプラグインのアンインストールはエラー
    let (_temp_dir, cache) = create_test_cache();
    let temp_dir = std::env::temp_dir();
    let result = uninstall_plugin(
        &cache,
        "nonexistent-plugin-12345",
        Some("github"),
        &temp_dir,
    );
    assert!(!result.success);
    assert!(result.error.is_some());
    assert!(result.error.unwrap().contains("not found"));
}

#[test]
fn test_disable_plugin_not_found() {
    // 存在しないプラグインのdisableはエラー
    let (_temp_dir, cache) = create_test_cache();
    let temp_dir = std::env::temp_dir();
    let result = disable_plugin(
        &cache,
        "nonexistent-plugin-12345",
        Some("github"),
        &temp_dir,
        None,
    );
    assert!(!result.success);
    assert!(result.error.is_some());
    assert!(result.error.unwrap().contains("not found"));
}

#[test]
fn test_enable_plugin_not_found() {
    // 存在しないプラグインのenableはエラー
    let (_temp_dir, cache) = create_test_cache();
    let temp_dir = std::env::temp_dir();
    let result = enable_plugin(
        &cache,
        "nonexistent-plugin-12345",
        Some("github"),
        &temp_dir,
        None,
    );
    assert!(!result.success);
    assert!(result.error.is_some());
    assert!(result.error.unwrap().contains("not found"));
}
