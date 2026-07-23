use super::*;
use crate::component::convert::{AgentFormat, ConversionOutcome};
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
        Component::new(ComponentKind::Agent, "test-agent".to_string(), source),
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
        Component::new(ComponentKind::Command, "test-cmd".to_string(), source),
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
        .component(Component::new(
            ComponentKind::Instruction,
            "test-instruction".to_string(),
            source,
        ))
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
        .component(Component::new(
            ComponentKind::Hook,
            "test-hook".to_string(),
            source,
        ))
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
        Component::new(ComponentKind::Skill, "my-skill".to_string(), source),
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
        Component::new(ComponentKind::Skill, "skill".to_string(), source),
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
        .component(Component::new(
            ComponentKind::Agent,
            "test".to_string(),
            PathBuf::from("/src/test.md"),
        ))
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
        Component::new(ComponentKind::Command, "commit".to_string(), source),
        target.clone(),
        ConversionConfig::Command {
            source: CommandFormat::ClaudeCode,
            dest: CommandFormat::Copilot,
        },
    );

    let result = deployment.execute().unwrap();

    // 変換が行われたことを確認
    match result {
        DeploymentOutput::CommandConverted(ConversionOutcome {
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
        Component::new(ComponentKind::Command, "commit".to_string(), source),
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
        Component::new(ComponentKind::Command, "commit".to_string(), source),
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
        Component::new(ComponentKind::Agent, "code-review".to_string(), source),
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
        Component::new(ComponentKind::Agent, "code-review".to_string(), source),
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
        Component::new(ComponentKind::Agent, "code-review".to_string(), source),
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
        Component::new(ComponentKind::Agent, "agent".to_string(), source),
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

// ========================================
// Skill frontmatter conversion tests
// ========================================

#[test]
fn test_execute_skill_strips_unsupported_frontmatter_for_codex() {
    use crate::target::TargetKind;

    let temp = TempDir::new().unwrap();
    let source = temp.path().join("my-skill");
    let target = temp.path().join("dest/my-skill");

    fs::create_dir(&source).unwrap();
    fs::write(
        source.join("SKILL.md"),
        "---\nname: my-skill\ndescription: demo\ndisable-model-invocation: true\nallowed-tools: Bash(ls *)\nargument-hint: [a] [b]\n---\n\n# Body\n",
    )
    .unwrap();
    fs::write(source.join("helper.py"), "print('hi')").unwrap();

    let deployment = make_deployment(
        Component::new(ComponentKind::Skill, "my-skill".to_string(), source),
        target.clone(),
        ConversionConfig::Skill {
            target_kind: TargetKind::Codex,
        },
    );

    deployment.execute().unwrap();

    let manifest = fs::read_to_string(target.join("SKILL.md")).unwrap();
    assert!(manifest.contains("name: my-skill"));
    assert!(manifest.contains("description: demo"));
    assert!(!manifest.contains("disable-model-invocation"));
    assert!(!manifest.contains("allowed-tools"));
    assert!(!manifest.contains("argument-hint"));
    assert!(manifest.contains("# Body"));
    // 付随ファイルはそのままコピーされる
    assert!(target.join("helper.py").exists());
}

#[test]
fn test_execute_skill_keeps_frontmatter_for_non_codex_target() {
    use crate::target::TargetKind;

    let temp = TempDir::new().unwrap();
    let source = temp.path().join("my-skill");
    let target = temp.path().join("dest/my-skill");

    let original = "---\nname: my-skill\ndescription: demo\nallowed-tools: Bash(ls *)\n---\nbody\n";
    fs::create_dir(&source).unwrap();
    fs::write(source.join("SKILL.md"), original).unwrap();

    let deployment = make_deployment(
        Component::new(ComponentKind::Skill, "my-skill".to_string(), source),
        target.clone(),
        ConversionConfig::Skill {
            target_kind: TargetKind::Copilot,
        },
    );

    deployment.execute().unwrap();

    let manifest = fs::read_to_string(target.join("SKILL.md")).unwrap();
    assert_eq!(manifest, original);
}

// ========================================
// Skill bundled resources (#392)
// ========================================

fn write_skill_with_bundled_resources(source: &Path) {
    fs::create_dir_all(source.join("references")).unwrap();
    fs::create_dir_all(source.join("assets/templates")).unwrap();
    fs::write(
        source.join("SKILL.md"),
        "---\nname: my-skill\ndescription: demo\nallowed-tools: Bash(ls *)\nmetadata:\n  short-description: short\n---\n\n# Body\nSee ./references/a.md\n",
    )
    .unwrap();
    fs::write(source.join("notes.md"), "# Notes\n").unwrap();
    fs::write(
        source.join("references/a.md"),
        "---\nname: should-not-strip\nallowed-tools: Bash(*)\n---\nref body\n",
    )
    .unwrap();
    fs::write(
        source.join("assets/templates/x.html"),
        "<html>template</html>",
    )
    .unwrap();
}

fn assert_bundled_resources_copied(target: &Path) {
    assert!(target.join("SKILL.md").is_file());
    assert!(target.join("notes.md").is_file());
    assert_eq!(
        fs::read_to_string(target.join("notes.md")).unwrap(),
        "# Notes\n"
    );
    assert_eq!(
        fs::read_to_string(target.join("references/a.md")).unwrap(),
        "---\nname: should-not-strip\nallowed-tools: Bash(*)\n---\nref body\n"
    );
    assert_eq!(
        fs::read_to_string(target.join("assets/templates/x.html")).unwrap(),
        "<html>template</html>"
    );
}

#[test]
fn test_execute_skill_copies_bundled_resources_same_structure() {
    use crate::target::TargetKind;

    let skill_targets = [
        TargetKind::Codex,
        TargetKind::Copilot,
        TargetKind::Antigravity,
        TargetKind::GeminiCli,
        TargetKind::Cursor,
    ];

    for target_kind in skill_targets {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("my-skill");
        let target = temp.path().join("dest/my-skill");
        write_skill_with_bundled_resources(&source);

        let deployment = make_deployment(
            Component::new(ComponentKind::Skill, "my-skill".to_string(), source),
            target.clone(),
            ConversionConfig::Skill { target_kind },
        );

        deployment.execute().unwrap();
        assert_bundled_resources_copied(&target);
    }
}

#[test]
fn test_execute_skill_strip_does_not_touch_bundled_markdown() {
    use crate::target::TargetKind;

    // Codex / Gemini CLI は frontmatter strip あり。付属 md は不変であること。
    for target_kind in [TargetKind::Codex, TargetKind::GeminiCli] {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("my-skill");
        let target = temp.path().join("dest/my-skill");
        write_skill_with_bundled_resources(&source);

        let bundled_before = fs::read_to_string(source.join("references/a.md")).unwrap();

        let deployment = make_deployment(
            Component::new(ComponentKind::Skill, "my-skill".to_string(), source),
            target.clone(),
            ConversionConfig::Skill { target_kind },
        );

        deployment.execute().unwrap();

        let manifest = fs::read_to_string(target.join("SKILL.md")).unwrap();
        assert!(
            !manifest.contains("allowed-tools"),
            "{target_kind:?} should strip unsupported fields from SKILL.md"
        );
        assert_eq!(
            fs::read_to_string(target.join("references/a.md")).unwrap(),
            bundled_before,
            "{target_kind:?} must not rewrite bundled markdown"
        );
        // strip 対象は target_path/SKILL.md のみ（付属パスに SKILL.md は作られない）
        assert!(target.join("SKILL.md").is_file());
        assert!(!target.join("references/SKILL.md").exists());
    }
}

#[test]
fn test_execute_skill_replace_dir_removes_stale_bundled_resources() {
    use crate::target::TargetKind;

    let temp = TempDir::new().unwrap();
    let source = temp.path().join("my-skill");
    let target = temp.path().join("dest/my-skill");

    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("SKILL.md"), "# Skill\n").unwrap();
    fs::create_dir_all(source.join("references")).unwrap();
    fs::write(source.join("references/keep.md"), "keep\n").unwrap();

    // 前回デプロイの残骸
    fs::create_dir_all(target.join("references")).unwrap();
    fs::write(target.join("references/stale.md"), "stale\n").unwrap();
    fs::create_dir_all(target.join("assets/old")).unwrap();
    fs::write(target.join("assets/old/gone.html"), "gone\n").unwrap();
    fs::write(target.join("obsolete.md"), "obsolete\n").unwrap();

    let deployment = make_deployment(
        Component::new(ComponentKind::Skill, "my-skill".to_string(), source),
        target.clone(),
        ConversionConfig::Skill {
            target_kind: TargetKind::Copilot,
        },
    );

    deployment.execute().unwrap();

    assert!(target.join("SKILL.md").is_file());
    assert!(target.join("references/keep.md").is_file());
    assert!(!target.join("references/stale.md").exists());
    assert!(!target.join("assets/old/gone.html").exists());
    assert!(!target.join("obsolete.md").exists());
}

// ========================================
// MockFs deployment tests
// ========================================

#[test]
fn test_execute_skill_with_mock_fs_replaces_existing_directory() {
    use crate::fs::mock::MockFs;

    let fs = MockFs::new();
    fs.add_dir("/src/skill");
    fs.add_file("/src/skill/new.md", "new");
    fs.add_dir("/dst/skill");
    fs.add_file("/dst/skill/old.md", "old");

    let deployment = make_deployment(
        Component::new(
            ComponentKind::Skill,
            "skill".to_string(),
            PathBuf::from("/src/skill"),
        ),
        PathBuf::from("/dst/skill"),
        ConversionConfig::None,
    );

    deployment.execute_with_fs(&fs).unwrap();

    assert!(fs.exists(Path::new("/dst/skill/new.md")));
    assert!(!fs.exists(Path::new("/dst/skill/old.md")));
}

#[test]
fn test_execute_agent_with_mock_fs_copies_file() {
    use crate::fs::mock::MockFs;

    let fs = MockFs::new();
    fs.add_file("/src/agent.md", "agent content");

    let deployment = make_deployment(
        Component::new(
            ComponentKind::Agent,
            "test-agent".to_string(),
            PathBuf::from("/src/agent.md"),
        ),
        PathBuf::from("/dest/agent.md"),
        ConversionConfig::None,
    );

    deployment.execute_with_fs(&fs).unwrap();

    assert_eq!(
        fs.read_to_string(Path::new("/dest/agent.md")).unwrap(),
        "agent content"
    );
}
