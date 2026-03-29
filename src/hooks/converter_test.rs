//! Tests for hooks converter module.

use super::converter::*;
use crate::error::RichError;
use crate::error::{ErrorCode, PlmError};
use crate::target::TargetKind;

// ============================================================================
// ConversionWarning Display
// ============================================================================

#[test]
fn test_warning_display_unsupported_event() {
    let w = ConversionWarning::UnsupportedEvent {
        event: "PreCompact".to_string(),
    };
    assert_eq!(
        w.to_string(),
        "Event 'PreCompact' is not supported in Copilot CLI and was excluded"
    );
}

#[test]
fn test_warning_display_unsupported_hook_type() {
    let w = ConversionWarning::UnsupportedHookType {
        hook_type: "websocket".to_string(),
        event: "preToolUse".to_string(),
    };
    assert_eq!(
        w.to_string(),
        "Hook type 'websocket' for event 'preToolUse' is not supported and was excluded"
    );
}

#[test]
fn test_warning_display_removed_field() {
    let w = ConversionWarning::RemovedField {
        field: "async".to_string(),
        reason: "Copilot CLI does not support async hooks".to_string(),
    };
    assert_eq!(
        w.to_string(),
        "Field 'async' was removed: Copilot CLI does not support async hooks"
    );
}

#[test]
fn test_warning_display_prompt_agent_stub() {
    let w = ConversionWarning::PromptAgentHookStub {
        event: "preToolUse".to_string(),
        hook_type: "prompt".to_string(),
    };
    assert_eq!(
        w.to_string(),
        "'prompt' hook for event 'preToolUse' is a Claude Code-specific feature. A stub script was generated; please manually rewrite."
    );
}

// ============================================================================
// BL-001: Source format detection
// ============================================================================

#[test]
fn test_detect_copilot_format_with_version() {
    let input = r#"{
        "version": 1,
        "hooks": {
            "sessionStart": [{ "bash": "echo hi" }]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    assert_eq!(result.json["version"], 1);
    assert!(result.warnings.is_empty());
    assert!(result.scripts.is_empty());
}

#[test]
fn test_detect_copilot_format_with_camelcase() {
    let input = r#"{
        "hooks": {
            "sessionStart": [{ "bash": "echo hi" }]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    // camelCase without version -> CopilotCli, version:1 inserted with warning
    assert_eq!(result.json["version"], 1);
    assert!(result
        .warnings
        .iter()
        .any(|w| matches!(w, ConversionWarning::MissingVersion)));
}

#[test]
fn test_detect_claude_format_with_pascalcase() {
    let input = r#"{
        "hooks": {
            "SessionStart": [
                {
                    "hooks": [
                        { "type": "command", "command": "echo hi" }
                    ]
                }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    assert_eq!(result.json["version"], 1);
    assert!(result.json["hooks"].get("sessionStart").is_some());
}

#[test]
fn test_detect_format_empty_hooks() {
    let input = r#"{ "hooks": {} }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    // Empty hooks with no version -> CopilotCli passthrough with MissingVersion warning
    assert!(result
        .warnings
        .iter()
        .any(|w| matches!(w, ConversionWarning::MissingVersion)));
}

// ============================================================================
// BL-002: Top-level structure conversion
// ============================================================================

#[test]
fn test_add_version_field() {
    let input = r#"{
        "hooks": {
            "SessionStart": [
                { "hooks": [{ "type": "command", "command": "echo hi" }] }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    assert_eq!(result.json["version"], 1);
}

#[test]
fn test_remove_disable_all_hooks() {
    let input = r#"{
        "disableAllHooks": true,
        "hooks": {
            "SessionStart": [
                { "hooks": [{ "type": "command", "command": "echo hi" }] }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    assert!(result.json.get("disableAllHooks").is_none());
    assert!(result.warnings.iter().any(|w| matches!(
        w,
        ConversionWarning::RemovedField { field, .. } if field == "disableAllHooks"
    )));
}

#[test]
fn test_no_disable_all_hooks() {
    let input = r#"{
        "hooks": {
            "SessionStart": [
                { "hooks": [{ "type": "command", "command": "echo hi" }] }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    assert!(!result.warnings.iter().any(|w| matches!(
        w,
        ConversionWarning::RemovedField { field, .. } if field == "disableAllHooks"
    )));
}

// ============================================================================
// BL-003: Event name conversion
// ============================================================================

#[test]
fn test_convert_supported_events() {
    let input = r#"{
        "hooks": {
            "SessionStart": [{ "hooks": [{ "type": "command", "command": "echo 1" }] }],
            "SessionEnd": [{ "hooks": [{ "type": "command", "command": "echo 2" }] }],
            "PreToolUse": [{ "hooks": [{ "type": "command", "command": "echo 3" }] }],
            "PostToolUse": [{ "hooks": [{ "type": "command", "command": "echo 4" }] }],
            "UserPromptSubmit": [{ "hooks": [{ "type": "command", "command": "echo 5" }] }],
            "Stop": [{ "hooks": [{ "type": "command", "command": "echo 6" }] }],
            "SubagentStop": [{ "hooks": [{ "type": "command", "command": "echo 7" }] }]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    let hooks = result.json["hooks"].as_object().unwrap();
    assert!(hooks.contains_key("sessionStart"));
    assert!(hooks.contains_key("sessionEnd"));
    assert!(hooks.contains_key("preToolUse"));
    assert!(hooks.contains_key("postToolUse"));
    assert!(hooks.contains_key("userPromptSubmitted"));
    assert!(hooks.contains_key("agentStop"));
    assert!(hooks.contains_key("subagentStop"));
    assert_eq!(hooks.len(), 7);
}

#[test]
fn test_exclude_unsupported_events() {
    let input = r#"{
        "hooks": {
            "PreCompact": [{ "hooks": [{ "type": "command", "command": "echo 1" }] }],
            "PostCompact": [{ "hooks": [{ "type": "command", "command": "echo 2" }] }],
            "Notification": [{ "hooks": [{ "type": "command", "command": "echo 3" }] }]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    let hooks = result.json["hooks"].as_object().unwrap();
    assert!(hooks.is_empty());
    assert_eq!(
        result
            .warnings
            .iter()
            .filter(|w| matches!(w, ConversionWarning::UnsupportedEvent { .. }))
            .count(),
        3
    );
}

#[test]
fn test_exclude_unknown_events() {
    let input = r#"{
        "hooks": {
            "SomeNewEvent": [{ "hooks": [{ "type": "command", "command": "echo" }] }]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    let hooks = result.json["hooks"].as_object().unwrap();
    assert!(hooks.is_empty());
    assert!(result.warnings.iter().any(|w| matches!(
        w,
        ConversionWarning::UnsupportedEvent { event } if event == "SomeNewEvent"
    )));
}

// ============================================================================
// BL-004: Matcher flattening
// ============================================================================

#[test]
fn test_flatten_single_matcher() {
    let input = r#"{
        "hooks": {
            "PreToolUse": [
                {
                    "matcher": "Bash",
                    "hooks": [{ "type": "command", "command": "cargo check" }]
                }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    let hooks = &result.json["hooks"]["preToolUse"];
    let arr = hooks.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert!(arr[0].get("steps").is_none());
    // matcher present -> wrapper script generated, bash points to wrapper
    assert!(arr[0]["bash"].as_str().unwrap().ends_with(".sh"));
    assert_eq!(result.scripts.len(), 1);
    assert!(result.scripts[0].content.contains("cargo check"));
    assert!(result.scripts[0].content.contains("Bash"));
    assert_eq!(result.scripts[0].matcher, Some("Bash".to_string()));
    // matcher moved to wrapper script with warning
    assert!(result.warnings.iter().any(|w| matches!(
        w,
        ConversionWarning::RemovedField { field, .. } if field == "matcher"
    )));
}

#[test]
fn test_flatten_multiple_matchers() {
    let input = r#"{
        "hooks": {
            "PreToolUse": [
                {
                    "matcher": "Bash",
                    "hooks": [
                        { "type": "command", "command": "cargo check" }
                    ]
                },
                {
                    "matcher": "Write|Edit",
                    "hooks": [
                        { "type": "command", "command": "tsc --noEmit" }
                    ]
                }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    let arr = result.json["hooks"]["preToolUse"].as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert!(arr[0].get("steps").is_none());
    // Each matcher produces a wrapper script
    assert!(arr[0]["bash"].as_str().unwrap().ends_with(".sh"));
    assert!(arr[1]["bash"].as_str().unwrap().ends_with(".sh"));
    assert_eq!(result.scripts.len(), 2);
    assert!(result.scripts[0].content.contains("cargo check"));
    assert!(result.scripts[1].content.contains("tsc --noEmit"));
    // 2 matcher warnings
    assert_eq!(
        result
            .warnings
            .iter()
            .filter(
                |w| matches!(w, ConversionWarning::RemovedField { field, .. } if field == "matcher")
            )
            .count(),
        2
    );
}

#[test]
fn test_flatten_empty_matcher() {
    let input = r#"{
        "hooks": {
            "PreToolUse": [
                {
                    "matcher": "",
                    "hooks": [{ "type": "command", "command": "echo all" }]
                }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    let arr = result.json["hooks"]["preToolUse"].as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert!(arr[0].get("steps").is_none());
    assert!(arr[0]["bash"].as_str().unwrap().ends_with(".sh"));
    assert!(result.scripts[0].content.contains("echo all"));
}

// ============================================================================
// BL-005: Key name conversion
// ============================================================================

#[test]
fn test_command_to_bash() {
    let input = r#"{
        "hooks": {
            "SessionStart": [
                { "hooks": [{ "type": "command", "command": "./script.sh" }] }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    let hook = &result.json["hooks"]["sessionStart"][0];
    // Always wrapped: bash points to a generated wrapper script
    assert!(hook["bash"].as_str().unwrap().ends_with(".sh"));
    assert!(hook.get("command").is_none());
    // Wrapper contains the original command
    assert!(result.scripts[0].content.contains("./script.sh"));
}

#[test]
fn test_timeout_to_timeout_sec() {
    let input = r#"{
        "hooks": {
            "SessionStart": [
                { "hooks": [{ "type": "command", "command": "echo", "timeout": 30 }] }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    let hook = &result.json["hooks"]["sessionStart"][0];
    assert_eq!(hook["timeoutSec"], 30);
    assert!(hook.get("timeout").is_none());
}

#[test]
fn test_status_message_to_comment() {
    let input = r#"{
        "hooks": {
            "SessionStart": [
                { "hooks": [{ "type": "command", "command": "echo", "statusMessage": "Running..." }] }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    let hook = &result.json["hooks"]["sessionStart"][0];
    assert_eq!(hook["comment"], "Running...");
    assert!(hook.get("statusMessage").is_none());
}

#[test]
fn test_remove_async_with_warning() {
    let input = r#"{
        "hooks": {
            "SessionStart": [
                { "hooks": [{ "type": "command", "command": "echo", "async": true }] }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    let hook = &result.json["hooks"]["sessionStart"][0];
    assert!(hook.get("async").is_none());
    assert!(result.warnings.iter().any(|w| matches!(
        w,
        ConversionWarning::RemovedField { field, .. } if field == "async"
    )));
}

#[test]
fn test_remove_once() {
    let input = r#"{
        "hooks": {
            "SessionStart": [
                { "hooks": [{ "type": "command", "command": "echo", "once": true }] }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    let hook = &result.json["hooks"]["sessionStart"][0];
    assert!(hook.get("once").is_none());
    assert!(result.warnings.iter().any(|w| matches!(
        w,
        ConversionWarning::RemovedField { field, .. } if field == "once"
    )));
}

// ============================================================================
// BL-006: Hook type conversion
// ============================================================================

#[test]
fn test_command_hook_always_has_type() {
    // When input omits "type" (defaults to command), output must still include type: "command"
    let input = r#"{
        "hooks": {
            "SessionStart": [
                { "hooks": [{ "command": "echo hi" }] }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    let hook = &result.json["hooks"]["sessionStart"][0];
    assert_eq!(hook["type"], "command");
    assert!(hook["bash"].as_str().unwrap().ends_with(".sh"));
    assert!(result.scripts[0].content.contains("echo hi"));
}

#[test]
fn test_command_hook_passthrough() {
    let input = r#"{
        "hooks": {
            "SessionStart": [
                { "hooks": [{ "type": "command", "command": "./run.sh", "timeout": 10 }] }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    let hook = &result.json["hooks"]["sessionStart"][0];
    assert!(hook["bash"].as_str().unwrap().ends_with(".sh"));
    assert_eq!(hook["timeoutSec"], 10);
    assert_eq!(hook["type"], "command");
    assert!(result.scripts[0].content.contains("./run.sh"));
    // Wrapper includes exit code handler
    assert!(result.scripts[0].content.contains("permissionDecision"));
    assert!(result.scripts[0].content.contains("hookSpecificOutput"));
}

#[test]
fn test_http_hook_to_curl_wrapper() {
    let input = r#"{
        "hooks": {
            "PostToolUse": [
                {
                    "hooks": [{
                        "type": "http",
                        "url": "https://example.com/webhook",
                        "method": "POST",
                        "headers": { "Authorization": "Bearer token123" },
                        "body": true,
                        "timeout": 15
                    }]
                }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    let hook = &result.json["hooks"]["postToolUse"][0];
    assert!(hook["bash"].as_str().unwrap().ends_with(".sh"));
    assert_eq!(hook["type"], "command");

    assert_eq!(result.scripts.len(), 1);
    let script = &result.scripts[0];
    // Core curl invocation
    assert!(script.content.contains("curl"));
    assert!(script.content.contains("https://example.com/webhook"));
    assert!(script.content.contains("-d @-"));
    assert!(script.content.contains("Bearer token123"));
    // ENV_BRIDGE with fail-open
    assert!(script.content.contains("COPILOT_INPUT=$(cat)"));
    assert!(script.content.contains("CLAUDE_PROJECT_DIR"));
    assert!(script
        .content
        .contains("CLAUDE_PLUGIN_ROOT=\"@@PLUGIN_ROOT@@\""));
    assert!(script.content.contains("command -v jq"));
    // HTTP status code handling
    assert!(script.content.contains("http_code"));
    assert!(script.content.contains("HTTP_CODE"));
    // hookSpecificOutput unwrap in response
    assert!(script.content.contains("hookSpecificOutput"));
    // Fail-open exit
    assert!(script.content.contains("exit 0"));
}

#[test]
fn test_http_hook_with_matcher_has_filter() {
    let input = r#"{
        "hooks": {
            "PostToolUse": [
                {
                    "matcher": "Bash",
                    "hooks": [{
                        "type": "http",
                        "url": "https://example.com/webhook",
                        "method": "POST"
                    }]
                }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    assert_eq!(result.scripts.len(), 1);
    let script = &result.scripts[0];
    assert!(script.content.contains("curl"));
    assert!(script.content.contains("matcher filter"));
    assert!(script.content.contains("Bash"));
}

#[test]
fn test_prompt_hook_to_stub() {
    let input = r#"{
        "hooks": {
            "PreToolUse": [
                {
                    "hooks": [{
                        "type": "prompt",
                        "prompt": "Review this code for security issues"
                    }]
                }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    let hook = &result.json["hooks"]["preToolUse"][0];
    assert!(hook["bash"].as_str().unwrap().ends_with(".sh"));
    assert_eq!(hook["type"], "command");

    assert_eq!(result.scripts.len(), 1);
    let script = &result.scripts[0];
    assert!(script.content.contains("STUB"));
    assert!(script.content.contains("COPILOT_INPUT=$(cat)"));
    assert!(script.content.contains("CLAUDE_PROJECT_DIR"));
    assert!(script
        .content
        .contains("CLAUDE_PLUGIN_ROOT=\"@@PLUGIN_ROOT@@\""));

    assert!(result.warnings.iter().any(|w| matches!(
        w,
        ConversionWarning::PromptAgentHookStub { hook_type, .. } if hook_type == "prompt"
    )));
}

#[test]
fn test_agent_hook_to_stub() {
    let input = r#"{
        "hooks": {
            "Stop": [
                {
                    "hooks": [{
                        "type": "agent",
                        "agent": "Summarize the session"
                    }]
                }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    let hook = &result.json["hooks"]["agentStop"][0];
    assert!(hook["bash"].as_str().unwrap().ends_with(".sh"));
    assert_eq!(hook["type"], "command");

    assert!(result.warnings.iter().any(|w| matches!(
        w,
        ConversionWarning::PromptAgentHookStub { hook_type, .. } if hook_type == "agent"
    )));
}

#[test]
fn test_prompt_hook_with_matcher_has_filter() {
    let input = r#"{
        "hooks": {
            "PreToolUse": [
                {
                    "matcher": "Bash",
                    "hooks": [{
                        "type": "prompt",
                        "prompt": "Review code"
                    }]
                }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    assert_eq!(result.scripts.len(), 1);
    let script = &result.scripts[0];
    assert!(script.content.contains("STUB"));
    assert!(script.content.contains("matcher filter"));
    assert!(script.content.contains("Bash"));
}

#[test]
fn test_prompt_hook_preserves_timeout_and_status_message() {
    let input = r#"{
        "hooks": {
            "PreToolUse": [
                {
                    "hooks": [{
                        "type": "prompt",
                        "prompt": "Review code",
                        "timeout": 20,
                        "statusMessage": "Reviewing..."
                    }]
                }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    let hook = &result.json["hooks"]["preToolUse"][0];
    assert_eq!(hook["type"], "command");
    assert_eq!(hook["timeoutSec"], 20);
    assert_eq!(hook["comment"], "Reviewing...");
}

#[test]
fn test_unknown_hook_type_excluded() {
    let input = r#"{
        "hooks": {
            "SessionStart": [
                {
                    "hooks": [{ "type": "websocket", "url": "wss://example.com" }]
                }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    let hooks = result.json["hooks"]["sessionStart"].as_array();
    assert!(hooks.is_none() || hooks.unwrap().is_empty());
    assert!(result.warnings.iter().any(|w| matches!(
        w,
        ConversionWarning::UnsupportedHookType { hook_type, .. } if hook_type == "websocket"
    )));
}

#[test]
fn test_http_hook_invalid_method() {
    let input = r#"{
        "hooks": {
            "SessionStart": [
                { "hooks": [{ "type": "http", "url": "https://example.com", "method": "EXEC" }] }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot);
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::HookConversion(msg) => assert!(msg.contains("unsupported method")),
        other => panic!("Expected HookConversion, got {:?}", other),
    }
}

#[test]
fn test_http_hook_shell_escape() {
    let input = r#"{
        "hooks": {
            "SessionStart": [
                {
                    "hooks": [{
                        "type": "http",
                        "url": "https://example.com/hook?q=it's",
                        "method": "POST"
                    }]
                }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    let script = &result.scripts[0];
    // Single quote in URL should be escaped
    assert!(script.content.contains("'\\''"));
    assert!(!script.content.contains("it's'"));
}

// ============================================================================
// HTTP header validation
// ============================================================================

#[test]
fn test_http_header_invalid_name_rejected() {
    let input = r#"{
        "hooks": {
            "SessionStart": [
                { "hooks": [{ "type": "http", "url": "https://example.com", "headers": { "Invalid Header": "value" } }] }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot);
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::HookConversion(msg) => assert!(msg.contains("invalid characters")),
        other => panic!("Expected HookConversion, got {:?}", other),
    }
}

#[test]
fn test_http_header_value_newline_rejected() {
    let input = "{\n\"hooks\": {\n\"SessionStart\": [\n{ \"hooks\": [{ \"type\": \"http\", \"url\": \"https://example.com\", \"headers\": { \"X-Test\": \"val\\nue\" } }] }\n]\n}\n}";
    let result = convert(input, TargetKind::Copilot);
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::HookConversion(msg) => assert!(msg.contains("newline")),
        other => panic!("Expected HookConversion, got {:?}", other),
    }
}

#[test]
fn test_http_header_value_command_substitution_rejected() {
    let input = r#"{
        "hooks": {
            "SessionStart": [
                { "hooks": [{ "type": "http", "url": "https://example.com", "headers": { "X-Test": "$(whoami)" } }] }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot);
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::HookConversion(msg) => assert!(msg.contains("command substitution")),
        other => panic!("Expected HookConversion, got {:?}", other),
    }
}

#[test]
fn test_http_header_value_backtick_rejected() {
    let input = r#"{
        "hooks": {
            "SessionStart": [
                { "hooks": [{ "type": "http", "url": "https://example.com", "headers": { "X-Test": "`whoami`" } }] }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot);
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::HookConversion(msg) => assert!(msg.contains("command substitution")),
        other => panic!("Expected HookConversion, got {:?}", other),
    }
}

#[test]
fn test_http_header_non_string_value_rejected() {
    let input = r#"{
        "hooks": {
            "SessionStart": [
                { "hooks": [{ "type": "http", "url": "https://example.com", "headers": { "X-Test": 123 } }] }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot);
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::HookConversion(msg) => assert!(msg.contains("non-string")),
        other => panic!("Expected HookConversion, got {:?}", other),
    }
}

// ============================================================================
// Error cases
// ============================================================================

#[test]
fn test_invalid_json_error() {
    let result = convert("not valid json", TargetKind::Copilot);
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::HookConversion(msg) => assert!(msg.contains("Invalid JSON")),
        other => panic!("Expected HookConversion, got {:?}", other),
    }
}

#[test]
fn test_command_with_newline_rejected() {
    let input = "{\n\"hooks\": {\n\"SessionStart\": [\n{ \"hooks\": [{ \"type\": \"command\", \"command\": \"echo\\ninjected\" }] }\n]\n}\n}";
    let result = convert(input, TargetKind::Copilot);
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::HookConversion(msg) => assert!(msg.contains("newline")),
        other => panic!("Expected HookConversion, got {:?}", other),
    }
}

#[test]
fn test_command_with_carriage_return_rejected() {
    let input = "{\n\"hooks\": {\n\"SessionStart\": [\n{ \"hooks\": [{ \"type\": \"command\", \"command\": \"echo\\rinjected\" }] }\n]\n}\n}";
    let result = convert(input, TargetKind::Copilot);
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::HookConversion(msg) => {
            assert!(msg.contains("newline") || msg.contains("carriage"))
        }
        other => panic!("Expected HookConversion, got {:?}", other),
    }
}

#[test]
fn test_command_missing_command_field() {
    let input = r#"{
        "hooks": {
            "SessionStart": [
                { "hooks": [{ "type": "command", "timeout": 10 }] }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot);
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::HookConversion(msg) => assert!(msg.contains("command")),
        other => panic!("Expected HookConversion, got {:?}", other),
    }
}

#[test]
fn test_http_missing_url_field() {
    let input = r#"{
        "hooks": {
            "SessionStart": [
                { "hooks": [{ "type": "http", "method": "GET" }] }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot);
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::HookConversion(msg) => assert!(msg.contains("url")),
        other => panic!("Expected HookConversion, got {:?}", other),
    }
}

#[test]
fn test_missing_hooks_field() {
    let input = r#"{ "version": 1 }"#;
    let result = convert(input, TargetKind::Copilot);
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::HookConversion(msg) => assert!(msg.contains("hooks")),
        other => panic!("Expected HookConversion, got {:?}", other),
    }
}

#[test]
fn test_hooks_not_object() {
    let input = r#"{ "hooks": [1, 2, 3] }"#;
    let result = convert(input, TargetKind::Copilot);
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::HookConversion(msg) => assert!(msg.contains("object")),
        other => panic!("Expected HookConversion, got {:?}", other),
    }
}

// ============================================================================
// E2E integration tests
// ============================================================================

#[test]
fn test_full_conversion_scenario() {
    let input = r#"{
        "disableAllHooks": false,
        "hooks": {
            "SessionStart": [
                {
                    "hooks": [
                        { "type": "command", "command": "echo start", "statusMessage": "Starting..." }
                    ]
                }
            ],
            "PreToolUse": [
                {
                    "matcher": "Bash",
                    "hooks": [
                        { "type": "command", "command": "cargo check", "timeout": 30 }
                    ]
                },
                {
                    "matcher": "Write|Edit",
                    "hooks": [
                        { "type": "command", "command": "tsc --noEmit" }
                    ]
                }
            ],
            "PostToolUse": [
                {
                    "hooks": [
                        {
                            "type": "http",
                            "url": "https://example.com/hook",
                            "method": "POST"
                        }
                    ]
                }
            ],
            "Stop": [
                {
                    "hooks": [
                        { "type": "prompt", "prompt": "Summarize changes" }
                    ]
                }
            ],
            "PreCompact": [
                {
                    "hooks": [
                        { "type": "command", "command": "echo compact" }
                    ]
                }
            ]
        }
    }"#;

    let result = convert(input, TargetKind::Copilot).unwrap();

    // version:1 added
    assert_eq!(result.json["version"], 1);

    // disableAllHooks removed
    assert!(result.json.get("disableAllHooks").is_none());

    // Supported events converted
    let hooks = result.json["hooks"].as_object().unwrap();
    assert!(hooks.contains_key("sessionStart"));
    assert!(hooks.contains_key("preToolUse"));
    assert!(hooks.contains_key("postToolUse"));
    assert!(hooks.contains_key("agentStop"));

    // Unsupported event excluded
    assert!(!hooks.contains_key("PreCompact"));
    assert!(!hooks.contains_key("preCompact"));

    // PreToolUse has 2 hooks (from 2 matcher groups, matchers moved to wrapper scripts)
    let pre_tool = hooks["preToolUse"].as_array().unwrap();
    assert_eq!(pre_tool.len(), 2);
    assert!(pre_tool[0].get("steps").is_none());
    // With matchers, bash points to wrapper scripts
    assert!(pre_tool[0]["bash"].as_str().unwrap().ends_with(".sh"));
    assert_eq!(pre_tool[0]["timeoutSec"], 30);
    assert!(pre_tool[1].get("steps").is_none());
    assert!(pre_tool[1]["bash"].as_str().unwrap().ends_with(".sh"));
    // Wrapper scripts contain the original commands and matcher patterns
    assert!(result
        .scripts
        .iter()
        .any(|s| s.content.contains("cargo check") && s.content.contains("Bash")));
    assert!(result
        .scripts
        .iter()
        .any(|s| s.content.contains("tsc --noEmit") && s.content.contains("Write|Edit")));

    // SessionStart key conversion
    let session = hooks["sessionStart"].as_array().unwrap();
    assert!(session[0]["bash"].as_str().unwrap().ends_with(".sh"));
    assert_eq!(session[0]["comment"], "Starting...");
    assert!(result
        .scripts
        .iter()
        .any(|s| s.content.contains("echo start")));

    // http hook -> wrapper script
    assert!(result.scripts.iter().any(|s| s.content.contains("curl")));

    // prompt hook -> stub
    assert!(result.scripts.iter().any(|s| s.content.contains("STUB")));

    // Warnings: disableAllHooks removed + PreCompact unsupported + prompt stub
    assert!(result.warnings.iter().any(
        |w| matches!(w, ConversionWarning::RemovedField { field, .. } if field == "disableAllHooks")
    ));
    assert!(result.warnings.iter().any(
        |w| matches!(w, ConversionWarning::UnsupportedEvent { event } if event == "PreCompact")
    ));
    assert!(result
        .warnings
        .iter()
        .any(|w| matches!(w, ConversionWarning::PromptAgentHookStub { .. })));
}

#[test]
fn test_copilot_format_passthrough() {
    let input = r#"{
        "version": 1,
        "hooks": {
            "sessionStart": [{ "bash": "echo hi" }],
            "preToolUse": [{ "bash": "cargo check", "steps": "*.rs" }]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    assert_eq!(result.json["version"], 1);
    assert_eq!(result.json["hooks"]["sessionStart"][0]["bash"], "echo hi");
    assert!(result.warnings.is_empty());
    assert!(result.scripts.is_empty());
}

// ============================================================================
// Error type mapping
// ============================================================================

#[test]
fn test_hook_conversion_error_code() {
    let error = PlmError::HookConversion("test error".to_string());
    let rich: RichError = error.into();
    assert_eq!(rich.code(), ErrorCode::Hok001);
}

// ============================================================================
// Event-specific ENV_BRIDGE (build_env_bridge)
// ============================================================================

#[test]
fn test_session_start_has_source_mapping() {
    let input = r#"{
        "hooks": {
            "SessionStart": [
                { "hooks": [{ "type": "command", "command": "echo start" }] }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    let script = &result.scripts[0];
    // sessionStart should have source mapping jq (new -> startup)
    assert!(script.content.contains("startup"));
    assert!(script.content.contains("resume"));
}

#[test]
fn test_session_start_has_del_initial_prompt() {
    let input = r#"{
        "hooks": {
            "SessionStart": [
                { "hooks": [{ "type": "command", "command": "echo start" }] }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    let script = &result.scripts[0];
    assert!(script.content.contains("del(.initialPrompt)"));
}

#[test]
fn test_pre_tool_use_has_tool_fields() {
    let input = r#"{
        "hooks": {
            "PreToolUse": [
                { "hooks": [{ "type": "command", "command": "echo pre" }] }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    let script = &result.scripts[0];
    // preToolUse should have tool_name mapping
    assert!(script.content.contains("tool_name"));
    assert!(script.content.contains("tool_input"));
    assert!(script.content.contains("toolName"));
}

#[test]
fn test_user_prompt_submitted_no_tool_fields() {
    let input = r#"{
        "hooks": {
            "UserPromptSubmit": [
                { "hooks": [{ "type": "command", "command": "echo prompt" }] }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    let script = &result.scripts[0];
    // userPromptSubmitted should NOT have tool_name/tool_input/tool_response mapping
    assert!(!script.content.contains("tool_name"));
    assert!(!script.content.contains("tool_input"));
    assert!(!script.content.contains("tool_response"));
    // But should still have COPILOT_INPUT and CLAUDE_INPUT
    assert!(script.content.contains("COPILOT_INPUT=$(cat)"));
    assert!(script.content.contains("CLAUDE_INPUT"));
    assert!(script.content.contains("CLAUDE_PROJECT_DIR"));
}

#[test]
fn test_session_start_fail_open_warning() {
    let input = r#"{
        "hooks": {
            "SessionStart": [
                { "hooks": [{ "type": "command", "command": "echo start" }] }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    let script = &result.scripts[0];
    // sessionStart should have fail-open warning for secondary jq
    assert!(script
        .content
        .contains("plm: warning: failed to apply sessionStart-specific transformation"));
}

// ============================================================================
// bash -n syntax validation for generated wrapper scripts
// ============================================================================

/// Helper: generate a command hook wrapper for the given Claude Code event name.
fn generate_wrapper_for_event(claude_event: &str) -> String {
    generate_wrapper_for_event_with_command(claude_event, "echo test")
}

/// Helper: generate a command hook wrapper with a custom command.
fn generate_wrapper_for_event_with_command(claude_event: &str, command: &str) -> String {
    let input = format!(
        r#"{{
        "hooks": {{
            "{}": [
                {{ "hooks": [{{ "type": "command", "command": "{}" }}] }}
            ]
        }}
    }}"#,
        claude_event, command
    );
    let result = convert(&input, TargetKind::Copilot).unwrap();
    result.scripts[0].content.clone()
}

/// Helper: run just the env bridge portion of a generated wrapper script,
/// then output $CLAUDE_INPUT to stdout. This bypasses the EXIT_CODE_HANDLER
/// which only outputs for preToolUse events.
fn run_env_bridge_for_event(claude_event: &str, stdin_json: &str) -> (String, String) {
    use std::io::Write;

    let script = generate_wrapper_for_event_with_command(claude_event, "true");
    // Extract everything before ORIGINAL_CMD= (the env bridge part),
    // then output CLAUDE_INPUT directly.
    let marker = "ORIGINAL_CMD=";
    let env_bridge_end = script.find(marker).unwrap_or(script.len());
    let test_script = format!(
        "{}printf '%s' \"$CLAUDE_INPUT\"\n",
        &script[..env_bridge_end]
    );

    let mut tmp = tempfile::Builder::new()
        .prefix("plm-integ-")
        .suffix(".sh")
        .tempfile()
        .unwrap();
    tmp.write_all(test_script.as_bytes()).unwrap();
    tmp.flush().unwrap();

    let output = std::process::Command::new("bash")
        .arg(tmp.path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            child
                .stdin
                .take()
                .unwrap()
                .write_all(stdin_json.as_bytes())
                .unwrap();
            child.wait_with_output()
        })
        .unwrap();
    // tmp is cleaned up automatically via Drop
    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

/// Helper: write script to a unique temp file and run `bash -n` syntax check.
fn assert_bash_n_valid(script: &str, label: &str) {
    use std::io::Write;

    let mut tmp = tempfile::Builder::new()
        .prefix(&format!("plm-bashn-{}-", label))
        .suffix(".sh")
        .tempfile()
        .unwrap();
    tmp.write_all(script.as_bytes()).unwrap();
    tmp.flush().unwrap();

    let output = std::process::Command::new("bash")
        .arg("-n")
        .arg(tmp.path())
        .output()
        .unwrap();
    // tmp is cleaned up automatically via Drop
    assert!(
        output.status.success(),
        "bash -n failed for {}:\n{}",
        label,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[cfg(unix)]
fn test_bash_n_syntax_pre_tool_use() {
    assert_bash_n_valid(&generate_wrapper_for_event("PreToolUse"), "preToolUse");
}

#[test]
#[cfg(unix)]
fn test_bash_n_syntax_post_tool_use() {
    assert_bash_n_valid(&generate_wrapper_for_event("PostToolUse"), "postToolUse");
}

#[test]
#[cfg(unix)]
fn test_bash_n_syntax_session_start() {
    assert_bash_n_valid(&generate_wrapper_for_event("SessionStart"), "sessionStart");
}

#[test]
#[cfg(unix)]
fn test_bash_n_syntax_user_prompt_submitted() {
    assert_bash_n_valid(
        &generate_wrapper_for_event("UserPromptSubmit"),
        "userPromptSubmitted",
    );
}

#[test]
#[cfg(unix)]
fn test_bash_n_syntax_session_end() {
    assert_bash_n_valid(&generate_wrapper_for_event("SessionEnd"), "sessionEnd");
}

// ============================================================================
// Integration tests: verify actual JSON transformation via script execution
// (unix-only: requires bash and jq)
// ============================================================================

/// Check if jq is available and working correctly.
fn jq_available() -> bool {
    std::process::Command::new("jq")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

#[test]
#[cfg(unix)]
fn test_integ_session_start_source_mapping_and_del_initial_prompt() {
    if !jq_available() {
        return;
    }

    let input_json =
        r#"{"source":"new","initialPrompt":"hello","sessionId":"s1","cwd":"/tmp","timestamp":123}"#;
    let (stdout, _stderr) = run_env_bridge_for_event("SessionStart", input_json);

    let result: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
        panic!("Failed to parse stdout JSON: {}\nstdout: {}", e, stdout);
    });

    // source "new" -> "startup"
    assert_eq!(result["source"], "startup");
    // initialPrompt removed
    assert!(result.get("initialPrompt").is_none());
    // session_id normalized
    assert_eq!(result["session_id"], "s1");
    // timestamp removed
    assert!(result.get("timestamp").is_none());
    // sessionId removed
    assert!(result.get("sessionId").is_none());
    // tool fields should NOT be present
    assert!(result.get("tool_name").is_none());
    assert!(result.get("tool_input").is_none());
    assert!(result.get("tool_response").is_none());
}

#[test]
#[cfg(unix)]
fn test_integ_session_start_source_resume() {
    if !jq_available() {
        return;
    }

    let input_json = r#"{"source":"resume","sessionId":"s2"}"#;
    let (stdout, _stderr) = run_env_bridge_for_event("SessionStart", input_json);

    let result: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
        panic!("Failed to parse stdout JSON: {}\nstdout: {}", e, stdout);
    });

    assert_eq!(result["source"], "resume");
}

#[test]
#[cfg(unix)]
fn test_integ_session_start_no_source_field() {
    if !jq_available() {
        return;
    }

    let input_json = r#"{"sessionId":"s3"}"#;
    let (stdout, _stderr) = run_env_bridge_for_event("SessionStart", input_json);

    let result: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
        panic!("Failed to parse stdout JSON: {}\nstdout: {}", e, stdout);
    });

    // source field should NOT be injected as null when missing
    assert!(
        result.get("source").is_none(),
        "source should not be injected when missing from input, got: {:?}",
        result.get("source")
    );
}

#[test]
#[cfg(unix)]
fn test_integ_user_prompt_no_tool_fields() {
    if !jq_available() {
        return;
    }

    let input_json = r#"{"prompt":"hello","sessionId":"s4","timestamp":456}"#;
    let (stdout, _stderr) = run_env_bridge_for_event("UserPromptSubmit", input_json);

    let result: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
        panic!("Failed to parse stdout JSON: {}\nstdout: {}", e, stdout);
    });

    // tool fields should NOT be present
    assert!(
        result.get("tool_name").is_none(),
        "tool_name should not be present for userPromptSubmitted"
    );
    assert!(
        result.get("tool_input").is_none(),
        "tool_input should not be present for userPromptSubmitted"
    );
    assert!(
        result.get("tool_response").is_none(),
        "tool_response should not be present for userPromptSubmitted"
    );
    // base transformation should still work
    assert!(result.get("timestamp").is_none());
    assert_eq!(result["session_id"], "s4");
    assert!(result.get("sessionId").is_none());
}

#[test]
#[cfg(unix)]
fn test_integ_pre_tool_use_has_tool_mapping() {
    if !jq_available() {
        return;
    }

    let input_json =
        r#"{"toolName":"bash","toolArgs":"{\"command\":\"ls\"}","sessionId":"s5","timestamp":789}"#;
    let (stdout, _stderr) = run_env_bridge_for_event("PreToolUse", input_json);

    let result: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
        panic!("Failed to parse stdout JSON: {}\nstdout: {}", e, stdout);
    });

    // tool_name mapped: bash -> Bash
    assert_eq!(result["tool_name"], "Bash");
    // tool_input parsed from JSON string
    assert_eq!(result["tool_input"]["command"], "ls");
    // Copilot-specific keys cleaned up
    assert!(result.get("toolName").is_none());
    assert!(result.get("toolArgs").is_none());
    assert!(result.get("sessionId").is_none());
    assert!(result.get("timestamp").is_none());
    assert_eq!(result["session_id"], "s5");
    // preToolUse should NOT have tool_response (it's postToolUse-only)
    assert!(
        result.get("tool_response").is_none(),
        "tool_response should not be present for preToolUse"
    );
}

// ============================================================================
// namespace_script_path
// ============================================================================

#[test]
fn test_namespace_script_path_basic() {
    assert_eq!(
        namespace_script_path("wrappers/foo.sh", "my-hook"),
        "wrappers/my-hook/foo.sh"
    );
}

#[test]
fn test_namespace_script_path_no_prefix() {
    assert_eq!(
        namespace_script_path("other/foo.sh", "my-hook"),
        "other/foo.sh"
    );
}

#[test]
fn test_namespace_script_path_nested() {
    assert_eq!(
        namespace_script_path("wrappers/sub/foo.sh", "my-hook"),
        "wrappers/my-hook/sub/foo.sh"
    );
}

// ============================================================================
// original_config補完
// ============================================================================

#[test]
fn test_command_hook_original_config_is_set() {
    let input = r#"{
        "hooks": {
            "SessionStart": [
                { "hooks": [{ "type": "command", "command": "echo hi", "timeout": 5 }] }
            ]
        }
    }"#;
    let result = convert(input, TargetKind::Copilot).unwrap();
    assert_eq!(result.scripts.len(), 1);
    let script = &result.scripts[0];
    // original_config should be the full hook object
    assert!(script.original_config.is_object());
    assert_eq!(script.original_config["command"], "echo hi");
    assert_eq!(script.original_config["timeout"], 5);
    assert_eq!(script.original_config["type"], "command");
}
