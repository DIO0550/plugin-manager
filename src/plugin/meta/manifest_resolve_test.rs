use super::*;
use std::fs;
use tempfile::TempDir;

/// テスト用のプラグインディレクトリを作成
fn create_test_plugin_dir(temp_dir: &TempDir, structure: &[(&str, Option<&str>)]) -> PathBuf {
    let plugin_dir = temp_dir.path().to_path_buf();

    for (path, content) in structure {
        let full_path = plugin_dir.join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        if let Some(content) = content {
            fs::write(&full_path, content).unwrap();
        } else {
            fs::create_dir_all(&full_path).unwrap();
        }
    }

    plugin_dir
}

#[test]
fn test_resolve_manifest_path_prefers_claude_plugin_dir() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = create_test_plugin_dir(
        &temp_dir,
        &[
            (
                ".claude-plugin/plugin.json",
                Some(r#"{"name":"test","version":"1.0.0"}"#),
            ),
            (
                "plugin.json",
                Some(r#"{"name":"fallback","version":"0.1.0"}"#),
            ),
        ],
    );

    let manifest_path = resolve_manifest_path(&plugin_dir).unwrap();
    assert!(manifest_path.ends_with(".claude-plugin/plugin.json"));
}

#[test]
fn test_resolve_manifest_path_fallback_to_root() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = create_test_plugin_dir(
        &temp_dir,
        &[("plugin.json", Some(r#"{"name":"test","version":"1.0.0"}"#))],
    );

    let manifest_path = resolve_manifest_path(&plugin_dir).unwrap();
    assert!(manifest_path.ends_with("plugin.json"));
    assert!(!manifest_path.to_string_lossy().contains(".claude-plugin"));
}

#[test]
fn test_resolve_manifest_path_returns_none_when_missing() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    assert!(resolve_manifest_path(&plugin_dir).is_none());
}

#[test]
fn test_has_manifest_returns_true_for_claude_plugin_dir() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = create_test_plugin_dir(
        &temp_dir,
        &[(
            ".claude-plugin/plugin.json",
            Some(r#"{"name":"test","version":"1.0.0"}"#),
        )],
    );

    assert!(has_manifest(&plugin_dir));
}

#[test]
fn test_has_manifest_returns_true_for_root_plugin_json() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = create_test_plugin_dir(
        &temp_dir,
        &[("plugin.json", Some(r#"{"name":"test","version":"1.0.0"}"#))],
    );

    assert!(has_manifest(&plugin_dir));
}

#[test]
fn test_has_manifest_returns_false_when_missing() {
    let temp_dir = TempDir::new().unwrap();
    assert!(!has_manifest(temp_dir.path()));
}
