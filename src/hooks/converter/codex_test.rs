//! Unit tests for Codex hook conversion layers.

use serde_json::json;

use super::super::event::codex::CodexEventMap;
use super::super::model::{CommandHook, HttpHook};
use super::codex::{CodexKeyMap, CodexScriptGenerator, CodexStructureConverter};
use super::converter::{
    convert, ConversionWarning, EventMap, KeyMap, ScriptGenerator, SourceFormat, StructureConverter,
};
use crate::target::TargetKind;

#[test]
fn test_codex_event_map_keeps_supported_events() {
    let map = CodexEventMap;
    assert_eq!(map.map_event("SessionStart"), Some("SessionStart"));
    assert_eq!(map.map_event("PreToolUse"), Some("PreToolUse"));
    assert_eq!(map.map_event("PostToolUse"), Some("PostToolUse"));
    assert_eq!(map.map_event("UserPromptSubmit"), Some("UserPromptSubmit"));
    assert_eq!(map.map_event("Stop"), Some("Stop"));
    assert_eq!(
        map.map_event("PermissionRequest"),
        Some("PermissionRequest")
    );
    assert_eq!(map.map_event("SubagentStop"), None);
}

#[test]
fn test_codex_key_map_keeps_command_fields() {
    let map = CodexKeyMap;
    let hook = json!({"type": "command", "command": "echo hi", "timeout": 10});
    let (mapped, warnings) = map.map_keys(&hook, "command");
    assert_eq!(mapped, hook);
    assert!(warnings.is_empty());
}

#[test]
fn test_codex_key_map_removes_unsupported_fields() {
    let map = CodexKeyMap;
    let hook = json!({
        "type": "command",
        "command": "echo hi",
        "async": true,
        "once": true,
        "bash": "./wrappers/hook.sh",
        "timeoutSec": 3,
        "comment": "checking"
    });

    let (mapped, warnings) = map.map_keys(&hook, "command");

    assert_eq!(mapped["type"], "command");
    assert_eq!(mapped["command"], "echo hi");
    assert_eq!(mapped["timeout"], 3);
    assert_eq!(mapped["statusMessage"], "checking");
    assert!(mapped.get("async").is_none());
    assert!(mapped.get("once").is_none());
    assert!(mapped.get("bash").is_none());
    assert_eq!(warnings.len(), 3);
}

#[test]
fn test_codex_key_map_prefers_timeout_over_timeout_sec() {
    let map = CodexKeyMap;
    let hook = json!({
        "type": "command",
        "command": "echo hi",
        "timeout": 10,
        "timeoutSec": 3
    });

    let (mapped, warnings) = map.map_keys(&hook, "command");

    assert_eq!(mapped["timeout"], 10);
    assert!(mapped.get("timeoutSec").is_none());
    assert!(warnings.is_empty());
}

#[test]
fn test_codex_structure_converter_detects_claude_code() {
    let conv = CodexStructureConverter;
    let value = json!({"hooks": {"SessionStart": []}});
    assert!(matches!(
        conv.detect_format(&value),
        SourceFormat::ClaudeCode
    ));
}

#[test]
fn test_codex_structure_converter_removes_non_codex_top_level_fields() {
    let conv = CodexStructureConverter;
    let value = json!({"version": 1, "disableAllHooks": true, "hooks": {"SessionStart": []}});
    let (result, warnings) = conv.convert_top_level(&value);
    assert!(result.get("version").is_none());
    assert!(result.get("disableAllHooks").is_none());
    assert_eq!(result["hooks"], json!({"SessionStart": []}));
    assert_eq!(warnings.len(), 2);
}

#[test]
fn test_codex_script_generator_command_returns_inline_marker() {
    let gen = CodexScriptGenerator;
    let hook_value = json!({"type": "command", "command": "echo hi"});
    let hook_obj = hook_value.as_object().unwrap();
    let command = CommandHook::new(hook_obj, &hook_value).unwrap();
    let result = gen.generate_command_script(&command, "SessionStart", None, 0);
    assert!(result.path.is_empty());
    assert!(result.content.is_empty());
    assert_eq!(result.original_config, hook_value);
}

#[test]
fn test_codex_script_generator_http_returns_inline_exclusion_marker() {
    let gen = CodexScriptGenerator;
    let hook_value = json!({"type": "http", "url": "https://example.com"});
    let hook_obj = hook_value.as_object().unwrap();
    let http = HttpHook::new(hook_obj, &hook_value).unwrap();
    let result = gen.generate_http_script(&http, "sessionStart", None, 0);
    let info = result.unwrap();
    assert!(info.path.is_empty());
}

#[test]
fn test_codex_convert_keeps_command_hook_inline() {
    let input = r#"{
        "hooks": {
            "PreToolUse": [
                {
                    "matcher": "Bash",
                    "hooks": [
                        {"type": "command", "command": "echo hi", "timeout": 5}
                    ]
                }
            ]
        }
    }"#;

    let result = convert(input, TargetKind::Codex).unwrap();

    assert!(result.scripts.is_empty());
    assert_eq!(result.json["hooks"]["PreToolUse"][0]["matcher"], "Bash");
    assert_eq!(
        result.json["hooks"]["PreToolUse"][0]["hooks"][0]["type"],
        "command"
    );
    assert_eq!(
        result.json["hooks"]["PreToolUse"][0]["hooks"][0]["command"],
        "echo hi"
    );
    assert!(result.json["hooks"]["PreToolUse"][0]["hooks"][0]
        .get("bash")
        .is_none());
}

#[test]
fn test_codex_convert_excludes_unsupported_hook_types() {
    let input = r#"{
        "hooks": {
            "PreToolUse": [
                {
                    "matcher": "Bash",
                    "hooks": [
                        {"type": "http", "url": "https://example.com"},
                        {"type": "prompt", "prompt": "check this"}
                    ]
                }
            ]
        }
    }"#;

    let result = convert(input, TargetKind::Codex).unwrap();

    assert!(result.scripts.is_empty());
    assert!(result.json["hooks"].as_object().unwrap().is_empty());
    assert!(result.warnings.iter().any(|w| matches!(
        w,
        ConversionWarning::UnsupportedHookType { hook_type, .. } if hook_type == "http"
    )));
    assert!(result.warnings.iter().any(|w| matches!(
        w,
        ConversionWarning::UnsupportedHookType { hook_type, .. } if hook_type == "prompt"
    )));
}
