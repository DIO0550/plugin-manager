//! Unit tests for Antigravity conversion layers and end-to-end convert().

use serde_json::json;

use super::antigravity::{
    AntigravityEventMap, AntigravityKeyMap, AntigravityStructureConverter,
    ANTIGRAVITY_DEFAULT_HOOK_NAME,
};
use super::converter::{
    convert, ConversionWarning, EventMap, KeyMap, SourceFormat, StructureConverter,
};
use crate::target::TargetKind;

#[test]
fn test_event_map_via_converter_module() {
    let map = AntigravityEventMap;
    assert_eq!(map.map_event("PreToolUse"), Some("PreToolUse"));
    assert_eq!(map.map_event("SessionStart"), Some("PreInvocation"));
}

#[test]
fn test_key_map_keeps_command_and_strips_unsupported() {
    let map = AntigravityKeyMap;
    let hook = json!({
        "type": "command",
        "command": "./lint.sh",
        "timeout": 10,
        "async": true,
        "bash": "./wrappers/x.sh",
        "statusMessage": "linting"
    });

    let (mapped, warnings) = map.map_keys(&hook, "command");
    let obj = mapped.as_object().unwrap();

    assert_eq!(obj.get("command").unwrap(), "./lint.sh");
    assert_eq!(obj.get("timeout").unwrap(), 10);
    assert_eq!(obj.get("type").unwrap(), "command");
    assert!(obj.get("async").is_none());
    assert!(obj.get("bash").is_none());
    assert!(obj.get("statusMessage").is_none());
    assert!(warnings
        .iter()
        .any(|w| matches!(w, ConversionWarning::RemovedField { field, .. } if field == "async")));
}

#[test]
fn test_detect_format_claude_code_has_hooks() {
    let conv = AntigravityStructureConverter;
    let value = json!({ "hooks": { "PreToolUse": [] } });
    assert!(matches!(
        conv.detect_format(&value),
        SourceFormat::ClaudeCode
    ));
}

#[test]
fn test_detect_format_native_named_hook_map() {
    let conv = AntigravityStructureConverter;
    let value = json!({
        "my-hook": {
            "PreToolUse": [
                { "matcher": "run_command", "hooks": [{ "type": "command", "command": "./x.sh" }] }
            ]
        }
    });
    assert!(matches!(
        conv.detect_format(&value),
        SourceFormat::TargetFormat
    ));
}

#[test]
fn test_convert_wraps_named_hook_and_remaps_matcher() {
    let input = r#"{
        "hooks": {
            "PreToolUse": [
                {
                    "matcher": "Bash",
                    "hooks": [
                        { "type": "command", "command": "./validate.sh", "timeout": 10 }
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

    let result = convert(input, TargetKind::Antigravity).unwrap();
    assert_eq!(result.source_format, SourceFormat::ClaudeCode);

    let root = result.json.as_object().unwrap();
    assert!(root.get("hooks").is_none());
    let named = root
        .get(ANTIGRAVITY_DEFAULT_HOOK_NAME)
        .and_then(|v| v.as_object())
        .expect("named hook wrapper");

    let pre = named.get("PreToolUse").and_then(|v| v.as_array()).unwrap();
    assert_eq!(pre.len(), 1);
    assert_eq!(pre[0]["matcher"], "run_command");
    assert_eq!(pre[0]["hooks"][0]["command"], "./validate.sh");
    assert_eq!(pre[0]["hooks"][0]["type"], "command");

    // Stop must be flat handlers (not matcher groups).
    let stop = named.get("Stop").and_then(|v| v.as_array()).unwrap();
    assert_eq!(stop.len(), 1);
    assert!(stop[0].get("hooks").is_none());
    assert_eq!(stop[0]["command"], "./on-stop.sh");
    assert_eq!(stop[0]["type"], "command");

    assert!(result.warnings.iter().any(
        |w| matches!(w, ConversionWarning::UnsupportedEvent { event } if event == "Notification")
    ));
}

#[test]
fn test_convert_merges_events_mapping_to_same_target() {
    let input = r#"{
        "hooks": {
            "SessionStart": [
                { "hooks": [{ "type": "command", "command": "./session.sh" }] }
            ],
            "UserPromptSubmit": [
                { "hooks": [{ "type": "command", "command": "./prompt.sh" }] }
            ]
        }
    }"#;

    let result = convert(input, TargetKind::Antigravity).unwrap();
    let named = result.json[ANTIGRAVITY_DEFAULT_HOOK_NAME]
        .as_object()
        .unwrap();
    let pre_inv = named
        .get("PreInvocation")
        .and_then(|v| v.as_array())
        .unwrap();
    assert_eq!(pre_inv.len(), 2);
    assert!(pre_inv.iter().all(|h| h.get("hooks").is_none()));
}

#[test]
fn test_convert_passthrough_native_format() {
    let input = r#"{
        "safety-gate": {
            "enabled": false,
            "PreToolUse": [
                {
                    "matcher": "run_command",
                    "hooks": [{ "type": "command", "command": "./safety.sh" }]
                }
            ]
        }
    }"#;

    let result = convert(input, TargetKind::Antigravity).unwrap();
    assert_eq!(result.source_format, SourceFormat::TargetFormat);
    assert_eq!(result.json["safety-gate"]["enabled"], false);
    assert!(result.scripts.is_empty());
}

#[test]
fn test_convert_excludes_http_and_stub_hooks() {
    let input = r#"{
        "hooks": {
            "PreToolUse": [
                {
                    "matcher": "Bash",
                    "hooks": [
                        { "type": "http", "url": "https://example.com" },
                        { "type": "prompt", "prompt": "ok?" },
                        { "type": "command", "command": "./ok.sh" }
                    ]
                }
            ]
        }
    }"#;

    let result = convert(input, TargetKind::Antigravity).unwrap();
    let hooks = &result.json[ANTIGRAVITY_DEFAULT_HOOK_NAME]["PreToolUse"][0]["hooks"];
    assert_eq!(hooks.as_array().unwrap().len(), 1);
    assert_eq!(hooks[0]["command"], "./ok.sh");
    assert!(result.warnings.iter().any(|w| matches!(
        w,
        ConversionWarning::UnsupportedHookType { hook_type, .. } if hook_type == "http"
    )));
    assert!(result.warnings.iter().any(|w| matches!(
        w,
        ConversionWarning::UnsupportedHookType { hook_type, .. } if hook_type == "prompt"
    )));
}
