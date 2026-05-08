use super::*;
use crate::component::ComponentKind;
use crate::hooks::converter::{ConversionWarning, SourceFormat};
use crate::install::{PlaceFailure, PlaceFailureStage, PlaceResult, PlaceSuccess};
use crate::target::TargetKind;
use std::path::PathBuf;
use tempfile::TempDir;

fn target_kind_from_name(name: &str) -> TargetKind {
    match name {
        "antigravity" => TargetKind::Antigravity,
        "codex" => TargetKind::Codex,
        "copilot" => TargetKind::Copilot,
        "gemini" => TargetKind::GeminiCli,
        other => panic!("unknown target name in test fixture: {}", other),
    }
}

fn make_success(
    component_kind: ComponentKind,
    component_name: &str,
    target: &str,
    target_path: &str,
    source_format: Option<&str>,
    dest_format: Option<&str>,
    hook_warnings: Vec<ConversionWarning>,
    script_count: usize,
    hook_count: usize,
    hook_source_format: Option<SourceFormat>,
) -> PlaceSuccess {
    PlaceSuccess {
        target: target.to_string(),
        target_kind: target_kind_from_name(target),
        component_name: component_name.to_string(),
        component_kind,
        target_path: PathBuf::from(target_path),
        source_format: source_format.map(|s| s.to_string()),
        dest_format: dest_format.map(|s| s.to_string()),
        hook_warnings,
        script_count,
        hook_count,
        hook_source_format,
    }
}

#[test]
fn render_place_success_hook_claude_code_with_unsupported_events() {
    let success = make_success(
        ComponentKind::Hook,
        "my-hook",
        "copilot",
        "/dest/copilot/hooks/my-hook.json",
        None,
        None,
        vec![
            ConversionWarning::UnsupportedEvent {
                event: "Notification".to_string(),
            },
            ConversionWarning::UnsupportedEvent {
                event: "PreCompact".to_string(),
            },
            ConversionWarning::UnsupportedEvent {
                event: "SubagentStart".to_string(),
            },
        ],
        1,
        1,
        Some(SourceFormat::ClaudeCode),
    );
    let (stdout, stderr) = render_place_success_to_strings(&success);
    assert!(stdout.contains("(converted from Claude Code format)"));
    assert!(stdout.starts_with("  + copilot Hook: my-hook -> "));
    assert_eq!(stderr.len(), 1);
    assert!(stderr[0].contains("3 events skipped"));
}

#[test]
fn render_place_success_hook_target_format_with_missing_version() {
    let success = make_success(
        ComponentKind::Hook,
        "copilot-hook",
        "copilot",
        "/dest/copilot/hooks/copilot-hook.json",
        None,
        None,
        vec![ConversionWarning::MissingVersion],
        0,
        0,
        Some(SourceFormat::TargetFormat),
    );
    let (stdout, stderr) = render_place_success_to_strings(&success);
    // Copilot 形式 passthrough なので suffix なし、source/dest_format も None で空 suffix
    assert!(!stdout.contains("(converted from Claude Code format)"));
    assert!(!stdout.contains("(Converted:"));
    assert_eq!(stderr.len(), 1);
    assert!(stderr[0].contains("Warning:"));
    assert!(stderr[0].contains("missing required 'version'"));
}

#[test]
fn render_place_success_hook_passthrough_copied_no_warnings() {
    // version 付き Copilot 形式 Hook → DeploymentOutput::Copied 経路
    // → hook_source_format == None, warnings 0 件
    let success = make_success(
        ComponentKind::Hook,
        "copilot-hook",
        "copilot",
        "/dest/copilot/hooks/copilot-hook.json",
        None,
        None,
        vec![],
        0,
        0,
        None,
    );
    let (stdout, stderr) = render_place_success_to_strings(&success);
    assert!(!stdout.contains("(converted from Claude Code format)"));
    assert!(!stdout.contains("(Converted:"));
    assert!(stderr.is_empty());
}

#[test]
fn render_place_success_command_keeps_legacy_converted_suffix() {
    let success = make_success(
        ComponentKind::Command,
        "test-plugin_my-cmd",
        "copilot",
        "/dest/copilot/commands/my-cmd.prompt.md",
        Some("ClaudeCode"),
        Some("Copilot"),
        vec![],
        0,
        0,
        None,
    );
    let (stdout, stderr) = render_place_success_to_strings(&success);
    assert!(stdout.contains("(Converted: ClaudeCode → Copilot)"));
    assert!(!stdout.contains("(converted from Claude Code format)"));
    assert!(stderr.is_empty());
}

#[test]
fn render_place_success_agent_keeps_legacy_converted_suffix() {
    let success = make_success(
        ComponentKind::Agent,
        "test-plugin_my-agent",
        "copilot",
        "/dest/copilot/agents/my-agent.agent.md",
        Some("ClaudeCode"),
        Some("Copilot"),
        vec![],
        0,
        0,
        None,
    );
    let (stdout, stderr) = render_place_success_to_strings(&success);
    assert!(stdout.contains("(Converted: ClaudeCode → Copilot)"));
    assert!(!stdout.contains("(converted from Claude Code format)"));
    assert!(stderr.is_empty());
}

#[test]
fn render_place_success_skill_no_suffix_no_stderr() {
    let success = make_success(
        ComponentKind::Skill,
        "test-plugin_my-skill",
        "codex",
        "/dest/codex/skills/my-skill",
        None,
        None,
        vec![],
        0,
        0,
        None,
    );
    let (stdout, stderr) = render_place_success_to_strings(&success);
    assert!(!stdout.contains("(Converted:"));
    assert!(!stdout.contains("(converted from Claude Code format)"));
    assert!(stderr.is_empty());
}

#[test]
fn render_place_success_instruction_no_suffix_no_stderr() {
    // 受入基準 4 (issue #190): Instruction の既存出力に変更がないことを固定する
    let success = make_success(
        ComponentKind::Instruction,
        "test-plugin_AGENTS",
        "codex",
        "/dest/codex/AGENTS.md",
        None,
        None,
        vec![],
        0,
        0,
        None,
    );
    let (stdout, stderr) = render_place_success_to_strings(&success);
    assert!(!stdout.contains("(Converted:"));
    assert!(!stdout.contains("(converted from Claude Code format)"));
    assert!(stderr.is_empty());
}

#[test]
fn make_success_keeps_hook_count_independent_from_script_count() {
    let success = make_success(
        ComponentKind::Hook,
        "codex-hook",
        "codex",
        "/dest/codex/hooks.json",
        None,
        None,
        vec![],
        0,
        1,
        Some(SourceFormat::ClaudeCode),
    );

    assert_eq!(success.script_count, 0);
    assert_eq!(success.hook_count, 1);
}

#[test]
fn update_status_after_install_marks_successful_targets_enabled() {
    let temp = TempDir::new().unwrap();
    let result = PlaceResult {
        plugin_name: "test-plugin".to_string(),
        successes: vec![make_success(
            ComponentKind::Hook,
            "test-plugin_hooks",
            "codex",
            "/dest/codex/hooks.json",
            None,
            None,
            vec![],
            0,
            1,
            Some(SourceFormat::ClaudeCode),
        )],
        failures: vec![],
    };

    crate::install::update_meta_after_place(temp.path(), &result);

    let plugin_meta = crate::plugin::meta::load_meta(temp.path()).unwrap();
    assert_eq!(plugin_meta.get_status("codex"), Some("enabled"));
    // Hook + Codex は所有権としても記録される
    assert!(plugin_meta.manages_file("codex", std::path::Path::new("/dest/codex/hooks.json")));
}

#[test]
fn update_meta_after_place_skips_managed_file_for_non_hook_codex_success() {
    // Skill のみ Codex に配置されたケースは statusByTarget は enabled になるが、
    // managedFiles には記録されない（hook_overwrite_error が誤って通過しないため）。
    let temp = TempDir::new().unwrap();
    let result = PlaceResult {
        plugin_name: "test-plugin".to_string(),
        successes: vec![make_success(
            ComponentKind::Skill,
            "test-plugin_skill",
            "codex",
            "/dest/codex/skills/test-plugin_skill",
            None,
            None,
            vec![],
            0,
            0,
            None,
        )],
        failures: vec![],
    };

    crate::install::update_meta_after_place(temp.path(), &result);

    let plugin_meta = crate::plugin::meta::load_meta(temp.path()).unwrap();
    assert_eq!(plugin_meta.get_status("codex"), Some("enabled"));
    assert!(
        !plugin_meta.manages_file("codex", std::path::Path::new("/dest/codex/hooks.json")),
        "Skill 配置のみで hooks.json の所有権を獲得してはならない"
    );
    assert!(plugin_meta
        .managed_files
        .get("codex")
        .is_none_or(|v| v.is_empty()));
}

#[test]
fn update_status_after_install_skips_targets_with_failures() {
    let temp = TempDir::new().unwrap();
    let result = PlaceResult {
        plugin_name: "test-plugin".to_string(),
        successes: vec![make_success(
            ComponentKind::Hook,
            "test-plugin_hooks",
            "codex",
            "/dest/codex/hooks.json",
            None,
            None,
            vec![],
            0,
            1,
            Some(SourceFormat::ClaudeCode),
        )],
        failures: vec![PlaceFailure {
            target: "codex".to_string(),
            component_name: "test-plugin_skill".to_string(),
            component_kind: ComponentKind::Skill,
            error: "failed".to_string(),
            stage: PlaceFailureStage::Deployment,
        }],
    };

    crate::install::update_meta_after_place(temp.path(), &result);

    // 同 target 内に failure があるとステータス更新は発生しないため、
    // .plm-meta.json は新規作成されない（不要な書き込みを避ける）。
    assert!(
        crate::plugin::meta::load_meta(temp.path()).is_none(),
        "failed-target install must not create .plm-meta.json"
    );
}

#[test]
fn update_status_after_install_skips_write_when_all_failed() {
    // 全 target が失敗（successes が空）の場合、.plm-meta.json を書き換えない。
    let temp = TempDir::new().unwrap();
    let result = PlaceResult {
        plugin_name: "test-plugin".to_string(),
        successes: vec![],
        failures: vec![PlaceFailure {
            target: "codex".to_string(),
            component_name: "test-plugin_hook".to_string(),
            component_kind: ComponentKind::Hook,
            error: "failed".to_string(),
            stage: PlaceFailureStage::Deployment,
        }],
    };

    crate::install::update_meta_after_place(temp.path(), &result);

    assert!(
        crate::plugin::meta::load_meta(temp.path()).is_none(),
        "fully-failed install must not create .plm-meta.json"
    );
}
