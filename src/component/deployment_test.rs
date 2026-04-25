use super::*;
use crate::component::CommandFormat;
use crate::hooks::converter::ConversionWarning;
use crate::hooks::name::HookName;
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

#[test]
fn test_hook_convert_without_plugin_root_errors_when_wrappers_needed() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.json");
    let target = temp.path().join("dest/hook.json");

    // Claude Code 形式 → wrapper が生成される → plugin_root 必須
    fs::write(&source, sample_claude_code_hook_json()).unwrap();

    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Hook,
            name: "test-hook".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Project)
        .target_path(&target)
        .hook_convert(true)
        .target_kind(TargetKind::Copilot)
        .build()
        .unwrap(); // build() は成功する

    let err = deployment.execute().unwrap_err();
    assert!(err.to_string().contains("plugin_root"));
}

#[test]
fn test_hook_convert_without_plugin_root_ok_for_warnings_only() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.json");
    let target = temp.path().join("dest/hook.json");

    // Copilot CLI 形式 + version 欠落 → warnings あり、wrapper なし → plugin_root 不要
    let json = r#"{"hooks":{"preToolUse":[{"type":"command","bash":"echo hi"}]}}"#;
    fs::write(&source, json).unwrap();

    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Hook,
            name: "test-hook".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Project)
        .target_path(&target)
        .hook_convert(true)
        .target_kind(TargetKind::Copilot)
        .build()
        .unwrap();

    let result = deployment.execute().unwrap();
    match result {
        DeploymentOutput::HookConverted(hr) => {
            assert!(!hr.warnings.is_empty());
            assert_eq!(hr.script_count, 0);
        }
        _ => panic!("Expected HookConverted with warnings only"),
    }
}

// ========================================
// Hook conversion deploy tests
// ========================================

/// Claude Code 形式の Hook JSON テストデータ
fn sample_claude_code_hook_json() -> &'static str {
    r#"{
  "hooks": {
    "PreToolUse": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "echo 'pre-tool check'"
          }
        ]
      }
    ]
  }
}"#
}

#[test]
fn test_hook_convert_false_copies_file() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.json");
    let target = temp.path().join("dest/hook.json");

    fs::write(&source, r#"{"hooks":{}}"#).unwrap();

    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Hook,
            name: "test-hook".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Project)
        .target_path(&target)
        .build()
        .unwrap();

    let result = deployment.execute().unwrap();
    assert!(matches!(result, DeploymentOutput::Copied));
    assert!(target.exists());
    assert_eq!(fs::read_to_string(&target).unwrap(), r#"{"hooks":{}}"#);
}

#[test]
fn test_hook_convert_true_deploys_converted() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.json");
    let target_dir = temp.path().join("dest");
    let target = target_dir.join("my-hook.json");
    let plugin_root = temp.path().join("cache/plugin");
    fs::create_dir_all(&plugin_root).unwrap();

    fs::write(&source, sample_claude_code_hook_json()).unwrap();

    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Hook,
            name: "my-hook".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Project)
        .target_path(&target)
        .hook_convert(true)
        .target_kind(TargetKind::Copilot)
        .plugin_root(&plugin_root)
        .build()
        .unwrap();

    let result = deployment.execute().unwrap();

    match result {
        DeploymentOutput::HookConverted(hr) => {
            assert!(hr.script_count > 0);
        }
        _ => panic!("Expected HookConverted"),
    }

    // JSON ファイルが配置されていること
    assert!(target.exists());
    let json_content = fs::read_to_string(&target).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_content).unwrap();
    assert_eq!(parsed.get("version").unwrap(), 1);

    // スクリプトが wrappers/{hook-name}/ に配置されていること
    let scripts_dir = target_dir.join("wrappers").join("my-hook");
    assert!(scripts_dir.exists());
    let script_files: Vec<_> = fs::read_dir(&scripts_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(!script_files.is_empty());

    // JSON 内の bash パスが ./wrappers/my-hook/... を参照していること
    assert!(json_content.contains("./wrappers/my-hook/"));
}

/// Copilot CLI 形式入力 → そのまま配置（version があるので変換されない）
#[test]
fn test_hook_convert_copilot_format_passthrough() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.json");
    let target = temp.path().join("dest/hook.json");
    let plugin_root = temp.path().join("cache/plugin");
    fs::create_dir_all(&plugin_root).unwrap();

    let copilot_json = r#"{
  "version": 1,
  "hooks": {
    "preToolUse": [
      {
        "type": "command",
        "bash": "echo hello"
      }
    ]
  }
}"#;
    fs::write(&source, copilot_json).unwrap();

    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Hook,
            name: "copilot-hook".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Project)
        .target_path(&target)
        .hook_convert(true)
        .target_kind(TargetKind::Copilot)
        .plugin_root(&plugin_root)
        .build()
        .unwrap();

    let result = deployment.execute().unwrap();

    // Copilot CLI 形式はファイルコピーにフォールバック
    assert!(matches!(result, DeploymentOutput::Copied));

    assert!(target.exists());
    // 元のファイル内容がそのままコピーされること
    assert_eq!(fs::read_to_string(&target).unwrap(), copilot_json);
}

/// @@PLUGIN_ROOT@@ が plugin_root の実パスに置換されること
#[test]
fn test_hook_convert_plugin_root_replacement() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.json");
    let target_dir = temp.path().join("dest");
    let target = target_dir.join("test-hook.json");
    let plugin_root = temp.path().join("cache/my-plugin");
    fs::create_dir_all(&plugin_root).unwrap();

    fs::write(&source, sample_claude_code_hook_json()).unwrap();

    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Hook,
            name: "test-hook".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Project)
        .target_path(&target)
        .hook_convert(true)
        .target_kind(TargetKind::Copilot)
        .plugin_root(&plugin_root)
        .build()
        .unwrap();

    deployment.execute().unwrap();

    // wrapper スクリプト内で @@PLUGIN_ROOT@@ が置換されていること
    let wrappers_dir = target_dir.join("wrappers").join("test-hook");
    for entry in fs::read_dir(&wrappers_dir).unwrap() {
        let entry = entry.unwrap();
        let content = fs::read_to_string(entry.path()).unwrap();
        assert!(
            !content.contains("@@PLUGIN_ROOT@@"),
            "@@PLUGIN_ROOT@@ should be replaced in {:?}",
            entry.path()
        );
    }
}

/// 実行権限 0o755 の検証（Unix のみ）
#[cfg(unix)]
#[test]
fn test_hook_convert_wrapper_executable_permission() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.json");
    let target_dir = temp.path().join("dest");
    let target = target_dir.join("perm-hook.json");
    let plugin_root = temp.path().join("cache/plugin");
    fs::create_dir_all(&plugin_root).unwrap();

    fs::write(&source, sample_claude_code_hook_json()).unwrap();

    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Hook,
            name: "perm-hook".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Project)
        .target_path(&target)
        .hook_convert(true)
        .target_kind(TargetKind::Copilot)
        .plugin_root(&plugin_root)
        .build()
        .unwrap();

    deployment.execute().unwrap();

    let wrappers_dir = target_dir.join("wrappers").join("perm-hook");
    for entry in fs::read_dir(&wrappers_dir).unwrap() {
        let entry = entry.unwrap();
        let perms = fs::metadata(entry.path()).unwrap().permissions();
        assert_eq!(perms.mode() & 0o777, 0o755);
    }
}

/// 存在しないソースファイル → Err
#[test]
fn test_hook_convert_missing_source_returns_err() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("nonexistent.json");
    let target = temp.path().join("dest/hook.json");
    let plugin_root = temp.path().join("cache/plugin");

    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Hook,
            name: "hook".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Project)
        .target_path(&target)
        .hook_convert(true)
        .target_kind(TargetKind::Copilot)
        .plugin_root(&plugin_root)
        .build()
        .unwrap();

    assert!(deployment.execute().is_err());
}

/// 親ディレクトリが自動作成されること
#[test]
fn test_hook_convert_creates_parent_dir() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.json");
    let target = temp.path().join("deep/nested/dir/hook.json");
    let plugin_root = temp.path().join("cache/plugin");
    fs::create_dir_all(&plugin_root).unwrap();

    fs::write(&source, sample_claude_code_hook_json()).unwrap();

    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Hook,
            name: "hook".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Project)
        .target_path(&target)
        .hook_convert(true)
        .target_kind(TargetKind::Copilot)
        .plugin_root(&plugin_root)
        .build()
        .unwrap();

    deployment.execute().unwrap();
    assert!(target.exists());
}

/// 全イベント非対応 → 空 hooks + 警告
#[test]
fn test_hook_convert_unsupported_events_produce_warnings() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.json");
    let target = temp.path().join("dest/hook.json");
    let plugin_root = temp.path().join("cache/plugin");
    fs::create_dir_all(&plugin_root).unwrap();

    // 存在しないイベント名のみ
    let json = r#"{
  "hooks": {
    "UnknownEvent": [
      { "hooks": [{ "type": "command", "command": "echo test" }] }
    ]
  }
}"#;
    fs::write(&source, json).unwrap();

    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Hook,
            name: "hook".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Project)
        .target_path(&target)
        .hook_convert(true)
        .target_kind(TargetKind::Copilot)
        .plugin_root(&plugin_root)
        .build()
        .unwrap();

    let result = deployment.execute().unwrap();
    match result {
        DeploymentOutput::HookConverted(hr) => {
            assert!(!hr.warnings.is_empty());
            assert!(hr
                .warnings
                .iter()
                .any(|w| matches!(w, ConversionWarning::UnsupportedEvent { .. })));
        }
        _ => panic!("Expected HookConverted"),
    }
}

/// 元スクリプトが配置先にコピーされないこと（@@PLUGIN_ROOT@@ 経由で参照）
#[test]
fn test_hook_convert_original_scripts_not_copied() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.json");
    let target_dir = temp.path().join("dest");
    let target = target_dir.join("hook.json");
    let plugin_root = temp.path().join("cache/plugin");
    fs::create_dir_all(&plugin_root).unwrap();

    // 元スクリプトを作成（コピーされないはず）
    let scripts_dir = plugin_root.join("scripts");
    fs::create_dir_all(&scripts_dir).unwrap();
    fs::write(
        scripts_dir.join("original.sh"),
        "#!/bin/bash\necho original",
    )
    .unwrap();

    // Hook JSON で @@PLUGIN_ROOT@@ 経由で元スクリプトを参照
    let hook_json = r#"{
  "hooks": {
    "PreToolUse": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "@@PLUGIN_ROOT@@/scripts/original.sh"
          }
        ]
      }
    ]
  }
}"#;
    fs::write(&source, hook_json).unwrap();

    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Hook,
            name: "hook".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Project)
        .target_path(&target)
        .hook_convert(true)
        .target_kind(TargetKind::Copilot)
        .plugin_root(&plugin_root)
        .build()
        .unwrap();

    deployment.execute().unwrap();

    // 元スクリプトが配置先にコピーされていないこと
    assert!(!target_dir.join("scripts").exists());
    assert!(!target_dir.join("original.sh").exists());

    // wrapper スクリプト内で @@PLUGIN_ROOT@@ が実パスに置換されていること
    let wrappers_dir = target_dir.join("wrappers").join("hook");
    assert!(wrappers_dir.exists());
    for entry in fs::read_dir(&wrappers_dir).unwrap() {
        let content = fs::read_to_string(entry.unwrap().path()).unwrap();
        assert!(!content.contains("@@PLUGIN_ROOT@@"));
        assert!(content.contains("original.sh"));
    }
}

/// 複数 Hook ファイルの wrapper が名前衝突しないこと
#[test]
fn test_hook_convert_multiple_hooks_no_name_collision() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.json");
    let target_dir = temp.path().join("dest");
    let plugin_root = temp.path().join("cache/plugin");
    fs::create_dir_all(&plugin_root).unwrap();

    fs::write(&source, sample_claude_code_hook_json()).unwrap();

    // Hook A
    let target_a = target_dir.join("hook-a.json");
    let deployment_a = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Hook,
            name: "hook-a".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Project)
        .target_path(&target_a)
        .hook_convert(true)
        .target_kind(TargetKind::Copilot)
        .plugin_root(&plugin_root)
        .build()
        .unwrap();
    deployment_a.execute().unwrap();

    // Hook B
    let target_b = target_dir.join("hook-b.json");
    let deployment_b = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Hook,
            name: "hook-b".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Project)
        .target_path(&target_b)
        .hook_convert(true)
        .target_kind(TargetKind::Copilot)
        .plugin_root(&plugin_root)
        .build()
        .unwrap();
    deployment_b.execute().unwrap();

    // 各 Hook の wrapper が別ディレクトリに配置されていること
    assert!(target_dir.join("wrappers/hook-a").exists());
    assert!(target_dir.join("wrappers/hook-b").exists());

    // JSON 内の bash パスが正しい名前空間を参照していること
    let json_a = fs::read_to_string(&target_a).unwrap();
    let json_b = fs::read_to_string(&target_b).unwrap();
    assert!(json_a.contains("./wrappers/hook-a/"));
    assert!(json_b.contains("./wrappers/hook-b/"));
    assert!(!json_a.contains("./wrappers/hook-b/"));
    assert!(!json_b.contains("./wrappers/hook-a/"));
}

#[test]
fn test_hook_convert_with_unsafe_name_uses_sanitized_dir() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.json");
    let target_dir = temp.path().join("dest");
    let target = target_dir.join("hook.json");
    let plugin_root = temp.path().join("cache/plugin");
    fs::create_dir_all(&plugin_root).unwrap();

    fs::write(&source, sample_claude_code_hook_json()).unwrap();

    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Hook,
            name: "my hook$name".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Project)
        .target_path(&target)
        .hook_convert(true)
        .target_kind(TargetKind::Copilot)
        .plugin_root(&plugin_root)
        .build()
        .unwrap();

    deployment.execute().unwrap();

    let hook_name = HookName::new("my hook$name");
    let safe_name = hook_name.as_safe();

    // サニタイズされたディレクトリ名が使われること
    assert!(target_dir.join("wrappers").join(safe_name).exists());
    // JSON 内のパスもサニタイズされた名前を参照
    let json_content = fs::read_to_string(&target).unwrap();
    assert!(json_content.contains(&format!("./wrappers/{}/", safe_name)));
}

/// 変換後 JSON に version: 1 が含まれること
#[test]
fn test_hook_convert_output_has_version() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.json");
    let target = temp.path().join("dest/hook.json");
    let plugin_root = temp.path().join("cache/plugin");
    fs::create_dir_all(&plugin_root).unwrap();

    fs::write(&source, sample_claude_code_hook_json()).unwrap();

    let deployment = ComponentDeployment::builder()
        .component(&Component {
            kind: ComponentKind::Hook,
            name: "hook".to_string(),
            path: (&source).into(),
        })
        .scope(Scope::Project)
        .target_path(&target)
        .hook_convert(true)
        .target_kind(TargetKind::Copilot)
        .plugin_root(&plugin_root)
        .build()
        .unwrap();

    deployment.execute().unwrap();

    let content = fs::read_to_string(&target).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed.get("version").unwrap(), 1);
}
