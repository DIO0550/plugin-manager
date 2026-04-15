use super::*;
use crate::component::{Component, ComponentKind};
use crate::plugin::PackageCache;
use std::fs;
use std::path::{Path, PathBuf};
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

fn comp(kind: ComponentKind, name: &str) -> Component {
    Component {
        kind,
        name: name.to_string(),
        path: PathBuf::from(format!("dummy/{}", name)),
    }
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
    assert_eq!(result[0].name, "my-plugin");
    assert_eq!(result[0].version, "1.0.0");
    assert!(result[0].marketplace.is_none());
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
    assert_eq!(result[0].name, "good-plugin");
}

#[test]
fn test_list_installed_plugins_no_manifest_excluded() {
    let (temp_dir, cache) = create_test_cache();
    setup_plugin_fixture(temp_dir.path(), "github", "valid-plugin", "1.0.0");
    let no_manifest_dir = temp_dir.path().join("github").join("no-manifest");
    fs::create_dir_all(&no_manifest_dir).unwrap();

    let result = list_installed_plugins(&cache).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "valid-plugin");
}

fn create_empty_summary() -> PluginSummary {
    PluginSummary {
        name: "test-plugin".to_string(),
        cache_key: None,
        marketplace: None,
        version: "1.0.0".to_string(),
        components: Vec::new(),
        enabled: true,
    }
}

fn create_full_summary() -> PluginSummary {
    PluginSummary {
        name: "full-plugin".to_string(),
        cache_key: None,
        marketplace: Some("awesome-marketplace".to_string()),
        version: "2.0.0".to_string(),
        components: vec![
            comp(ComponentKind::Skill, "skill1"),
            comp(ComponentKind::Skill, "skill2"),
            comp(ComponentKind::Agent, "agent1"),
            comp(ComponentKind::Command, "cmd1"),
            comp(ComponentKind::Command, "cmd2"),
            comp(ComponentKind::Command, "cmd3"),
            comp(ComponentKind::Instruction, "inst1"),
            comp(ComponentKind::Hook, "hook1"),
            comp(ComponentKind::Hook, "hook2"),
        ],
        enabled: true,
    }
}

// ========================================
// cache_key tests
// ========================================

#[test]
fn test_plugin_summary_cache_key_returns_some_value() {
    let mut summary = create_empty_summary();
    summary.cache_key = Some("owner--repo".to_string());
    assert_eq!(summary.cache_key(), "owner--repo");
}

#[test]
fn test_plugin_summary_cache_key_falls_back_to_name() {
    let summary = create_empty_summary();
    assert_eq!(summary.cache_key(), "test-plugin");
}

// ========================================
// component_type_counts tests
// ========================================

#[test]
fn test_component_type_counts_empty() {
    let summary = create_empty_summary();
    let counts = summary.component_type_counts();
    assert!(counts.is_empty());
}

#[test]
fn test_component_type_counts_full() {
    let summary = create_full_summary();
    let counts = summary.component_type_counts();

    assert_eq!(counts.len(), 5);

    let skill_count = counts
        .iter()
        .find(|c| c.kind == ComponentKind::Skill)
        .unwrap();
    assert_eq!(skill_count.count, 2);

    let agent_count = counts
        .iter()
        .find(|c| c.kind == ComponentKind::Agent)
        .unwrap();
    assert_eq!(agent_count.count, 1);

    let cmd_count = counts
        .iter()
        .find(|c| c.kind == ComponentKind::Command)
        .unwrap();
    assert_eq!(cmd_count.count, 3);

    let inst_count = counts
        .iter()
        .find(|c| c.kind == ComponentKind::Instruction)
        .unwrap();
    assert_eq!(inst_count.count, 1);

    let hook_count = counts
        .iter()
        .find(|c| c.kind == ComponentKind::Hook)
        .unwrap();
    assert_eq!(hook_count.count, 2);
}

#[test]
fn test_component_type_counts_partial() {
    let summary = PluginSummary {
        name: "partial".to_string(),
        cache_key: None,
        marketplace: None,
        version: "1.0.0".to_string(),
        components: vec![
            comp(ComponentKind::Agent, "a1"),
            comp(ComponentKind::Agent, "a2"),
            comp(ComponentKind::Hook, "h1"),
        ],
        enabled: true,
    };

    let counts = summary.component_type_counts();

    assert_eq!(counts.len(), 2);
    assert!(counts
        .iter()
        .any(|c| c.kind == ComponentKind::Agent && c.count == 2));
    assert!(counts
        .iter()
        .any(|c| c.kind == ComponentKind::Hook && c.count == 1));
}

#[test]
fn test_component_type_counts_order() {
    let summary = create_full_summary();
    let counts = summary.component_type_counts();

    assert_eq!(counts[0].kind, ComponentKind::Skill);
    assert_eq!(counts[1].kind, ComponentKind::Agent);
    assert_eq!(counts[2].kind, ComponentKind::Command);
    assert_eq!(counts[3].kind, ComponentKind::Instruction);
    assert_eq!(counts[4].kind, ComponentKind::Hook);
}

// ========================================
// component_names tests
// ========================================

#[test]
fn test_component_names_skills() {
    let summary = create_full_summary();
    let names = summary.component_names(ComponentKind::Skill);

    assert_eq!(names.len(), 2);
    assert_eq!(names[0].name, "skill1");
    assert_eq!(names[1].name, "skill2");
}

#[test]
fn test_component_names_agents() {
    let summary = create_full_summary();
    let names = summary.component_names(ComponentKind::Agent);

    assert_eq!(names.len(), 1);
    assert_eq!(names[0].name, "agent1");
}

#[test]
fn test_component_names_commands() {
    let summary = create_full_summary();
    let names = summary.component_names(ComponentKind::Command);

    assert_eq!(names.len(), 3);
}

#[test]
fn test_component_names_instructions() {
    let summary = create_full_summary();
    let names = summary.component_names(ComponentKind::Instruction);

    assert_eq!(names.len(), 1);
    assert_eq!(names[0].name, "inst1");
}

#[test]
fn test_component_names_hooks() {
    let summary = create_full_summary();
    let names = summary.component_names(ComponentKind::Hook);

    assert_eq!(names.len(), 2);
}

#[test]
fn test_component_names_empty() {
    let summary = create_empty_summary();

    for kind in ComponentKind::all() {
        let names = summary.component_names(*kind);
        assert!(names.is_empty(), "{:?} should be empty", kind);
    }
}
