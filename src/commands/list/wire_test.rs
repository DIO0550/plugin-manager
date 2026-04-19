use super::*;
use crate::component::{Component, ComponentKind};
use crate::plugin::InstalledPlugin;
use std::path::PathBuf;

fn comp(kind: ComponentKind, name: &str) -> Component {
    Component {
        kind,
        name: name.to_string(),
        path: PathBuf::from(format!("dummy/{}", name)),
    }
}

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
fn wire_from_installed_snapshot_full() {
    let plugin = snapshot_plugin_full();
    let wire = Wire::from_installed(&plugin);
    let actual = serde_json::to_string_pretty(&wire).unwrap();
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
fn wire_id_fallback_to_name() {
    let plugin = InstalledPlugin::new_for_test(
        "no-install-id",
        "0.1.0",
        Vec::new(),
        None,
        Some("github".to_string()),
        false,
    );
    let wire = Wire::from_installed(&plugin);
    let actual = serde_json::to_string_pretty(&wire).unwrap();
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
fn wire_skips_marketplace_key_when_none() {
    let plugin = InstalledPlugin::new_for_test(
        "local-plugin",
        "0.0.1",
        Vec::new(),
        Some("local-plugin".to_string()),
        None,
        true,
    );
    let wire = Wire::from_installed(&plugin);
    let actual = serde_json::to_string_pretty(&wire).unwrap();
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
fn wire_components_nested_shape() {
    let plugin = snapshot_plugin_full();
    let wire = Wire::from_installed(&plugin);
    let json: serde_json::Value = serde_json::to_value(&wire).unwrap();

    let components = json
        .get("components")
        .expect("components should be nested object");
    assert_eq!(
        components["skills"],
        serde_json::json!(["skill-a", "skill-b"])
    );
    assert_eq!(components["agents"], serde_json::json!(["agent-a"]));
    assert_eq!(components["commands"], serde_json::json!(["cmd-a"]));
    assert_eq!(components["instructions"], serde_json::json!(["inst-a"]));
    assert_eq!(components["hooks"], serde_json::json!(["hook-a"]));
    assert!(json.get("skills").is_none());
}
