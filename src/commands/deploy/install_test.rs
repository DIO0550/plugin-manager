use super::*;
use crate::component::ComponentKind;
use crate::hooks::converter::{ConversionWarning, SourceFormat};
use crate::install::PlaceSuccess;
use std::path::PathBuf;

fn make_success(
    component_kind: ComponentKind,
    component_name: &str,
    target: &str,
    target_path: &str,
    source_format: Option<&str>,
    dest_format: Option<&str>,
    hook_warnings: Vec<ConversionWarning>,
    script_count: usize,
    hook_source_format: Option<SourceFormat>,
) -> PlaceSuccess {
    PlaceSuccess {
        target: target.to_string(),
        component_name: component_name.to_string(),
        component_kind,
        target_path: PathBuf::from(target_path),
        source_format: source_format.map(|s| s.to_string()),
        dest_format: dest_format.map(|s| s.to_string()),
        hook_warnings,
        script_count,
        hook_count: script_count,
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
        None,
    );
    let (stdout, stderr) = render_place_success_to_strings(&success);
    assert!(!stdout.contains("(Converted:"));
    assert!(!stdout.contains("(converted from Claude Code format)"));
    assert!(stderr.is_empty());
}
