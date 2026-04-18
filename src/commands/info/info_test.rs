use super::json::render_json;
use super::table::{format_list, render_table};
use super::yaml::render_yaml;
use crate::application::{AuthorInfo, PluginDetail, PluginSource};
use crate::component::{Component, ComponentKind};
use std::path::PathBuf;

const TEST_CACHE_PATH: &str = "/home/user/.plm/cache/plugins/github/owner--repo";
const TEST_ENABLED: bool = true;

fn comp(kind: ComponentKind, name: &str) -> Component {
    Component {
        kind,
        name: name.to_string(),
        path: PathBuf::from(format!("dummy/{}", name)),
    }
}

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
        components: vec![
            comp(ComponentKind::Skill, "skill1"),
            comp(ComponentKind::Skill, "skill2"),
            comp(ComponentKind::Command, "cmd1"),
        ],
        enabled: TEST_ENABLED,
        cache_path: TEST_CACHE_PATH.to_string(),
    }
}

#[test]
fn test_format_list_empty() {
    assert_eq!(format_list(&[]), "none");
}

#[test]
fn test_format_list_single() {
    assert_eq!(format_list(&["item"]), "item");
}

#[test]
fn test_format_list_multiple() {
    assert_eq!(format_list(&["a", "b", "c"]), "a, b, c");
}

#[test]
fn test_json_serialization() {
    let detail = create_test_detail();
    let json = render_json(&detail).unwrap();

    assert!(json.contains("\"name\": \"test-plugin\""));
    assert!(json.contains("\"version\": \"1.0.0\""));
    assert!(json.contains("\"installedAt\": \"2025-01-15T10:30:00Z\""));

    assert!(json.contains("\"author\""));
    assert!(json.contains("\"Test Author\""));

    assert!(json.contains("\"type\": \"github\""));
    assert!(json.contains("\"repository\": \"owner/repo\""));

    assert!(json.contains("\"cachePath\""));
    assert!(json.contains(&format!("\"cachePath\": \"{}\"", TEST_CACHE_PATH)));
    assert!(json.contains("\"enabled\""));
    assert!(json.contains(&format!("\"enabled\": {}", TEST_ENABLED)));
}

#[test]
fn test_json_serialization_no_author() {
    let mut detail = create_test_detail();
    detail.author = None;

    let json = render_json(&detail).unwrap();

    assert!(!json.contains("\"author\""));
}

#[test]
fn test_yaml_serialization() {
    let detail = create_test_detail();
    let yaml = render_yaml(&detail).unwrap();

    assert!(yaml.contains("name: test-plugin"));
    assert!(yaml.contains("version: 1.0.0"));
    assert!(yaml.contains("type: github"));

    assert!(yaml.contains("cachePath:"));
    assert!(yaml.contains(TEST_CACHE_PATH));
    assert!(yaml.contains("enabled:"));
    assert!(yaml.contains(&format!("enabled: {}", TEST_ENABLED)));
}

#[test]
fn table_output_contains_cache_path() {
    let detail = create_test_detail();
    let rendered = render_table(&detail);
    assert!(rendered.contains("Cache Path"));
    assert!(rendered.contains(TEST_CACHE_PATH));
}

#[test]
fn table_output_contains_status() {
    let detail = create_test_detail();
    let rendered = render_table(&detail);
    assert!(rendered.contains("Status"));
    assert!(rendered.contains("enabled"));

    let mut disabled_detail = create_test_detail();
    disabled_detail.enabled = false;
    let rendered = render_table(&disabled_detail);
    assert!(rendered.contains("disabled"));
}

#[test]
fn table_output_omits_author_section_when_none() {
    let mut detail = create_test_detail();
    detail.author = None;
    let rendered = render_table(&detail);
    assert!(!rendered.contains("Author\n------"));
}

#[test]
fn table_output_formats_source_github() {
    let detail = create_test_detail();
    let rendered = render_table(&detail);
    assert!(rendered.contains("GitHub (owner/repo)"));
}

#[test]
fn table_output_formats_source_marketplace() {
    let mut detail = create_test_detail();
    detail.source = PluginSource::Marketplace {
        name: "official".to_string(),
    };
    let rendered = render_table(&detail);
    assert!(rendered.contains("Marketplace (official)"));
}

#[test]
fn test_json_components_nested_shape() {
    let detail = create_test_detail();
    let json: serde_json::Value = serde_json::to_value(&detail).unwrap();

    let components = &json["components"];
    assert_eq!(
        components["skills"],
        serde_json::json!(["skill1", "skill2"])
    );
    assert_eq!(components["commands"], serde_json::json!(["cmd1"]));
    assert_eq!(components["agents"], serde_json::json!([]));
    assert_eq!(components["instructions"], serde_json::json!([]));
    assert_eq!(components["hooks"], serde_json::json!([]));
}
