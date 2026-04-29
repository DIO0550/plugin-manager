use super::json::render_json;
use super::table::{format_list, render_table};
use super::yaml::render_yaml;
use crate::application::{InstalledPlugin, PluginInfo, Source};
use crate::component::{Component, ComponentKind};
use crate::plugin::{Author, PluginManifest};
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

fn create_test_info() -> PluginInfo {
    let manifest = PluginManifest {
        name: "test-plugin".to_string(),
        version: "1.0.0".to_string(),
        description: Some("A test plugin".to_string()),
        author: Some(Author {
            name: "Test Author".to_string(),
            email: Some("test@example.com".to_string()),
            url: None,
        }),
        homepage: None,
        repository: None,
        license: None,
        keywords: None,
        commands: None,
        agents: None,
        skills: None,
        instructions: None,
        hooks: None,
        mcp_servers: None,
        lsp_servers: None,
    };
    let installed = InstalledPlugin::new_for_test_full(
        manifest,
        PathBuf::from(TEST_CACHE_PATH),
        vec![
            comp(ComponentKind::Skill, "skill1"),
            comp(ComponentKind::Skill, "skill2"),
            comp(ComponentKind::Command, "cmd1"),
        ],
        Some("owner--repo".to_string()),
        Some("github".to_string()),
        TEST_ENABLED,
    );
    PluginInfo {
        installed,
        installed_at: Some("2025-01-15T10:30:00Z".to_string()),
        source: Source::GitHub {
            repository: "owner/repo".to_string(),
        },
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
    let info = create_test_info();
    let json = render_json(&info).unwrap();

    assert!(json.contains("\"name\": \"test-plugin\""));
    assert!(json.contains("\"version\": \"1.0.0\""));
    assert!(json.contains("\"installed_at\": \"2025-01-15T10:30:00Z\""));

    assert!(json.contains("\"author\""));
    assert!(json.contains("\"Test Author\""));

    assert!(json.contains("\"type\": \"github\""));
    assert!(json.contains("\"repository\": \"owner/repo\""));

    assert!(json.contains("\"cache_path\""));
    assert!(json.contains(&format!("\"cache_path\": \"{}\"", TEST_CACHE_PATH)));
    assert!(json.contains("\"enabled\""));
    assert!(json.contains(&format!("\"enabled\": {}", TEST_ENABLED)));

    // 旧 camelCase キーが出力されないことを明示的に検証
    assert!(!json.contains("\"cachePath\""));
    assert!(!json.contains("\"installedAt\""));
}

#[test]
fn test_json_serialization_no_author() {
    let mut info = create_test_info();
    let manifest = PluginManifest {
        name: "test-plugin".to_string(),
        version: "1.0.0".to_string(),
        description: Some("A test plugin".to_string()),
        author: None,
        homepage: None,
        repository: None,
        license: None,
        keywords: None,
        commands: None,
        agents: None,
        skills: None,
        instructions: None,
        hooks: None,
        mcp_servers: None,
        lsp_servers: None,
    };
    info.installed = InstalledPlugin::new_for_test_full(
        manifest,
        PathBuf::from(TEST_CACHE_PATH),
        vec![],
        None,
        Some("github".to_string()),
        TEST_ENABLED,
    );

    let json = render_json(&info).unwrap();

    assert!(!json.contains("\"author\""));
}

#[test]
fn test_yaml_serialization() {
    let info = create_test_info();
    let yaml = render_yaml(&info).unwrap();

    assert!(yaml.contains("name: test-plugin"));
    assert!(yaml.contains("version: 1.0.0"));
    assert!(yaml.contains("type: github"));

    assert!(yaml.contains("cache_path:"));
    assert!(yaml.contains(TEST_CACHE_PATH));
    assert!(yaml.contains("enabled:"));
    assert!(yaml.contains(&format!("enabled: {}", TEST_ENABLED)));

    // 旧 camelCase キーが出力されないことを明示的に検証
    assert!(!yaml.contains("cachePath:"));
    assert!(!yaml.contains("installedAt:"));
}

#[test]
fn table_output_contains_cache_path() {
    let info = create_test_info();
    let rendered = render_table(&info);
    assert!(rendered.contains("Cache Path"));
    assert!(rendered.contains(TEST_CACHE_PATH));
}

#[test]
fn table_output_contains_status() {
    let info = create_test_info();
    let rendered = render_table(&info);
    assert!(rendered.contains("Status"));
    assert!(rendered.contains("enabled"));

    let manifest = PluginManifest {
        name: "test-plugin".to_string(),
        version: "1.0.0".to_string(),
        description: None,
        author: None,
        homepage: None,
        repository: None,
        license: None,
        keywords: None,
        commands: None,
        agents: None,
        skills: None,
        instructions: None,
        hooks: None,
        mcp_servers: None,
        lsp_servers: None,
    };
    let disabled_info = PluginInfo {
        installed: InstalledPlugin::new_for_test_full(
            manifest,
            PathBuf::from(TEST_CACHE_PATH),
            vec![],
            None,
            Some("github".to_string()),
            false,
        ),
        installed_at: None,
        source: Source::GitHub {
            repository: "owner/repo".to_string(),
        },
    };
    let rendered = render_table(&disabled_info);
    assert!(rendered.contains("disabled"));
}

#[test]
fn table_output_omits_author_section_when_none() {
    let manifest = PluginManifest {
        name: "test-plugin".to_string(),
        version: "1.0.0".to_string(),
        description: None,
        author: None,
        homepage: None,
        repository: None,
        license: None,
        keywords: None,
        commands: None,
        agents: None,
        skills: None,
        instructions: None,
        hooks: None,
        mcp_servers: None,
        lsp_servers: None,
    };
    let info = PluginInfo {
        installed: InstalledPlugin::new_for_test_full(
            manifest,
            PathBuf::from(TEST_CACHE_PATH),
            vec![],
            None,
            Some("github".to_string()),
            true,
        ),
        installed_at: None,
        source: Source::GitHub {
            repository: "owner/repo".to_string(),
        },
    };
    let rendered = render_table(&info);
    assert!(!rendered.contains("Author\n------"));
}

#[test]
fn table_output_formats_source_github() {
    let info = create_test_info();
    let rendered = render_table(&info);
    assert!(rendered.contains("GitHub (owner/repo)"));
}

#[test]
fn table_output_formats_source_marketplace() {
    let mut info = create_test_info();
    info.source = Source::Marketplace {
        name: "official".to_string(),
    };
    let rendered = render_table(&info);
    assert!(rendered.contains("Marketplace (official)"));
}

#[test]
fn test_json_components_nested_shape() {
    let info = create_test_info();
    let json_str = render_json(&info).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

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
