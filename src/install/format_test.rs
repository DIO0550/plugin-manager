use super::*;
use crate::component::ComponentKind;
use crate::hooks::converter::{ConversionWarning, SourceFormat};
use owo_colors::OwoColorize;
use std::collections::BTreeSet;

// ============================================================================
// should_show_converted_suffix（3 分岐網羅）
// ============================================================================

#[test]
fn should_show_converted_suffix_returns_true_for_claude_code() {
    assert!(should_show_converted_suffix(Some(SourceFormat::ClaudeCode)));
}

#[test]
fn should_show_converted_suffix_returns_false_for_target_format() {
    // Copilot 形式 passthrough のときに suffix を出さない（false positive 防止）
    assert!(!should_show_converted_suffix(Some(
        SourceFormat::TargetFormat
    )));
}

#[test]
fn should_show_converted_suffix_returns_false_for_none() {
    // Hook 以外（hook_source_format == None）では suffix を出さない
    assert!(!should_show_converted_suffix(None));
}

// ============================================================================
// format_converted_hook_suffix
// ============================================================================

#[test]
fn format_converted_hook_suffix_returns_cyan_decorated_label() {
    let actual = format_converted_hook_suffix();
    let expected = format!(" {}", "(converted from Claude Code format)".cyan());
    assert_eq!(actual, expected);
    assert!(actual.contains("(converted from Claude Code format)"));
    // 先頭スペースは `+` 行末に直接連結する想定
    assert!(actual.starts_with(' '));
}

// ============================================================================
// classify_hook_warnings
// ============================================================================

#[test]
fn classify_hook_warnings_collects_unsupported_event_into_skipped() {
    let warnings = vec![ConversionWarning::UnsupportedEvent {
        event: "Notification".to_string(),
    }];
    let classified = classify_hook_warnings(&warnings);
    assert_eq!(classified.skipped.len(), 1);
    assert!(classified.skipped.contains("Notification"));
    assert!(classified.stubs.is_empty());
    assert!(classified.others.is_empty());
}

#[test]
fn classify_hook_warnings_routes_unsupported_hook_type_to_others() {
    // `UnsupportedHookType` は「イベント内の特定フックのみ除外」されたケースで発生する。
    // イベント全体がスキップされたかのような誤解を防ぐため、`skipped` には入れず
    // `others` に流して個別 Warning として `Display` の正確な文言で出す。
    let warnings = vec![ConversionWarning::UnsupportedHookType {
        hook_type: "weird".to_string(),
        event: "PreCompact".to_string(),
    }];
    let classified = classify_hook_warnings(&warnings);
    assert!(classified.skipped.is_empty());
    assert!(classified.stubs.is_empty());
    assert_eq!(classified.others.len(), 1);
    assert!(matches!(
        classified.others[0],
        ConversionWarning::UnsupportedHookType { .. }
    ));
}

#[test]
fn classify_hook_warnings_dedupes_unsupported_events_via_btreeset() {
    // `UnsupportedEvent` のみ 3 件（重複あり）→ BTreeSet で 2 件に一意化、アルファベット順。
    // `UnsupportedHookType` は別ルートに行くため、ここでは混ぜない。
    let warnings = vec![
        ConversionWarning::UnsupportedEvent {
            event: "Foo".to_string(),
        },
        ConversionWarning::UnsupportedEvent {
            event: "Foo".to_string(),
        },
        ConversionWarning::UnsupportedEvent {
            event: "Bar".to_string(),
        },
    ];
    let classified = classify_hook_warnings(&warnings);
    assert_eq!(classified.skipped.len(), 2);
    let ordered: Vec<&String> = classified.skipped.iter().collect();
    assert_eq!(ordered[0], "Bar");
    assert_eq!(ordered[1], "Foo");
}

#[test]
fn classify_hook_warnings_separates_unsupported_event_from_unsupported_hook_type() {
    // 同じ event 名でも、`UnsupportedEvent` は skipped、`UnsupportedHookType` は others に
    // 分かれることを固定する。これにより「イベント全体除外」と「イベント内の一部除外」が
    // 出力上も明確に区別される。
    let warnings = vec![
        ConversionWarning::UnsupportedEvent {
            event: "PreToolUse".to_string(),
        },
        ConversionWarning::UnsupportedHookType {
            hook_type: "weird".to_string(),
            event: "PreToolUse".to_string(),
        },
    ];
    let classified = classify_hook_warnings(&warnings);
    assert_eq!(classified.skipped.len(), 1);
    assert!(classified.skipped.contains("PreToolUse"));
    assert_eq!(classified.others.len(), 1);
    assert!(matches!(
        classified.others[0],
        ConversionWarning::UnsupportedHookType { .. }
    ));
}

#[test]
fn classify_hook_warnings_collects_prompt_agent_stubs_in_input_order() {
    let warnings = vec![
        ConversionWarning::PromptAgentHookStub {
            hook_type: "prompt".to_string(),
            event: "preToolUse".to_string(),
        },
        ConversionWarning::PromptAgentHookStub {
            hook_type: "agent".to_string(),
            event: "postToolUse".to_string(),
        },
    ];
    let classified = classify_hook_warnings(&warnings);
    assert_eq!(classified.stubs.len(), 2);
    assert_eq!(classified.stubs[0].0, "prompt");
    assert_eq!(classified.stubs[0].1, "preToolUse");
    assert_eq!(classified.stubs[1].0, "agent");
    assert_eq!(classified.stubs[1].1, "postToolUse");
    assert!(classified.skipped.is_empty());
    assert!(classified.others.is_empty());
}

#[test]
fn classify_hook_warnings_routes_others_for_removed_field_and_missing_version() {
    let warnings = vec![
        ConversionWarning::RemovedField {
            field: "matcher".to_string(),
            reason: "Copilot CLI does not support matchers".to_string(),
        },
        ConversionWarning::MissingVersion,
    ];
    let classified = classify_hook_warnings(&warnings);
    assert_eq!(classified.others.len(), 2);
    assert!(classified.skipped.is_empty());
    assert!(classified.stubs.is_empty());
}

// ============================================================================
// format_skipped_events_warning
// ============================================================================

#[test]
fn format_skipped_events_warning_returns_none_for_empty() {
    let events: BTreeSet<String> = BTreeSet::new();
    assert!(format_skipped_events_warning(&events).is_none());
}

#[test]
fn format_skipped_events_warning_uses_singular_for_one_event() {
    // user-facing CLI の英文として「1 events」は誤りなので、件数 1 のとき "event" を使う。
    let mut events = BTreeSet::new();
    events.insert("Notification".to_string());
    let actual = format_skipped_events_warning(&events).unwrap();
    let expected = format!(
        "  {} 1 event skipped (not supported in Copilot CLI): Notification",
        "Warning:".yellow()
    );
    assert_eq!(actual, expected);
}

#[test]
fn format_skipped_events_warning_lists_three_events_alphabetically() {
    let mut events = BTreeSet::new();
    events.insert("PreCompact".to_string());
    events.insert("Notification".to_string());
    events.insert("SubagentStart".to_string());
    let actual = format_skipped_events_warning(&events).unwrap();
    let expected = format!(
        "  {} 3 events skipped (not supported in Copilot CLI): Notification, PreCompact, SubagentStart",
        "Warning:".yellow()
    );
    assert_eq!(actual, expected);
}

// ============================================================================
// format_manual_rewrite_section
// ============================================================================

#[test]
fn format_manual_rewrite_section_returns_none_for_empty() {
    assert!(format_manual_rewrite_section(&[]).is_none());
}

#[test]
fn format_manual_rewrite_section_uses_singular_for_one_stub() {
    // user-facing CLI の英文として「(1 hooks)」は誤りなので、件数 1 のときは "hook"。
    let stubs = vec![("prompt".to_string(), "preToolUse".to_string())];
    let actual = format_manual_rewrite_section(&stubs).unwrap();
    let header = "Manual rewrite required (1 hook):";
    assert!(actual.contains(&header.magenta().bold().to_string()));
    assert!(!actual.contains("(1 hooks)"));
}

#[test]
fn format_manual_rewrite_section_renders_header_lines_and_note() {
    let stubs = vec![
        ("prompt".to_string(), "preToolUse".to_string()),
        ("agent".to_string(), "postToolUse".to_string()),
    ];
    let actual = format_manual_rewrite_section(&stubs).unwrap();
    let header = "Manual rewrite required (2 hooks):";
    assert!(actual.contains(&header.magenta().bold().to_string()));
    assert!(actual.contains("    - 'prompt' hook for event 'preToolUse'"));
    assert!(actual.contains("    - 'agent' hook for event 'postToolUse'"));
    assert!(
        actual.contains("  Note: stub scripts have been generated; please rewrite them manually.")
    );
    // 行構成: 見出し + 2 stub 行 + Note 行 = 4 行
    assert_eq!(actual.lines().count(), 4);
}

// ============================================================================
// format_empty_hooks_warning
// ============================================================================

#[test]
fn format_empty_hooks_warning_some_for_claude_code_with_zero_scripts() {
    let actual = format_empty_hooks_warning(0, Some(SourceFormat::ClaudeCode)).unwrap();
    let expected = format!(
        "  {} An empty hooks.json was placed; no hooks remained after conversion.",
        "Warning:".yellow()
    );
    assert_eq!(actual, expected);
}

#[test]
fn format_empty_hooks_warning_none_when_scripts_present() {
    // 明示的な hook が 1 件以上残っているケース
    assert!(format_empty_hooks_warning(2, Some(SourceFormat::ClaudeCode)).is_none());
}

#[test]
fn format_empty_hooks_warning_none_for_target_format_passthrough() {
    // TargetFormat passthrough は入力 JSON が保持されるので空配置警告対象外（false positive 防止）
    assert!(format_empty_hooks_warning(0, Some(SourceFormat::TargetFormat)).is_none());
}

#[test]
fn format_empty_hooks_warning_none_when_source_format_is_none() {
    // Hook 以外、または Copied 経路の Hook（version 付き Copilot passthrough）はこの警告対象外
    assert!(format_empty_hooks_warning(0, None).is_none());
}

// ============================================================================
// format_individual_warning
// ============================================================================

#[test]
fn format_individual_warning_decorates_removed_field() {
    let warning = ConversionWarning::RemovedField {
        field: "matcher".to_string(),
        reason: "Copilot CLI does not support matchers".to_string(),
    };
    let actual = format_individual_warning(&warning);
    let expected = format!("  {} {}", "Warning:".yellow(), warning);
    assert_eq!(actual, expected);
    assert!(actual.contains("matcher"));
}

#[test]
fn format_individual_warning_decorates_missing_version() {
    let warning = ConversionWarning::MissingVersion;
    let actual = format_individual_warning(&warning);
    let expected = format!("  {} {}", "Warning:".yellow(), warning);
    assert_eq!(actual, expected);
}

// ============================================================================
// render_hook_success（副作用なしレンダラ）
// ============================================================================

#[test]
fn render_hook_success_claude_code_with_unsupported_events_emits_suffix_and_one_block() {
    let warnings = vec![
        ConversionWarning::UnsupportedEvent {
            event: "Notification".to_string(),
        },
        ConversionWarning::UnsupportedEvent {
            event: "PreCompact".to_string(),
        },
        ConversionWarning::UnsupportedEvent {
            event: "SubagentStart".to_string(),
        },
    ];
    let rendered = render_hook_success(
        ComponentKind::Hook,
        Some(SourceFormat::ClaudeCode),
        &warnings,
        1,
        1,
    );
    assert!(rendered.stdout_suffix.is_some());
    assert_eq!(rendered.stderr_blocks.len(), 1);
    assert!(rendered.stderr_blocks[0].contains("3 events skipped"));
}

#[test]
fn render_hook_success_copilot_format_with_missing_version_no_suffix_only_individual_warning() {
    // Copilot 形式 + MissingVersion 1 件 → suffix なし、stderr_blocks に individual warning のみ
    let warnings = vec![ConversionWarning::MissingVersion];
    let rendered = render_hook_success(
        ComponentKind::Hook,
        Some(SourceFormat::TargetFormat),
        &warnings,
        0,
        0,
    );
    assert!(rendered.stdout_suffix.is_none());
    assert_eq!(rendered.stderr_blocks.len(), 1);
    assert_eq!(
        rendered.stderr_blocks[0],
        format_individual_warning(&ConversionWarning::MissingVersion)
    );
}

#[test]
fn render_hook_success_skill_returns_empty_output() {
    let rendered = render_hook_success(
        ComponentKind::Skill,
        Some(SourceFormat::ClaudeCode),
        &[ConversionWarning::MissingVersion],
        1,
        1,
    );
    assert!(rendered.stdout_suffix.is_none());
    assert!(rendered.stderr_blocks.is_empty());
}

#[test]
fn render_hook_success_all_events_skipped_includes_empty_hooks_warning() {
    let warnings = vec![
        ConversionWarning::UnsupportedEvent {
            event: "Notification".to_string(),
        },
        ConversionWarning::UnsupportedEvent {
            event: "PreCompact".to_string(),
        },
    ];
    let rendered = render_hook_success(
        ComponentKind::Hook,
        Some(SourceFormat::ClaudeCode),
        &warnings,
        0,
        0,
    );
    assert!(rendered.stdout_suffix.is_some());
    // skipped events warning + empty hooks warning の 2 ブロック
    assert_eq!(rendered.stderr_blocks.len(), 2);
    assert!(rendered.stderr_blocks[0].contains("2 events skipped"));
    assert!(rendered.stderr_blocks[1].contains("no hooks remained after conversion"));
}

#[test]
fn render_hook_success_inline_hooks_do_not_emit_empty_hooks_warning() {
    let rendered = render_hook_success(
        ComponentKind::Hook,
        Some(SourceFormat::ClaudeCode),
        &[],
        0,
        1,
    );
    assert!(rendered.stdout_suffix.is_some());
    assert!(
        !rendered
            .stderr_blocks
            .iter()
            .any(|b| b.contains("no hooks remained after conversion")),
        "inline Codex hooks should not be treated as an empty hooks.json"
    );
}

#[test]
fn render_hook_success_all_unsupported_hook_types_emits_empty_hooks_warning() {
    // `UnsupportedHookType` で event 内の全 hook が落ちると、`convert_event_hooks` は
    // その event ごと JSON から落とす（`UnsupportedEvent` は出ない）。この経路では
    // `skipped_count == 0` だが `script_count == 0` で空 `hooks.json` が配置されるため、
    // 空配置警告を出す必要がある。
    let warnings = vec![
        ConversionWarning::UnsupportedHookType {
            hook_type: "weird".to_string(),
            event: "PreToolUse".to_string(),
        },
        ConversionWarning::UnsupportedHookType {
            hook_type: "weird".to_string(),
            event: "PostToolUse".to_string(),
        },
    ];
    let rendered = render_hook_success(
        ComponentKind::Hook,
        Some(SourceFormat::ClaudeCode),
        &warnings,
        0,
        0,
    );
    assert!(rendered.stdout_suffix.is_some());
    // 空配置警告 + 個別 warning 2 件 = 3 ブロック
    assert_eq!(rendered.stderr_blocks.len(), 3);
    assert!(
        rendered
            .stderr_blocks
            .iter()
            .any(|b| b.contains("no hooks remained after conversion")),
        "empty hooks warning should appear when all hooks are filtered as UnsupportedHookType"
    );
    let individual_count = rendered
        .stderr_blocks
        .iter()
        .filter(|b| b.contains("Hook type 'weird' for event"))
        .count();
    assert_eq!(individual_count, 2);
}

#[test]
fn render_hook_success_only_removed_field_warnings_still_emits_empty_hooks_warning() {
    // `RemovedField` 経路でも空 `hooks.json` が配置され得る:
    // `flatten_matchers` がイベント値を配列として読めなかった、または matcher group の
    // `hooks` 配列が欠落していた場合（`src/hooks/converter.rs::flatten_matchers`）に
    // `RemovedField` のみが出力され、`UnsupportedEvent` / `UnsupportedHookType` は出ない。
    // ClaudeCode 入力の `script_count == 0` という不変条件で取りこぼさないことを固定する。
    let warnings = vec![
        ConversionWarning::RemovedField {
            field: "PreToolUse".to_string(),
            reason: "Event 'PreToolUse' value is not an array; expected matcher group structure"
                .to_string(),
        },
        ConversionWarning::RemovedField {
            field: "hooks".to_string(),
            reason: "Matcher group in event 'PostToolUse' is missing 'hooks' array; skipped"
                .to_string(),
        },
    ];
    let rendered = render_hook_success(
        ComponentKind::Hook,
        Some(SourceFormat::ClaudeCode),
        &warnings,
        0,
        0,
    );
    assert!(rendered.stdout_suffix.is_some());
    // 空配置警告 + 個別 warning 2 件 = 3 ブロック
    assert_eq!(rendered.stderr_blocks.len(), 3);
    assert!(
        rendered
            .stderr_blocks
            .iter()
            .any(|b| b.contains("no hooks remained after conversion")),
        "empty hooks warning should appear even when only RemovedField warnings are emitted"
    );
}

#[test]
fn render_hook_success_target_format_with_zero_scripts_does_not_emit_empty_hooks_warning() {
    // TargetFormat passthrough は入力 JSON がそのまま配置されるので `script_count == 0`
    // でも空配置警告は出してはいけない（false positive 防止）。
    let warnings = vec![ConversionWarning::MissingVersion];
    let rendered = render_hook_success(
        ComponentKind::Hook,
        Some(SourceFormat::TargetFormat),
        &warnings,
        0,
        0,
    );
    assert!(rendered.stdout_suffix.is_none());
    assert!(
        !rendered
            .stderr_blocks
            .iter()
            .any(|b| b.contains("no hooks remained after conversion")),
        "empty hooks warning must NOT appear for TargetFormat passthrough"
    );
}

#[test]
fn render_hook_success_hook_with_none_source_format_and_no_warnings_returns_empty() {
    // version 付き Copilot 形式 Hook は DeploymentOutput::Copied 経路で
    // hook_source_format == None / warnings 0 になる。既存挙動の固定。
    let rendered = render_hook_success(ComponentKind::Hook, None, &[], 0, 0);
    assert!(rendered.stdout_suffix.is_none());
    assert!(rendered.stderr_blocks.is_empty());
}

#[test]
fn render_hook_success_prompt_agent_stub_emits_manual_rewrite_section() {
    // PromptAgentHookStub → classify_hook_warnings → render_hook_success の配線回帰を固定。
    // 受入基準 3 (issue #190): "Manual rewrite required (N hooks):" セクションが
    // stderr ブロックに 1 件入ることを確認する。
    let warnings = vec![
        ConversionWarning::PromptAgentHookStub {
            hook_type: "prompt".to_string(),
            event: "preToolUse".to_string(),
        },
        ConversionWarning::PromptAgentHookStub {
            hook_type: "agent".to_string(),
            event: "postToolUse".to_string(),
        },
    ];
    let rendered = render_hook_success(
        ComponentKind::Hook,
        Some(SourceFormat::ClaudeCode),
        &warnings,
        2,
        2,
    );
    assert!(rendered.stdout_suffix.is_some());
    assert_eq!(rendered.stderr_blocks.len(), 1);
    assert!(rendered.stderr_blocks[0].contains("Manual rewrite required (2 hooks):"));
    assert!(rendered.stderr_blocks[0].contains("'prompt' hook for event 'preToolUse'"));
    assert!(rendered.stderr_blocks[0].contains("'agent' hook for event 'postToolUse'"));
    assert!(rendered.stderr_blocks[0].contains("Note: stub scripts have been generated"));
}
