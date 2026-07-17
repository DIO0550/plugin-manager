//! Unit tests for Cursor conversion layers and end-to-end convert().

use serde_json::json;

use super::converter::{ConversionWarning, EventMap, KeyMap, StructureConverter};
use super::cursor::{CursorEventMap, CursorKeyMap, CursorStructureConverter};
use crate::hooks::converter;
use crate::target::TargetKind;

// ============================================================================
// EventMap
// ============================================================================

#[test]
fn test_event_map_supported_events() {
    let map = CursorEventMap;
    assert_eq!(map.map_event("SessionStart"), Some("sessionStart"));
    assert_eq!(map.map_event("SessionEnd"), Some("sessionEnd"));
    assert_eq!(map.map_event("PreToolUse"), Some("preToolUse"));
    assert_eq!(map.map_event("PostToolUse"), Some("postToolUse"));
    assert_eq!(
        map.map_event("PostToolUseFailure"),
        Some("postToolUseFailure")
    );
    assert_eq!(
        map.map_event("UserPromptSubmit"),
        Some("beforeSubmitPrompt")
    );
    assert_eq!(map.map_event("Stop"), Some("stop"));
    assert_eq!(map.map_event("SubagentStart"), Some("subagentStart"));
    assert_eq!(map.map_event("SubagentStop"), Some("subagentStop"));
    assert_eq!(map.map_event("PreCompact"), Some("preCompact"));
}

#[test]
fn test_event_map_differs_from_copilot() {
    let map = CursorEventMap;
    // Must not reuse Copilot's agentStop / userPromptSubmitted names.
    assert_ne!(map.map_event("Stop"), Some("agentStop"));
    assert_ne!(
        map.map_event("UserPromptSubmit"),
        Some("userPromptSubmitted")
    );
}

#[test]
fn test_event_map_unsupported_event() {
    let map = CursorEventMap;
    assert_eq!(map.map_event("Notification"), None);
    assert_eq!(map.map_event("UnknownEvent"), None);
}

// ============================================================================
// KeyMap
// ============================================================================

#[test]
fn test_key_map_keeps_command_and_timeout() {
    let map = CursorKeyMap;
    let hook = json!({
        "type": "command",
        "command": "./validate.sh",
        "timeout": 30,
        "async": true,
        "once": true,
        "statusMessage": "Running..."
    });

    let (mapped, warnings) = map.map_keys(&hook, "command");
    let obj = mapped.as_object().unwrap();

    assert_eq!(obj.get("command").unwrap(), "./validate.sh");
    assert_eq!(obj.get("timeout").unwrap(), 30);
    assert_eq!(obj.get("type").unwrap(), "command");
    assert!(obj.get("bash").is_none());
    assert!(obj.get("timeoutSec").is_none());
    assert!(obj.get("async").is_none());
    assert!(obj.get("once").is_none());
    assert!(obj.get("statusMessage").is_none());

    assert!(warnings
        .iter()
        .any(|w| matches!(w, ConversionWarning::RemovedField { field, .. } if field == "async")));
    assert!(warnings
        .iter()
        .any(|w| matches!(w, ConversionWarning::RemovedField { field, .. } if field == "once")));
}

// ============================================================================
// StructureConverter
// ============================================================================

#[test]
fn test_detect_format_with_version() {
    let conv = CursorStructureConverter;
    let value = json!({ "version": 1, "hooks": {} });
    assert!(matches!(
        conv.detect_format(&value),
        super::converter::SourceFormat::TargetFormat
    ));
}

#[test]
fn test_detect_format_pascal_case() {
    let conv = CursorStructureConverter;
    let value = json!({ "hooks": { "SessionStart": [] } });
    assert!(matches!(
        conv.detect_format(&value),
        super::converter::SourceFormat::ClaudeCode
    ));
}

#[test]
fn test_convert_top_level_adds_version() {
    let conv = CursorStructureConverter;
    let value = json!({ "hooks": { "SessionStart": [] }, "disableAllHooks": true });
    let (mapped, warnings) = conv.convert_top_level(&value);
    assert_eq!(mapped.get("version").unwrap(), 1);
    assert!(warnings.iter().any(
        |w| matches!(w, ConversionWarning::RemovedField { field, .. } if field == "disableAllHooks")
    ));
}

// ============================================================================
// End-to-end convert()
// ============================================================================

#[test]
fn test_convert_flattens_and_attaches_mapped_matcher() {
    let input = r#"{
        "hooks": {
            "PreToolUse": [
                {
                    "matcher": "Bash",
                    "hooks": [
                        { "type": "command", "command": "./validate-bash.sh", "timeout": 10 }
                    ]
                },
                {
                    "matcher": "Write|Edit",
                    "hooks": [
                        { "type": "command", "command": "./validate-write.sh" }
                    ]
                }
            ],
            "UserPromptSubmit": [
                {
                    "hooks": [
                        { "type": "command", "command": "./check-prompt.sh" }
                    ]
                }
            ],
            "Stop": [
                {
                    "hooks": [
                        { "type": "command", "command": "./on-stop.sh" }
                    ]
                }
            ],
            "Notification": [
                {
                    "hooks": [
                        { "type": "command", "command": "./notify.sh" }
                    ]
                }
            ]
        }
    }"#;

    let result = converter::convert(input, TargetKind::Cursor).unwrap();
    let hooks = result.json.get("hooks").unwrap();

    assert_eq!(result.json.get("version").unwrap(), 1);
    assert!(result.scripts.is_empty());

    let pre = hooks.get("preToolUse").unwrap().as_array().unwrap();
    assert_eq!(pre.len(), 2);
    assert_eq!(pre[0]["command"], "./validate-bash.sh");
    assert_eq!(pre[0]["matcher"], "Shell");
    assert_eq!(pre[0]["timeout"], 10);
    assert_eq!(pre[1]["command"], "./validate-write.sh");
    assert_eq!(pre[1]["matcher"], "Write");

    assert_eq!(
        hooks["beforeSubmitPrompt"][0]["command"],
        "./check-prompt.sh"
    );
    assert_eq!(hooks["stop"][0]["command"], "./on-stop.sh");
    assert!(hooks.get("Notification").is_none());
    assert!(hooks.get("userPromptSubmitted").is_none());
    assert!(hooks.get("agentStop").is_none());

    assert!(result.warnings.iter().any(|w| matches!(
        w,
        ConversionWarning::UnsupportedEvent { event } if event == "Notification"
    )));
    assert!(!result.warnings.iter().any(|w| matches!(
        w,
        ConversionWarning::RemovedField { field, .. } if field == "matcher"
    )));
}

#[test]
fn test_convert_passthrough_cursor_format() {
    let input = r#"{
        "version": 1,
        "hooks": {
            "preToolUse": [
                { "command": "./already-cursor.sh", "matcher": "Shell" }
            ]
        }
    }"#;

    let result = converter::convert(input, TargetKind::Cursor).unwrap();
    assert_eq!(result.source_format, converter::SourceFormat::TargetFormat);
    assert_eq!(
        result.json["hooks"]["preToolUse"][0]["command"],
        "./already-cursor.sh"
    );
}
