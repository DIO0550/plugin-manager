use super::*;
use crate::component::{Component, ComponentKind};
use std::path::PathBuf;

fn comp(kind: ComponentKind, name: &str) -> Component {
    Component {
        kind,
        name: name.to_string(),
        path: PathBuf::from(format!("dummy/{}", name)),
    }
}

fn create_empty_plugin(name: &str) -> PluginSummary {
    PluginSummary {
        name: name.to_string(),
        cache_key: None,
        marketplace: None,
        version: "1.0.0".to_string(),
        components: Vec::new(),
        enabled: false,
    }
}

fn create_plugin_with_skills(name: &str, skill_count: usize, enabled: bool) -> PluginSummary {
    PluginSummary {
        name: name.to_string(),
        cache_key: None,
        marketplace: Some("github".to_string()),
        version: "1.0.0".to_string(),
        components: (0..skill_count)
            .map(|i| comp(ComponentKind::Skill, &format!("skill{}", i)))
            .collect(),
        enabled,
    }
}

fn create_full_plugin(name: &str, enabled: bool) -> PluginSummary {
    PluginSummary {
        name: name.to_string(),
        cache_key: None,
        marketplace: Some("github".to_string()),
        version: "2.0.0".to_string(),
        components: vec![
            comp(ComponentKind::Skill, "skill1"),
            comp(ComponentKind::Skill, "skill2"),
            comp(ComponentKind::Agent, "agent1"),
            comp(ComponentKind::Command, "cmd1"),
            comp(ComponentKind::Instruction, "inst1"),
            comp(ComponentKind::Hook, "hook1"),
        ],
        enabled,
    }
}

// ========================================
// filter_by_type tests
// ========================================

#[test]
fn test_filter_by_type_none_passes_all() {
    let empty = create_empty_plugin("empty");
    let with_skills = create_plugin_with_skills("with-skills", 2, true);

    assert!(filter_by_type(&empty, None));
    assert!(filter_by_type(&with_skills, None));
}

#[test]
fn test_filter_by_type_skill_matches() {
    let plugin = create_plugin_with_skills("test", 2, true);
    assert!(filter_by_type(&plugin, Some(&ComponentKind::Skill)));
}

#[test]
fn test_filter_by_type_skill_no_match() {
    let plugin = create_empty_plugin("test");
    assert!(!filter_by_type(&plugin, Some(&ComponentKind::Skill)));
}

#[test]
fn test_filter_by_type_agent_matches() {
    let plugin = create_full_plugin("test", true);
    assert!(filter_by_type(&plugin, Some(&ComponentKind::Agent)));
}

#[test]
fn test_filter_by_type_command_matches() {
    let plugin = create_full_plugin("test", true);
    assert!(filter_by_type(&plugin, Some(&ComponentKind::Command)));
}

#[test]
fn test_filter_by_type_instruction_matches() {
    let plugin = create_full_plugin("test", true);
    assert!(filter_by_type(&plugin, Some(&ComponentKind::Instruction)));
}

#[test]
fn test_filter_by_type_hook_matches() {
    let plugin = create_full_plugin("test", true);
    assert!(filter_by_type(&plugin, Some(&ComponentKind::Hook)));
}

#[test]
fn test_filter_by_type_hook_no_match() {
    let plugin = create_plugin_with_skills("test", 1, true);
    assert!(!filter_by_type(&plugin, Some(&ComponentKind::Hook)));
}

// ========================================
// filter_by_target tests
// ========================================

#[test]
fn test_filter_by_target_none_passes_all() {
    let enabled = create_plugin_with_skills("enabled", 1, true);
    let disabled = create_plugin_with_skills("disabled", 1, false);

    assert!(filter_by_target(&enabled, None));
    assert!(filter_by_target(&disabled, None));
}

#[test]
fn test_filter_by_target_enabled_only() {
    let enabled = create_plugin_with_skills("enabled", 1, true);
    let disabled = create_plugin_with_skills("disabled", 1, false);

    assert!(filter_by_target(&enabled, Some(&TargetKind::Codex)));
    assert!(!filter_by_target(&disabled, Some(&TargetKind::Codex)));

    assert!(filter_by_target(&enabled, Some(&TargetKind::Copilot)));
    assert!(!filter_by_target(&disabled, Some(&TargetKind::Copilot)));
}

// ========================================
// format_components tests
// ========================================

#[test]
fn test_format_components_empty() {
    let plugin = create_empty_plugin("empty");
    assert_eq!(format_components(&plugin), "-");
}

#[test]
fn test_format_components_single() {
    let plugin = create_plugin_with_skills("test", 2, true);
    assert_eq!(format_components(&plugin), "2 skills");
}

#[test]
fn test_format_components_multiple() {
    let plugin = create_full_plugin("test", true);
    assert_eq!(
        format_components(&plugin),
        "2 skills, 1 agents, 1 commands, 1 instructions, 1 hooks"
    );
}

// ========================================
// filter_plugins tests
// ========================================

#[test]
fn test_filter_plugins_combined() {
    let plugins = vec![
        create_plugin_with_skills("enabled-with-skills", 2, true),
        create_plugin_with_skills("disabled-with-skills", 1, false),
        create_empty_plugin("enabled-empty"),
        create_full_plugin("enabled-full", true),
    ];

    let args = Args {
        component_type: Some(ComponentKind::Skill),
        target: Some(TargetKind::Codex),
        json: false,
        simple: false,
        outdated: false,
    };

    let filtered = filter_plugins(plugins, &args);

    assert_eq!(filtered.len(), 2);
    assert_eq!(filtered[0].name, "enabled-with-skills");
    assert_eq!(filtered[1].name, "enabled-full");
}

#[test]
fn test_json_components_flatten_shape() {
    let plugin = create_full_plugin("test", true);
    let json: serde_json::Value = serde_json::to_value(&plugin).unwrap();

    // flatten: skills/agents/... are top-level, not nested under "components"
    assert!(json.get("components").is_none());
    assert_eq!(json["skills"], serde_json::json!(["skill1", "skill2"]));
    assert_eq!(json["agents"], serde_json::json!(["agent1"]));
    assert_eq!(json["commands"], serde_json::json!(["cmd1"]));
    assert_eq!(json["instructions"], serde_json::json!(["inst1"]));
    assert_eq!(json["hooks"], serde_json::json!(["hook1"]));
}

// ========================================
// JSON snapshot tests (Phase 1)
// Guard for Phase 2-5: these snapshots must stay green unchanged.
// Phase 6 intentionally updates them to the new JSON shape.
// ========================================

fn snapshot_plugin_full() -> PluginSummary {
    PluginSummary {
        name: "my-plugin".to_string(),
        cache_key: Some("my-plugin-abc".to_string()),
        marketplace: Some("github".to_string()),
        version: "1.2.3".to_string(),
        components: vec![
            comp(ComponentKind::Skill, "skill-a"),
            comp(ComponentKind::Skill, "skill-b"),
            comp(ComponentKind::Agent, "agent-a"),
            comp(ComponentKind::Command, "cmd-a"),
            comp(ComponentKind::Instruction, "inst-a"),
            comp(ComponentKind::Hook, "hook-a"),
        ],
        enabled: true,
    }
}

#[test]
fn test_plugin_summary_json_snapshot_full() {
    let plugin = snapshot_plugin_full();
    let actual = serde_json::to_string_pretty(&plugin).unwrap();
    let expected = r#"{
  "name": "my-plugin",
  "cache_key": "my-plugin-abc",
  "marketplace": "github",
  "version": "1.2.3",
  "skills": [
    "skill-a",
    "skill-b"
  ],
  "agents": [
    "agent-a"
  ],
  "commands": [
    "cmd-a"
  ],
  "instructions": [
    "inst-a"
  ],
  "hooks": [
    "hook-a"
  ],
  "enabled": true
}"#;
    assert_eq!(actual, expected);
}

#[test]
fn test_plugin_summary_json_snapshot_cache_key_none() {
    let plugin = PluginSummary {
        name: "no-cache-key".to_string(),
        cache_key: None,
        marketplace: Some("github".to_string()),
        version: "0.1.0".to_string(),
        components: Vec::new(),
        enabled: false,
    };
    let actual = serde_json::to_string_pretty(&plugin).unwrap();
    let expected = r#"{
  "name": "no-cache-key",
  "marketplace": "github",
  "version": "0.1.0",
  "skills": [],
  "agents": [],
  "commands": [],
  "instructions": [],
  "hooks": [],
  "enabled": false
}"#;
    assert_eq!(actual, expected);
}

#[test]
fn test_plugin_summary_json_snapshot_marketplace_null() {
    let plugin = PluginSummary {
        name: "local-plugin".to_string(),
        cache_key: Some("local-plugin".to_string()),
        marketplace: None,
        version: "0.0.1".to_string(),
        components: Vec::new(),
        enabled: true,
    };
    let actual = serde_json::to_string_pretty(&plugin).unwrap();
    let expected = r#"{
  "name": "local-plugin",
  "cache_key": "local-plugin",
  "marketplace": null,
  "version": "0.0.1",
  "skills": [],
  "agents": [],
  "commands": [],
  "instructions": [],
  "hooks": [],
  "enabled": true
}"#;
    assert_eq!(actual, expected);
}

#[test]
fn test_outdated_json_snapshot() {
    // Case A: update available
    let entry_a = PluginWithUpdateInfo {
        summary: snapshot_plugin_full(),
        current_sha: Some("abc1234567890".to_string()),
        latest_sha: Some("def1234567890".to_string()),
        has_update: true,
        check_error: None,
    };
    let actual_a = serde_json::to_string_pretty(&entry_a).unwrap();
    let expected_a = r#"{
  "name": "my-plugin",
  "cache_key": "my-plugin-abc",
  "marketplace": "github",
  "version": "1.2.3",
  "skills": [
    "skill-a",
    "skill-b"
  ],
  "agents": [
    "agent-a"
  ],
  "commands": [
    "cmd-a"
  ],
  "instructions": [
    "inst-a"
  ],
  "hooks": [
    "hook-a"
  ],
  "enabled": true,
  "current_sha": "abc1234567890",
  "latest_sha": "def1234567890",
  "has_update": true
}"#;
    assert_eq!(actual_a, expected_a);

    // Case B: check failed
    let entry_b = PluginWithUpdateInfo {
        summary: PluginSummary {
            name: "failing-plugin".to_string(),
            cache_key: Some("failing-plugin".to_string()),
            marketplace: Some("github".to_string()),
            version: "0.0.1".to_string(),
            components: Vec::new(),
            enabled: true,
        },
        current_sha: None,
        latest_sha: None,
        has_update: false,
        check_error: Some("network error".to_string()),
    };
    let actual_b = serde_json::to_string_pretty(&entry_b).unwrap();
    let expected_b = r#"{
  "name": "failing-plugin",
  "cache_key": "failing-plugin",
  "marketplace": "github",
  "version": "0.0.1",
  "skills": [],
  "agents": [],
  "commands": [],
  "instructions": [],
  "hooks": [],
  "enabled": true,
  "current_sha": null,
  "latest_sha": null,
  "has_update": false,
  "check_error": "network error"
}"#;
    assert_eq!(actual_b, expected_b);
}
