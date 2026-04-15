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

fn create_empty_plugin(name: &str) -> InstalledPlugin {
    InstalledPlugin::new_for_test(name, "1.0.0", Vec::new(), None, None, false)
}

fn create_plugin_with_skills(name: &str, skill_count: usize, enabled: bool) -> InstalledPlugin {
    InstalledPlugin::new_for_test(
        name,
        "1.0.0",
        (0..skill_count)
            .map(|i| comp(ComponentKind::Skill, &format!("skill{}", i)))
            .collect(),
        None,
        Some("github".to_string()),
        enabled,
    )
}

fn create_full_plugin(name: &str, enabled: bool) -> InstalledPlugin {
    InstalledPlugin::new_for_test(
        name,
        "2.0.0",
        vec![
            comp(ComponentKind::Skill, "skill1"),
            comp(ComponentKind::Skill, "skill2"),
            comp(ComponentKind::Agent, "agent1"),
            comp(ComponentKind::Command, "cmd1"),
            comp(ComponentKind::Instruction, "inst1"),
            comp(ComponentKind::Hook, "hook1"),
        ],
        None,
        Some("github".to_string()),
        enabled,
    )
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
    assert_eq!(filtered[0].name(), "enabled-with-skills");
    assert_eq!(filtered[1].name(), "enabled-full");
}

#[test]
fn test_json_components_nested_shape() {
    let plugin = create_full_plugin("test", true);
    let json: serde_json::Value = serde_json::to_value(&plugin).unwrap();

    // Phase 6: components are nested under "components" key (not flattened)
    let components = json
        .get("components")
        .expect("components should be nested object");
    assert_eq!(
        components["skills"],
        serde_json::json!(["skill1", "skill2"])
    );
    assert_eq!(components["agents"], serde_json::json!(["agent1"]));
    assert_eq!(components["commands"], serde_json::json!(["cmd1"]));
    assert_eq!(components["instructions"], serde_json::json!(["inst1"]));
    assert_eq!(components["hooks"], serde_json::json!(["hook1"]));
    // skills should NOT be at top level
    assert!(json.get("skills").is_none());
}

// ========================================
// JSON snapshot tests (Phase 6 new shape)
// ========================================

fn snapshot_plugin_full() -> InstalledPlugin {
    InstalledPlugin::new_for_test(
        "my-plugin",
        "1.2.3",
        vec![
            comp(ComponentKind::Skill, "skill-a"),
            comp(ComponentKind::Skill, "skill-b"),
            comp(ComponentKind::Agent, "agent-a"),
            comp(ComponentKind::Command, "cmd-a"),
            comp(ComponentKind::Instruction, "inst-a"),
            comp(ComponentKind::Hook, "hook-a"),
        ],
        Some("my-plugin-abc".to_string()),
        Some("github".to_string()),
        true,
    )
}

#[test]
fn test_plugin_summary_json_snapshot_full() {
    let plugin = snapshot_plugin_full();
    let actual = serde_json::to_string_pretty(&plugin).unwrap();
    let expected = r#"{
  "name": "my-plugin",
  "version": "1.2.3",
  "install_id": "my-plugin-abc",
  "marketplace": "github",
  "enabled": true,
  "components": {
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
    ]
  }
}"#;
    assert_eq!(actual, expected);
}

#[test]
fn test_plugin_summary_json_snapshot_install_id_fallback() {
    // install_id = None → name にフォールバックし、常に install_id キーは出力される
    let plugin = InstalledPlugin::new_for_test(
        "no-install-id",
        "0.1.0",
        Vec::new(),
        None,
        Some("github".to_string()),
        false,
    );
    let actual = serde_json::to_string_pretty(&plugin).unwrap();
    let expected = r#"{
  "name": "no-install-id",
  "version": "0.1.0",
  "install_id": "no-install-id",
  "marketplace": "github",
  "enabled": false,
  "components": {
    "skills": [],
    "agents": [],
    "commands": [],
    "instructions": [],
    "hooks": []
  }
}"#;
    assert_eq!(actual, expected);
}

#[test]
fn test_plugin_summary_json_snapshot_marketplace_none() {
    // marketplace = None → marketplace キー自体を出力しない
    let plugin = InstalledPlugin::new_for_test(
        "local-plugin",
        "0.0.1",
        Vec::new(),
        Some("local-plugin".to_string()),
        None,
        true,
    );
    let actual = serde_json::to_string_pretty(&plugin).unwrap();
    let expected = r#"{
  "name": "local-plugin",
  "version": "0.0.1",
  "install_id": "local-plugin",
  "enabled": true,
  "components": {
    "skills": [],
    "agents": [],
    "commands": [],
    "instructions": [],
    "hooks": []
  }
}"#;
    assert_eq!(actual, expected);
}

#[test]
fn test_outdated_json_snapshot() {
    use crate::plugin::UpdateCheck;

    // Case A: update available
    let plugin_a = snapshot_plugin_full();
    let check_a = UpdateCheck::Available {
        current_sha: Some("abc1234567890".to_string()),
        latest_sha: "def1234567890".to_string(),
    };
    let entry_a = OutdatedEntry {
        plugin: &plugin_a,
        check: &check_a,
    };
    let actual_a = serde_json::to_string_pretty(&entry_a).unwrap();
    let expected_a = r#"{
  "plugin": {
    "name": "my-plugin",
    "version": "1.2.3",
    "install_id": "my-plugin-abc",
    "marketplace": "github",
    "enabled": true,
    "components": {
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
      ]
    }
  },
  "check": {
    "status": "available",
    "current_sha": "abc1234567890",
    "latest_sha": "def1234567890"
  }
}"#;
    assert_eq!(actual_a, expected_a);

    // Case B: check failed
    let plugin_b = InstalledPlugin::new_for_test(
        "failing-plugin",
        "0.0.1",
        Vec::new(),
        Some("failing-plugin".to_string()),
        Some("github".to_string()),
        true,
    );
    let check_b = UpdateCheck::Failed {
        current_sha: None,
        error: "network error".to_string(),
    };
    let entry_b = OutdatedEntry {
        plugin: &plugin_b,
        check: &check_b,
    };
    let actual_b = serde_json::to_string_pretty(&entry_b).unwrap();
    let expected_b = r#"{
  "plugin": {
    "name": "failing-plugin",
    "version": "0.0.1",
    "install_id": "failing-plugin",
    "marketplace": "github",
    "enabled": true,
    "components": {
      "skills": [],
      "agents": [],
      "commands": [],
      "instructions": [],
      "hooks": []
    }
  },
  "check": {
    "status": "failed",
    "current_sha": null,
    "error": "network error"
  }
}"#;
    assert_eq!(actual_b, expected_b);

    // Case C: up to date
    let plugin_c = InstalledPlugin::new_for_test(
        "uptodate-plugin",
        "1.0.0",
        Vec::new(),
        Some("uptodate-plugin".to_string()),
        Some("github".to_string()),
        true,
    );
    let check_c = UpdateCheck::UpToDate {
        current_sha: Some("same1234567890".to_string()),
        latest_sha: "same1234567890".to_string(),
    };
    let entry_c = OutdatedEntry {
        plugin: &plugin_c,
        check: &check_c,
    };
    let actual_c = serde_json::to_string_pretty(&entry_c).unwrap();
    let expected_c = r#"{
  "plugin": {
    "name": "uptodate-plugin",
    "version": "1.0.0",
    "install_id": "uptodate-plugin",
    "marketplace": "github",
    "enabled": true,
    "components": {
      "skills": [],
      "agents": [],
      "commands": [],
      "instructions": [],
      "hooks": []
    }
  },
  "check": {
    "status": "up_to_date",
    "current_sha": "same1234567890",
    "latest_sha": "same1234567890"
  }
}"#;
    assert_eq!(actual_c, expected_c);
}
