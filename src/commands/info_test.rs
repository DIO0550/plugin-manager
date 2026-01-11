use super::*;
use crate::application::{AuthorInfo, ComponentInfo, PluginSource};

fn create_test_detail() -> PluginDetail {
    PluginDetail {
        name: "test-plugin".to_string(),
        version: "1.0.0".to_string(),
        description: Some("A test plugin".to_string()),
        author: Some(AuthorInfo {
            name: "Test Author".to_string(),
            email: Some("test@example.com".to_string()),
            url: None,
        }),
        installed_at: Some("2025-01-15T10:30:00Z".to_string()),
        source: PluginSource::GitHub {
            repository: "owner/repo".to_string(),
        },
        components: ComponentInfo {
            skills: vec!["skill1".to_string(), "skill2".to_string()],
            agents: vec![],
            commands: vec!["cmd1".to_string()],
            instructions: vec![],
            hooks: vec![],
        },
        enabled: true,
        cache_path: "/home/user/.plm/cache/plugins/github/owner--repo".to_string(),
    }
}

#[test]
fn test_format_list_empty() {
    assert_eq!(format_list(&[]), "none");
}

#[test]
fn test_format_list_single() {
    assert_eq!(format_list(&["item".to_string()]), "item");
}

#[test]
fn test_format_list_multiple() {
    assert_eq!(
        format_list(&["a".to_string(), "b".to_string(), "c".to_string()]),
        "a, b, c"
    );
}

#[test]
fn test_json_serialization() {
    let detail = create_test_detail();
    let json = serde_json::to_string_pretty(&detail).unwrap();

    // 基本フィールドが含まれていることを確認
    assert!(json.contains("\"name\": \"test-plugin\""));
    assert!(json.contains("\"version\": \"1.0.0\""));
    assert!(json.contains("\"installedAt\": \"2025-01-15T10:30:00Z\""));

    // author フィールドが含まれていることを確認
    assert!(json.contains("\"author\""));
    assert!(json.contains("\"Test Author\""));

    // source のタグ付きシリアライズを確認
    assert!(json.contains("\"type\": \"github\""));
    assert!(json.contains("\"repository\": \"owner/repo\""));
}

#[test]
fn test_json_serialization_no_author() {
    let mut detail = create_test_detail();
    detail.author = None;

    let json = serde_json::to_string_pretty(&detail).unwrap();

    // author フィールドが省略されていることを確認（skip_serializing_if）
    assert!(!json.contains("\"author\""));
}

#[test]
fn test_yaml_serialization() {
    let detail = create_test_detail();
    let yaml = serde_yaml::to_string(&detail).unwrap();

    assert!(yaml.contains("name: test-plugin"));
    assert!(yaml.contains("version: 1.0.0"));
    assert!(yaml.contains("type: github"));
}
