use super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_normalize_installed_at_valid() {
    assert_eq!(
        normalize_installed_at(Some("2025-01-15T10:30:00Z")),
        Some("2025-01-15T10:30:00Z".to_string())
    );
}

#[test]
fn test_normalize_installed_at_empty() {
    assert_eq!(normalize_installed_at(Some("")), None);
}

#[test]
fn test_normalize_installed_at_whitespace() {
    assert_eq!(normalize_installed_at(Some("   ")), None);
}

#[test]
fn test_normalize_installed_at_trimmed() {
    assert_eq!(
        normalize_installed_at(Some("  2025-01-15T10:30:00Z  ")),
        Some("2025-01-15T10:30:00Z".to_string())
    );
}

#[test]
fn test_normalize_installed_at_none() {
    assert_eq!(normalize_installed_at(None), None);
}

#[test]
fn test_write_and_load_meta() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path();

    let meta = PluginMeta {
        installed_at: Some("2025-01-15T10:30:00Z".to_string()),
    };

    write_meta(plugin_dir, &meta).unwrap();

    let loaded = load_meta(plugin_dir).unwrap();
    assert_eq!(loaded.installed_at, Some("2025-01-15T10:30:00Z".to_string()));
}

#[test]
fn test_write_installed_at() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path();

    write_installed_at(plugin_dir).unwrap();

    let loaded = load_meta(plugin_dir).unwrap();
    assert!(loaded.installed_at.is_some());
    let installed_at = loaded.installed_at.unwrap();
    assert!(installed_at.contains("T"));
    assert!(installed_at.ends_with("Z"));
}

#[test]
fn test_load_meta_not_exists() {
    let temp_dir = TempDir::new().unwrap();
    let loaded = load_meta(temp_dir.path());
    assert!(loaded.is_none());
}

#[test]
fn test_load_meta_corrupted() {
    let temp_dir = TempDir::new().unwrap();
    let meta_path = temp_dir.path().join(META_FILE);

    // 破損したJSONを書き込む
    fs::write(&meta_path, "{ invalid json }").unwrap();

    let loaded = load_meta(temp_dir.path());
    assert!(loaded.is_none());
}

#[test]
fn test_plugin_meta_serde() {
    let meta = PluginMeta {
        installed_at: Some("2025-01-15T10:30:00Z".to_string()),
    };

    let json = serde_json::to_string(&meta).unwrap();
    assert!(json.contains("installedAt"));

    let parsed: PluginMeta = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.installed_at, meta.installed_at);
}

#[test]
fn test_plugin_meta_default() {
    let meta = PluginMeta::default();
    assert!(meta.installed_at.is_none());
}

#[test]
fn test_resolve_installed_at_from_meta() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path();

    // .plm-meta.json を作成
    let meta = PluginMeta {
        installed_at: Some("2025-01-15T10:30:00Z".to_string()),
    };
    write_meta(plugin_dir, &meta).unwrap();

    let result = resolve_installed_at(plugin_dir, None);
    assert_eq!(result, Some("2025-01-15T10:30:00Z".to_string()));
}

#[test]
fn test_resolve_installed_at_fallback_to_manifest() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path();

    // plugin.json を作成（.plm-meta.json なし）
    let manifest_content = r#"{"name":"test","version":"1.0.0","installedAt":"2025-01-10T00:00:00Z"}"#;
    fs::write(plugin_dir.join("plugin.json"), manifest_content).unwrap();

    let manifest = PluginManifest::parse(manifest_content).unwrap();
    let result = resolve_installed_at(plugin_dir, Some(&manifest));
    assert_eq!(result, Some("2025-01-10T00:00:00Z".to_string()));
}

#[test]
fn test_resolve_installed_at_meta_priority() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path();

    // 両方に値がある場合、.plm-meta.json が優先
    let meta = PluginMeta {
        installed_at: Some("2025-01-15T10:30:00Z".to_string()),
    };
    write_meta(plugin_dir, &meta).unwrap();

    let manifest_content = r#"{"name":"test","version":"1.0.0","installedAt":"2025-01-10T00:00:00Z"}"#;
    fs::write(plugin_dir.join("plugin.json"), manifest_content).unwrap();

    let manifest = PluginManifest::parse(manifest_content).unwrap();
    let result = resolve_installed_at(plugin_dir, Some(&manifest));
    assert_eq!(result, Some("2025-01-15T10:30:00Z".to_string()));
}

#[test]
fn test_resolve_installed_at_empty_meta_fallback() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path();

    // .plm-meta.json は空の installedAt
    let meta = PluginMeta {
        installed_at: Some("".to_string()),
    };
    write_meta(plugin_dir, &meta).unwrap();

    // plugin.json に値あり
    let manifest_content = r#"{"name":"test","version":"1.0.0","installedAt":"2025-01-10T00:00:00Z"}"#;
    fs::write(plugin_dir.join("plugin.json"), manifest_content).unwrap();

    let manifest = PluginManifest::parse(manifest_content).unwrap();
    let result = resolve_installed_at(plugin_dir, Some(&manifest));
    assert_eq!(result, Some("2025-01-10T00:00:00Z".to_string()));
}

#[test]
fn test_resolve_installed_at_both_none() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path();

    // plugin.json に installedAt なし
    let manifest_content = r#"{"name":"test","version":"1.0.0"}"#;
    fs::write(plugin_dir.join("plugin.json"), manifest_content).unwrap();

    let manifest = PluginManifest::parse(manifest_content).unwrap();
    let result = resolve_installed_at(plugin_dir, Some(&manifest));
    assert!(result.is_none());
}
