use super::*;

#[test]
fn test_parse_minimal() {
    let json = r#"{"name": "test-plugin", "version": "1.0.0"}"#;
    let manifest = PluginManifest::parse(json).unwrap();
    assert_eq!(manifest.name, "test-plugin");
    assert_eq!(manifest.version, "1.0.0");
    assert!(manifest.description.is_none());
}

#[test]
fn test_parse_full() {
    let json = r#"{
        "name": "full-plugin",
        "version": "2.0.0",
        "description": "A full plugin",
        "author": {
            "name": "Test Author",
            "email": "test@example.com"
        },
        "skills": "./skills/",
        "agents": "./agents/"
    }"#;
    let manifest = PluginManifest::parse(json).unwrap();
    assert_eq!(manifest.name, "full-plugin");
    assert_eq!(manifest.version, "2.0.0");
    assert_eq!(manifest.description, Some("A full plugin".to_string()));
    assert!(manifest.author.is_some());
    assert!(manifest.has_skills());
    assert!(manifest.has_agents());
}

#[test]
fn test_parse_invalid() {
    let json = r#"{"name": "test"}"#; // missing version
    assert!(PluginManifest::parse(json).is_err());
}

// === 境界値テスト: 必須フィールド ===

#[test]
fn test_parse_missing_name() {
    // name 欠落
    let json = r#"{"version": "1.0.0"}"#;
    assert!(PluginManifest::parse(json).is_err());
}

#[test]
fn test_parse_missing_version() {
    // version 欠落
    let json = r#"{"name": "test"}"#;
    assert!(PluginManifest::parse(json).is_err());
}

#[test]
fn test_parse_name_wrong_type() {
    // name が数値
    let json = r#"{"name": 123, "version": "1.0.0"}"#;
    assert!(PluginManifest::parse(json).is_err());
}

#[test]
fn test_parse_version_wrong_type() {
    // version が数値
    let json = r#"{"name": "test", "version": 100}"#;
    assert!(PluginManifest::parse(json).is_err());
}

#[test]
fn test_parse_empty_name() {
    // 空文字の name（パースは成功するが意味的には無効）
    let json = r#"{"name": "", "version": "1.0.0"}"#;
    let manifest = PluginManifest::parse(json).unwrap();
    assert_eq!(manifest.name, "");
}

#[test]
fn test_parse_empty_version() {
    // 空文字の version
    let json = r#"{"name": "test", "version": ""}"#;
    let manifest = PluginManifest::parse(json).unwrap();
    assert_eq!(manifest.version, "");
}

// === 境界値テスト: 空文字パス ===

#[test]
fn test_parse_empty_skills_path() {
    // skills: "" の場合
    let json = r#"{"name": "test", "version": "1.0.0", "skills": ""}"#;
    let manifest = PluginManifest::parse(json).unwrap();
    assert!(manifest.has_skills());
    assert_eq!(manifest.skills, Some("".to_string()));
}

#[test]
fn test_parse_empty_agents_path() {
    // agents: "" の場合
    let json = r#"{"name": "test", "version": "1.0.0", "agents": ""}"#;
    let manifest = PluginManifest::parse(json).unwrap();
    assert!(manifest.has_agents());
}

// === 境界値テスト: パス解決 ===

#[test]
fn test_skills_dir_default() {
    // デフォルトパス
    let json = r#"{"name": "test", "version": "1.0.0"}"#;
    let manifest = PluginManifest::parse(json).unwrap();
    let base = Path::new("/plugin");
    assert_eq!(manifest.skills_dir(base), Path::new("/plugin/skills"));
}

#[test]
fn test_skills_dir_custom() {
    // カスタムパス
    let json = r#"{"name": "test", "version": "1.0.0", "skills": "my-skills"}"#;
    let manifest = PluginManifest::parse(json).unwrap();
    let base = Path::new("/plugin");
    assert_eq!(manifest.skills_dir(base), Path::new("/plugin/my-skills"));
}

#[test]
fn test_skills_dir_empty_uses_empty_path() {
    // 空文字の場合は空パスとして扱われる
    let json = r#"{"name": "test", "version": "1.0.0", "skills": ""}"#;
    let manifest = PluginManifest::parse(json).unwrap();
    let base = Path::new("/plugin");
    // 空文字は join で base そのものになる
    assert_eq!(manifest.skills_dir(base), Path::new("/plugin/"));
}

#[test]
fn test_skills_dir_with_dot_slash() {
    // ./ プレフィックス
    let json = r#"{"name": "test", "version": "1.0.0", "skills": "./skills"}"#;
    let manifest = PluginManifest::parse(json).unwrap();
    let base = Path::new("/plugin");
    assert_eq!(manifest.skills_dir(base), Path::new("/plugin/./skills"));
}

#[test]
fn test_skills_dir_absolute_path() {
    // 絶対パス指定
    let json = r#"{"name": "test", "version": "1.0.0", "skills": "/absolute/path"}"#;
    let manifest = PluginManifest::parse(json).unwrap();
    let base = Path::new("/plugin");
    // 絶対パスは base を置換する（Path::join の仕様）
    assert_eq!(manifest.skills_dir(base), Path::new("/absolute/path"));
}

#[test]
fn test_agents_dir_default() {
    let json = r#"{"name": "test", "version": "1.0.0"}"#;
    let manifest = PluginManifest::parse(json).unwrap();
    let base = Path::new("/plugin");
    assert_eq!(manifest.agents_dir(base), Path::new("/plugin/agents"));
}

#[test]
fn test_instructions_path_default() {
    let json = r#"{"name": "test", "version": "1.0.0"}"#;
    let manifest = PluginManifest::parse(json).unwrap();
    let base = Path::new("/plugin");
    assert_eq!(
        manifest.instructions_path(base),
        Path::new("/plugin/instructions.md")
    );
}

#[test]
fn test_instructions_dir_default() {
    let json = r#"{"name": "test", "version": "1.0.0"}"#;
    let manifest = PluginManifest::parse(json).unwrap();
    let base = Path::new("/plugin");
    assert_eq!(
        manifest.instructions_dir(base),
        Path::new("/plugin/instructions")
    );
}

#[test]
fn test_hooks_dir_default() {
    let json = r#"{"name": "test", "version": "1.0.0"}"#;
    let manifest = PluginManifest::parse(json).unwrap();
    let base = Path::new("/plugin");
    assert_eq!(manifest.hooks_dir(base), Path::new("/plugin/hooks"));
}

#[test]
fn test_commands_dir_default() {
    let json = r#"{"name": "test", "version": "1.0.0"}"#;
    let manifest = PluginManifest::parse(json).unwrap();
    let base = Path::new("/plugin");
    assert_eq!(manifest.commands_dir(base), Path::new("/plugin/commands"));
}

// === 境界値テスト: JSON不正 ===

#[test]
fn test_parse_invalid_json() {
    // 不正なJSON
    let json = r#"{"name": "test", version: "1.0.0"}"#;
    assert!(PluginManifest::parse(json).is_err());
}

#[test]
fn test_parse_empty_json() {
    // 空のJSON
    let json = "{}";
    assert!(PluginManifest::parse(json).is_err());
}

#[test]
fn test_parse_empty_string() {
    // 空文字
    let json = "";
    assert!(PluginManifest::parse(json).is_err());
}

#[test]
fn test_parse_null_fields() {
    // null 値
    let json = r#"{"name": "test", "version": "1.0.0", "skills": null}"#;
    let manifest = PluginManifest::parse(json).unwrap();
    assert!(!manifest.has_skills());
}

// === installed_at フィールドのテスト ===

#[test]
fn test_parse_with_installed_at() {
    let json = r#"{
        "name": "test",
        "version": "1.0.0",
        "installedAt": "2025-01-15T10:30:00Z"
    }"#;
    let manifest = PluginManifest::parse(json).unwrap();
    assert_eq!(
        manifest.installed_at,
        Some("2025-01-15T10:30:00Z".to_string())
    );
}

#[test]
fn test_parse_without_installed_at() {
    let json = r#"{"name": "test", "version": "1.0.0"}"#;
    let manifest = PluginManifest::parse(json).unwrap();
    assert!(manifest.installed_at.is_none());
}

#[test]
fn test_parse_installed_at_null() {
    let json = r#"{"name": "test", "version": "1.0.0", "installedAt": null}"#;
    let manifest = PluginManifest::parse(json).unwrap();
    assert!(manifest.installed_at.is_none());
}
