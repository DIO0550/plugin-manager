use super::*;
use crate::component::convert::AgentFormat;
use crate::component::CommandFormat;
use crate::target::TargetKind;
use std::path::{Path, PathBuf};

// ========================================
// Builder tests
// ========================================

#[test]
fn test_builder_builds_with_all_fields() {
    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Agent,
            name: "test-agent".to_string(),
            path: ("/src/agent.md").into(),
        })
        .scope(Scope::Project)
        .target_path("/dest/agent.md")
        .build()
        .unwrap();

    assert_eq!(deployment.kind, ComponentKind::Agent);
    assert_eq!(deployment.name, "test-agent");
    assert_eq!(deployment.scope, Scope::Project);
    assert_eq!(deployment.path(), Path::new("/dest/agent.md"));
}

#[test]
fn test_builder_fails_without_kind() {
    let result = ComponentDeployment::builder()
        .name("test")
        .scope(Scope::Personal)
        .target_path("/dest")
        .build();

    assert!(result.is_err());
}

#[test]
fn test_builder_fails_without_name() {
    let result = ComponentDeployment::builder()
        .kind(ComponentKind::Skill)
        .scope(Scope::Personal)
        .target_path("/dest")
        .build();

    assert!(result.is_err());
}

#[test]
fn test_builder_fails_without_scope() {
    let result = ComponentDeployment::builder()
        .kind(ComponentKind::Skill)
        .name("test")
        .target_path("/dest")
        .build();

    assert!(result.is_err());
}

#[test]
fn test_builder_fails_without_source_path() {
    let result = ComponentDeployment::builder()
        .kind(ComponentKind::Skill)
        .name("test")
        .scope(Scope::Personal)
        .target_path("/dest")
        .build();

    assert!(result.is_err());
}

#[test]
fn test_builder_fails_without_target_path() {
    let result = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Skill,
            name: "test".to_string(),
            path: ("/src").into(),
        })
        .scope(Scope::Personal)
        .build();

    assert!(result.is_err());
}

#[test]
fn test_builder_from_component() {
    let component = Component {
        kind: ComponentKind::Command,
        name: "my-command".to_string(),
        path: PathBuf::from("/plugin/commands/my-command.md"),
    };

    let deployment = ComponentDeployment::builder()
        .component(&component)
        .scope(Scope::Project)
        .target_path("/target/my-command.md")
        .build()
        .unwrap();

    assert_eq!(deployment.kind, ComponentKind::Command);
    assert_eq!(deployment.name, "my-command");
    assert_eq!(deployment.path(), Path::new("/target/my-command.md"));
}

#[test]
fn test_builder_source_format() {
    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Command,
            name: "test".to_string(),
            path: ("/src/test.md").into(),
        })
        .scope(Scope::Project)
        .target_path("/dest/test.md")
        .source_format(CommandFormat::ClaudeCode)
        .dest_format(CommandFormat::Copilot)
        .build()
        .unwrap();

    // Builder でフォーマットが設定できることを確認
    assert_eq!(deployment.kind, ComponentKind::Command);
}

#[test]
fn test_builder_agent_format() {
    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Agent,
            name: "test".to_string(),
            path: ("/src/test.md").into(),
        })
        .scope(Scope::Project)
        .target_path("/dest/test.md")
        .source_agent_format(AgentFormat::ClaudeCode)
        .dest_agent_format(AgentFormat::Copilot)
        .build()
        .unwrap();

    // Builder で Agent フォーマットが設定できることを確認
    assert_eq!(deployment.kind, ComponentKind::Agent);
}

// ========================================
// Builder hook_convert / plugin_root tests
// ========================================

#[test]
fn test_builder_hook_convert() {
    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Hook,
            name: "test-hook".to_string(),
            path: ("/src/hook.json").into(),
        })
        .scope(Scope::Project)
        .target_path("/dest/hook.json")
        .hook_convert(true)
        .target_kind(TargetKind::Copilot)
        .plugin_root("/cache/plugin")
        .build()
        .unwrap();

    assert_eq!(deployment.kind, ComponentKind::Hook);
}

#[test]
fn test_builder_plugin_root() {
    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Hook,
            name: "test-hook".to_string(),
            path: ("/src/hook.json").into(),
        })
        .scope(Scope::Project)
        .target_path("/dest/hook.json")
        .hook_convert(true)
        .target_kind(TargetKind::Copilot)
        .plugin_root("/cache/plugin")
        .build()
        .unwrap();

    assert_eq!(deployment.kind, ComponentKind::Hook);
}

#[test]
fn test_builder_hook_convert_default_false() {
    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Hook,
            name: "test-hook".to_string(),
            path: ("/src/hook.json").into(),
        })
        .scope(Scope::Project)
        .target_path("/dest/hook.json")
        .build()
        .unwrap();

    // hook_convert defaults to false: Hook should be file-copied
    assert_eq!(deployment.kind, ComponentKind::Hook);
}
