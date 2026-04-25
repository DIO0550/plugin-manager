use super::*;
use crate::component::convert::{AgentFormat, ConversionResult};
use crate::component::{CommandFormat, Component, ComponentKind, ConversionConfig, Scope};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn make_deployment(
    component: Component,
    target: PathBuf,
    conversion: ConversionConfig,
) -> ComponentDeployment {
    ComponentDeployment::builder()
        .component(component)
        .scope(Scope::Project)
        .target_path(target)
        .conversion(conversion)
        .build()
        .unwrap()
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

    let deployment = make_deployment(
        Component {
            kind: ComponentKind::Agent,
            name: "test-agent".to_string(),
            path: source,
        },
        target.clone(),
        ConversionConfig::None,
    );

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

    let deployment = make_deployment(
        Component {
            kind: ComponentKind::Command,
            name: "test-cmd".to_string(),
            path: source,
        },
        target.clone(),
        ConversionConfig::None,
    );

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
        .component(Component {
            kind: ComponentKind::Instruction,
            name: "test-instruction".to_string(),
            path: source,
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
        .component(Component {
            kind: ComponentKind::Hook,
            name: "test-hook".to_string(),
            path: source,
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

    let deployment = make_deployment(
        Component {
            kind: ComponentKind::Skill,
            name: "my-skill".to_string(),
            path: source,
        },
        target.clone(),
        ConversionConfig::None,
    );

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

    let deployment = make_deployment(
        Component {
            kind: ComponentKind::Skill,
            name: "skill".to_string(),
            path: source,
        },
        target.clone(),
        ConversionConfig::None,
    );

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
        .component(Component {
            kind: ComponentKind::Agent,
            name: "test".to_string(),
            path: PathBuf::from("/src/test.md"),
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

    let deployment = make_deployment(
        Component {
            kind: ComponentKind::Command,
            name: "commit".to_string(),
            path: source,
        },
        target.clone(),
        ConversionConfig::Command {
            source: CommandFormat::ClaudeCode,
            dest: CommandFormat::Copilot,
        },
    );

    let result = deployment.execute().unwrap();

    // 変換が行われたことを確認
    match result {
        DeploymentOutput::CommandConverted(ConversionResult {
            converted,
            source_format,
            dest_format,
        }) => {
            assert!(converted);
            assert_eq!(source_format, CommandFormat::ClaudeCode);
            assert_eq!(dest_format, CommandFormat::Copilot);
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

    let deployment = make_deployment(
        Component {
            kind: ComponentKind::Command,
            name: "commit".to_string(),
            path: source,
        },
        target.clone(),
        ConversionConfig::Command {
            source: CommandFormat::ClaudeCode,
            dest: CommandFormat::ClaudeCode,
        },
    );

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

    // ConversionConfig::None ならフォーマット指定なしで Copied
    let deployment = make_deployment(
        Component {
            kind: ComponentKind::Command,
            name: "commit".to_string(),
            path: source,
        },
        target.clone(),
        ConversionConfig::None,
    );

    let result = deployment.execute().unwrap();

    match result {
        DeploymentOutput::Copied => {}
        _ => panic!("Expected Copied without conversion"),
    }

    assert!(target.exists());
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

    let deployment = make_deployment(
        Component {
            kind: ComponentKind::Agent,
            name: "code-review".to_string(),
            path: source,
        },
        target.clone(),
        ConversionConfig::Agent {
            source: AgentFormat::ClaudeCode,
            dest: AgentFormat::Copilot,
        },
    );

    let result = deployment.execute().unwrap();

    match result {
        DeploymentOutput::AgentConverted(conv) => {
            assert!(conv.converted);
            assert_eq!(conv.source_format, AgentFormat::ClaudeCode);
            assert_eq!(conv.dest_format, AgentFormat::Copilot);
        }
        _ => panic!("Expected AgentConverted"),
    }

    let content = fs::read_to_string(&target).unwrap();
    assert!(content.contains("tools:"));
    assert!(content.contains("codebase"));
    assert!(content.contains("GPT-4o"));
    assert!(content.contains("target: vscode"));
}

#[test]
fn test_execute_agent_with_conversion_to_codex() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("code-review.md");
    let target = temp.path().join("dest/code-review.agent.md");

    fs::write(&source, sample_claude_code_agent_content()).unwrap();

    let deployment = make_deployment(
        Component {
            kind: ComponentKind::Agent,
            name: "code-review".to_string(),
            path: source,
        },
        target.clone(),
        ConversionConfig::Agent {
            source: AgentFormat::ClaudeCode,
            dest: AgentFormat::Codex,
        },
    );

    let result = deployment.execute().unwrap();

    match result {
        DeploymentOutput::AgentConverted(conv) => {
            assert!(conv.converted);
            assert_eq!(conv.source_format, AgentFormat::ClaudeCode);
            assert_eq!(conv.dest_format, AgentFormat::Codex);
        }
        _ => panic!("Expected AgentConverted"),
    }

    let content = fs::read_to_string(&target).unwrap();
    assert!(content.contains("description:"));
    assert!(!content.contains("tools:"));
    assert!(!content.contains("model:"));
}

#[test]
fn test_execute_agent_same_format_copies() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("code-review.md");
    let target = temp.path().join("dest/code-review.md");

    fs::write(&source, sample_claude_code_agent_content()).unwrap();

    let deployment = make_deployment(
        Component {
            kind: ComponentKind::Agent,
            name: "code-review".to_string(),
            path: source,
        },
        target.clone(),
        ConversionConfig::Agent {
            source: AgentFormat::ClaudeCode,
            dest: AgentFormat::ClaudeCode,
        },
    );

    let result = deployment.execute().unwrap();

    match result {
        DeploymentOutput::AgentConverted(conv) => {
            assert!(!conv.converted);
        }
        _ => panic!("Expected AgentConverted with converted=false"),
    }

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

    let deployment = make_deployment(
        Component {
            kind: ComponentKind::Agent,
            name: "agent".to_string(),
            path: source,
        },
        target.clone(),
        ConversionConfig::None,
    );

    let result = deployment.execute().unwrap();

    match result {
        DeploymentOutput::Copied => {}
        _ => panic!("Expected Copied without conversion"),
    }

    assert!(target.exists());
    assert_eq!(fs::read_to_string(&target).unwrap(), "simple agent content");
}
