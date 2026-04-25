use super::*;
use crate::component::CommandFormat;
use crate::target::TargetKind;
use std::fs;
use tempfile::TempDir;

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

// ========================================
// Execute tests
// ========================================

#[test]
fn test_execute_copies_file_for_agent() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("agent.md");
    let target = temp.path().join("dest/agent.md");

    fs::write(&source, "agent content").unwrap();

    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Agent,
            name: "test-agent".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Project)
        .target_path(&target)
        .build()
        .unwrap();

    deployment.execute().unwrap();

    assert!(target.exists());
    assert_eq!(fs::read_to_string(&target).unwrap(), "agent content");
}

#[test]
fn test_execute_copies_file_for_command() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("cmd.prompt.md");
    let target = temp.path().join("dest/cmd.prompt.md");

    fs::write(&source, "command content").unwrap();

    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Command,
            name: "test-cmd".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Project)
        .target_path(&target)
        .build()
        .unwrap();

    deployment.execute().unwrap();

    assert!(target.exists());
}

#[test]
fn test_execute_copies_file_for_instruction() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("instruction.md");
    let target = temp.path().join("dest/instruction.md");

    fs::write(&source, "instruction content").unwrap();

    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Instruction,
            name: "test-instruction".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Personal)
        .target_path(&target)
        .build()
        .unwrap();

    deployment.execute().unwrap();

    assert!(target.exists());
}

#[test]
fn test_execute_copies_file_for_hook() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.sh");
    let target = temp.path().join("dest/hook.sh");

    fs::write(&source, "#!/bin/bash").unwrap();

    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Hook,
            name: "test-hook".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Personal)
        .target_path(&target)
        .build()
        .unwrap();

    deployment.execute().unwrap();

    assert!(target.exists());
}

#[test]
fn test_execute_copies_directory_for_skill() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("my-skill");
    let target = temp.path().join("dest/my-skill");

    fs::create_dir(&source).unwrap();
    fs::write(source.join("SKILL.md"), "skill content").unwrap();
    fs::write(source.join("helper.py"), "print('hello')").unwrap();

    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Skill,
            name: "my-skill".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Project)
        .target_path(&target)
        .build()
        .unwrap();

    deployment.execute().unwrap();

    assert!(target.exists());
    assert!(target.is_dir());
    assert!(target.join("SKILL.md").exists());
    assert!(target.join("helper.py").exists());
}

#[test]
fn test_execute_skill_replaces_existing_directory() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("skill");
    let target = temp.path().join("dest/skill");

    // Create source
    fs::create_dir(&source).unwrap();
    fs::write(source.join("new.md"), "new").unwrap();

    // Create existing target with different content
    fs::create_dir_all(&target).unwrap();
    fs::write(target.join("old.md"), "old").unwrap();

    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Skill,
            name: "skill".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Project)
        .target_path(&target)
        .build()
        .unwrap();

    deployment.execute().unwrap();

    assert!(!target.join("old.md").exists());
    assert!(target.join("new.md").exists());
}

// ========================================
// Accessor tests
// ========================================

#[test]
fn test_path_returns_target_path() {
    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Agent,
            name: "test".to_string(),
            path: ("/src/test.md").into(),
        })
        .scope(Scope::Project)
        .target_path("/dest/test.md")
        .build()
        .unwrap();

    assert_eq!(deployment.path(), Path::new("/dest/test.md"));
}

// ========================================
// Conversion tests
// ========================================

/// テスト用の ClaudeCode コマンドコンテンツ
fn sample_claude_code_content() -> &'static str {
    r#"---
name: commit
description: Generate a commit message
allowed-tools: Bash(git:*), Read
model: sonnet
---

Please generate a commit message for $ARGUMENTS.
"#
}

#[test]
fn test_execute_command_with_conversion() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("commit.md");
    let target = temp.path().join("dest/commit.prompt.md");

    fs::write(&source, sample_claude_code_content()).unwrap();

    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Command,
            name: "commit".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Project)
        .target_path(&target)
        .source_format(CommandFormat::ClaudeCode)
        .dest_format(CommandFormat::Copilot)
        .build()
        .unwrap();

    let result = deployment.execute().unwrap();

    // 変換が行われたことを確認
    match result {
        DeploymentOutput::CommandConverted(conv) => {
            assert!(conv.converted);
            assert_eq!(conv.source_format, CommandFormat::ClaudeCode);
            assert_eq!(conv.dest_format, CommandFormat::Copilot);
        }
        _ => panic!("Expected Converted result"),
    }

    // ファイルが Copilot 形式になっていることを確認
    let content = fs::read_to_string(&target).unwrap();
    assert!(content.contains("tools:"));
    assert!(content.contains("${arguments}"));
}

#[test]
fn test_execute_command_same_format_copies() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("commit.md");
    let target = temp.path().join("dest/commit.md");

    fs::write(&source, sample_claude_code_content()).unwrap();

    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Command,
            name: "commit".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Project)
        .target_path(&target)
        .source_format(CommandFormat::ClaudeCode)
        .dest_format(CommandFormat::ClaudeCode)
        .build()
        .unwrap();

    let result = deployment.execute().unwrap();

    // コピーのみ（変換なし）
    match result {
        DeploymentOutput::CommandConverted(conv) => {
            assert!(!conv.converted); // converted = false means copy
        }
        _ => panic!("Expected Converted with converted=false"),
    }

    // 内容はそのまま
    assert_eq!(
        fs::read_to_string(&target).unwrap(),
        sample_claude_code_content()
    );
}

#[test]
fn test_execute_command_without_format_copies() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("commit.md");
    let target = temp.path().join("dest/commit.md");

    fs::write(&source, "simple content").unwrap();

    // source_format と dest_format を設定しない
    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Command,
            name: "commit".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Project)
        .target_path(&target)
        .build()
        .unwrap();

    let result = deployment.execute().unwrap();

    // 変換設定がない場合は Copied
    match result {
        DeploymentOutput::Copied => {}
        _ => panic!("Expected Copied without format settings"),
    }

    assert!(target.exists());
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

// ========================================
// Agent Conversion tests
// ========================================

/// テスト用の ClaudeCode Agent コンテンツ
fn sample_claude_code_agent_content() -> &'static str {
    r#"---
name: code-review
description: Review code for best practices
tools: Read, Write, Bash
model: sonnet
---

You are a code review agent. Please review the code for best practices.
"#
}

#[test]
fn test_execute_agent_with_conversion_to_copilot() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("code-review.md");
    let target = temp.path().join("dest/code-review.agent.md");

    fs::write(&source, sample_claude_code_agent_content()).unwrap();

    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Agent,
            name: "code-review".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Project)
        .target_path(&target)
        .source_agent_format(AgentFormat::ClaudeCode)
        .dest_agent_format(AgentFormat::Copilot)
        .build()
        .unwrap();

    let result = deployment.execute().unwrap();

    // 変換が行われたことを確認
    match result {
        DeploymentOutput::AgentConverted(conv) => {
            assert!(conv.converted);
            assert_eq!(conv.source_format, AgentFormat::ClaudeCode);
            assert_eq!(conv.dest_format, AgentFormat::Copilot);
        }
        _ => panic!("Expected AgentConverted"),
    }

    // ファイルが Copilot 形式になっていることを確認
    let content = fs::read_to_string(&target).unwrap();
    assert!(content.contains("tools:"));
    assert!(content.contains("codebase")); // tools 変換
    assert!(content.contains("GPT-4o")); // model 変換
    assert!(content.contains("target: vscode")); // target 追加
}

#[test]
fn test_execute_agent_with_conversion_to_codex() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("code-review.md");
    let target = temp.path().join("dest/code-review.agent.md");

    fs::write(&source, sample_claude_code_agent_content()).unwrap();

    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Agent,
            name: "code-review".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Project)
        .target_path(&target)
        .source_agent_format(AgentFormat::ClaudeCode)
        .dest_agent_format(AgentFormat::Codex)
        .build()
        .unwrap();

    let result = deployment.execute().unwrap();

    // 変換が行われたことを確認
    match result {
        DeploymentOutput::AgentConverted(conv) => {
            assert!(conv.converted);
            assert_eq!(conv.source_format, AgentFormat::ClaudeCode);
            assert_eq!(conv.dest_format, AgentFormat::Codex);
        }
        _ => panic!("Expected AgentConverted"),
    }

    // ファイルが Codex 形式になっていることを確認
    let content = fs::read_to_string(&target).unwrap();
    assert!(content.contains("description:"));
    // Codex は tools や model を持たない
    assert!(!content.contains("tools:"));
    assert!(!content.contains("model:"));
}

#[test]
fn test_execute_agent_same_format_copies() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("code-review.md");
    let target = temp.path().join("dest/code-review.md");

    fs::write(&source, sample_claude_code_agent_content()).unwrap();

    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Agent,
            name: "code-review".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Project)
        .target_path(&target)
        .source_agent_format(AgentFormat::ClaudeCode)
        .dest_agent_format(AgentFormat::ClaudeCode)
        .build()
        .unwrap();

    let result = deployment.execute().unwrap();

    // コピーのみ（変換なし）
    match result {
        DeploymentOutput::AgentConverted(conv) => {
            assert!(!conv.converted); // converted = false means copy
        }
        _ => panic!("Expected AgentConverted with converted=false"),
    }

    // 内容はそのまま
    assert_eq!(
        fs::read_to_string(&target).unwrap(),
        sample_claude_code_agent_content()
    );
}

#[test]
fn test_execute_agent_without_format_copies() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("agent.md");
    let target = temp.path().join("dest/agent.md");

    fs::write(&source, "simple agent content").unwrap();

    // source_agent_format と dest_agent_format を設定しない
    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Agent,
            name: "agent".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Project)
        .target_path(&target)
        .build()
        .unwrap();

    let result = deployment.execute().unwrap();

    // 変換設定がない場合は Copied
    match result {
        DeploymentOutput::Copied => {}
        _ => panic!("Expected Copied without format settings"),
    }

    assert!(target.exists());
    assert_eq!(fs::read_to_string(&target).unwrap(), "simple agent content");
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
