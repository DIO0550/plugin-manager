use super::*;
use crate::component::{Component, ComponentKind};
use crate::plugin::{InstalledPlugin, UpgradeState};
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
fn render_outdated_json_outdated_case() {
    let plugin = snapshot_plugin_full();
    let check = UpgradeState::Outdated {
        current_sha: Some("abc1234567890".to_string()),
        latest_sha: "def1234567890".to_string(),
    };
    let actual = render_outdated_json(&[(&plugin, &check)]).unwrap();
    let expected = r#"[
  {
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
      "status": "outdated",
      "current_sha": "abc1234567890",
      "latest_sha": "def1234567890"
    }
  }
]"#;
    assert_eq!(actual, expected);
}

#[test]
fn render_outdated_json_unknown_case() {
    let plugin = InstalledPlugin::new_for_test(
        "failing-plugin",
        "0.0.1",
        Vec::new(),
        Some("failing-plugin".to_string()),
        Some("github".to_string()),
        true,
    );
    let check = UpgradeState::Unknown {
        current_sha: None,
        error: "network error".to_string(),
    };
    let actual = render_outdated_json(&[(&plugin, &check)]).unwrap();
    let expected = r#"[
  {
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
      "status": "unknown",
      "current_sha": null,
      "error": "network error"
    }
  }
]"#;
    assert_eq!(actual, expected);
}

#[test]
fn render_outdated_json_latest_case() {
    let plugin = InstalledPlugin::new_for_test(
        "uptodate-plugin",
        "1.0.0",
        Vec::new(),
        Some("uptodate-plugin".to_string()),
        Some("github".to_string()),
        true,
    );
    let check = UpgradeState::Latest {
        current_sha: Some("same1234567890".to_string()),
        latest_sha: "same1234567890".to_string(),
    };
    let actual = render_outdated_json(&[(&plugin, &check)]).unwrap();
    let expected = r#"[
  {
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
      "status": "latest",
      "current_sha": "same1234567890",
      "latest_sha": "same1234567890"
    }
  }
]"#;
    assert_eq!(actual, expected);
}

#[test]
fn render_outdated_json_empty_entries() {
    let actual = render_outdated_json(&[]).unwrap();
    assert_eq!(actual, "[]");
}
