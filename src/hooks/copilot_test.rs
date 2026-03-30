//! Unit tests for Copilot conversion layers.

use serde_json::json;

use super::converter::{ConversionWarning, EventMap, KeyMap, ScriptGenerator, StructureConverter};
use super::copilot::{
    CopilotEventMap, CopilotKeyMap, CopilotScriptGenerator, CopilotStructureConverter,
};
use super::hook_definition::{CommandHook, HttpHook, StubHook};

// ============================================================================
// EventMap
// ============================================================================

#[test]
fn test_event_map_supported_events() {
    let map = CopilotEventMap;
    assert_eq!(map.map_event("SessionStart"), Some("sessionStart"));
    assert_eq!(map.map_event("PreToolUse"), Some("preToolUse"));
    assert_eq!(map.map_event("PostToolUse"), Some("postToolUse"));
    assert_eq!(map.map_event("Stop"), Some("agentStop"));
}

#[test]
fn test_event_map_unsupported_event() {
    let map = CopilotEventMap;
    assert_eq!(map.map_event("PreCompact"), None);
    assert_eq!(map.map_event("UnknownEvent"), None);
}

// ============================================================================
// KeyMap
// ============================================================================

#[test]
fn test_key_map_command_hook() {
    let map = CopilotKeyMap;
    let hook = json!({
        "type": "command",
        "command": "echo hi",
        "timeout": 30,
        "statusMessage": "Running...",
        "async": true,
        "once": true,
        "customField": "keep"
    });

    let (mapped, warnings) = map.map_keys(&hook, "command");
    let obj = mapped.as_object().unwrap();

    assert_eq!(obj.get("timeoutSec").unwrap(), 30);
    assert_eq!(obj.get("comment").unwrap(), "Running...");
    assert_eq!(obj.get("customField").unwrap(), "keep");
    assert!(obj.get("command").is_none());
    assert!(obj.get("type").is_none());
    assert!(obj.get("timeout").is_none());
    assert!(obj.get("statusMessage").is_none());
    assert!(obj.get("async").is_none());
    assert!(obj.get("once").is_none());

    assert!(warnings
        .iter()
        .any(|w| matches!(w, ConversionWarning::RemovedField { field, .. } if field == "async")));
    assert!(warnings
        .iter()
        .any(|w| matches!(w, ConversionWarning::RemovedField { field, .. } if field == "once")));
}

#[test]
fn test_key_map_http_hook() {
    let map = CopilotKeyMap;
    let hook = json!({
        "type": "http",
        "url": "https://example.com",
        "method": "POST",
        "headers": { "Authorization": "Bearer token" },
        "timeout": 15,
        "statusMessage": "Calling..."
    });

    let (mapped, warnings) = map.map_keys(&hook, "http");
    let obj = mapped.as_object().unwrap();

    assert_eq!(obj.get("timeoutSec").unwrap(), 15);
    assert_eq!(obj.get("comment").unwrap(), "Calling...");
    assert!(obj.get("type").is_none());
    assert!(obj.get("url").is_none());
    assert!(obj.get("method").is_none());
    assert!(obj.get("headers").is_none());
    assert!(warnings.is_empty());
}

#[test]
fn test_key_map_prompt_hook() {
    let map = CopilotKeyMap;
    let hook = json!({
        "type": "prompt",
        "prompt": "Review code",
        "timeout": 20,
        "statusMessage": "Reviewing..."
    });

    let (mapped, warnings) = map.map_keys(&hook, "prompt");
    let obj = mapped.as_object().unwrap();

    assert_eq!(obj.get("timeoutSec").unwrap(), 20);
    assert_eq!(obj.get("comment").unwrap(), "Reviewing...");
    assert!(obj.get("type").is_none());
    assert!(obj.get("prompt").is_none());
    assert!(warnings.is_empty());
}

// ============================================================================
// StructureConverter
// ============================================================================

#[test]
fn test_detect_format_with_version() {
    let conv = CopilotStructureConverter;
    let value = json!({ "version": 1, "hooks": {} });
    assert!(matches!(
        conv.detect_format(&value),
        super::converter::SourceFormat::TargetFormat
    ));
}

#[test]
fn test_detect_format_pascal_case() {
    let conv = CopilotStructureConverter;
    let value = json!({ "hooks": { "SessionStart": [] } });
    assert!(matches!(
        conv.detect_format(&value),
        super::converter::SourceFormat::ClaudeCode
    ));
}

#[test]
fn test_detect_format_camel_case() {
    let conv = CopilotStructureConverter;
    let value = json!({ "hooks": { "sessionStart": [] } });
    assert!(matches!(
        conv.detect_format(&value),
        super::converter::SourceFormat::TargetFormat
    ));
}

#[test]
fn test_handle_target_format_adds_version() {
    let conv = CopilotStructureConverter;
    let value = json!({ "hooks": { "sessionStart": [] } });
    let (result, warnings) = conv.handle_target_format(value).unwrap();
    assert_eq!(result["version"], 1);
    assert!(warnings
        .iter()
        .any(|w| matches!(w, ConversionWarning::MissingVersion)));
}

#[test]
fn test_handle_target_format_with_version() {
    let conv = CopilotStructureConverter;
    let value = json!({ "version": 1, "hooks": {} });
    let (result, warnings) = conv.handle_target_format(value).unwrap();
    assert_eq!(result["version"], 1);
    assert!(warnings.is_empty());
}

#[test]
fn test_convert_top_level_adds_version_removes_disable() {
    let conv = CopilotStructureConverter;
    let value = json!({ "disableAllHooks": true, "hooks": { "SessionStart": [] } });
    let (result, warnings) = conv.convert_top_level(&value);
    assert_eq!(result["version"], 1);
    assert!(result.get("disableAllHooks").is_none());
    assert!(warnings.iter().any(|w| matches!(
        w,
        ConversionWarning::RemovedField { field, .. } if field == "disableAllHooks"
    )));
}

// ============================================================================
// ScriptGenerator
// ============================================================================

#[test]
fn test_generate_command_script() {
    let gen = CopilotScriptGenerator;
    let hook_value = json!({"type": "command", "command": "echo hello"});
    let hook_obj = hook_value.as_object().unwrap();
    let cmd = CommandHook::new(hook_obj, &hook_value).unwrap();
    let info = gen.generate_command_script(&cmd, "sessionStart", None, 0);
    assert_eq!(info.path, "wrappers/cmd-sessionStart-0.sh");
    assert!(info.content.contains("#!/bin/bash"));
    assert!(info.content.contains("echo hello"));
    assert!(info.content.contains("COPILOT_INPUT=$(cat)"));
    assert!(info.content.contains("CLAUDE_INPUT"));
    assert!(info.content.contains("permissionDecision"));
}

#[test]
fn test_generate_command_script_with_matcher() {
    let gen = CopilotScriptGenerator;
    let hook_value = json!({"type": "command", "command": "cargo check"});
    let hook_obj = hook_value.as_object().unwrap();
    let cmd = CommandHook::new(hook_obj, &hook_value).unwrap();
    let info = gen.generate_command_script(&cmd, "preToolUse", Some("Bash"), 0);
    assert!(info.content.contains("matcher filter"));
    assert!(info.content.contains("Bash"));
    assert_eq!(info.matcher, Some("Bash".to_string()));
}

#[test]
fn test_generate_http_script() {
    let gen = CopilotScriptGenerator;
    let hook_value = json!({
        "type": "http",
        "url": "https://example.com/webhook",
        "method": "POST"
    });
    let hook_obj = hook_value.as_object().unwrap();
    let http = HttpHook::new(hook_obj, &hook_value).unwrap();
    let info = gen
        .generate_http_script(&http, "postToolUse", None, 0)
        .unwrap();
    assert_eq!(info.path, "wrappers/http-postToolUse-0.sh");
    assert!(info.content.contains("curl"));
    assert!(info.content.contains("https://example.com/webhook"));
}

#[test]
fn test_generate_http_script_invalid_method() {
    let hook_value = json!({
        "type": "http",
        "url": "https://example.com",
        "method": "EXEC"
    });
    let hook_obj = hook_value.as_object().unwrap();
    let result = HttpHook::new(hook_obj, &hook_value);
    assert!(result.is_err());
}

#[test]
fn test_generate_stub_script() {
    let gen = CopilotScriptGenerator;
    let hook_value = json!({
        "type": "prompt",
        "prompt": "Review code"
    });
    let stub = StubHook::new("prompt", &hook_value);
    let info = gen.generate_stub_script(&stub, "preToolUse", None, 0);
    assert_eq!(info.path, "wrappers/prompt-preToolUse-0.sh");
    assert!(info.content.contains("STUB"));
    assert!(info.content.contains("COPILOT_INPUT=$(cat)"));
}
