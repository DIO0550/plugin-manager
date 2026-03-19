//! Tests for hooks converter module.

use super::converter::*;
use crate::error::RichError;
use crate::error::{ErrorCode, PlmError};

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
    let result = convert(input).unwrap();
    assert_eq!(result.json["version"], 1);
    assert!(result.warnings.is_empty());
    assert!(result.wrapper_scripts.is_empty());
}

#[test]
fn test_detect_copilot_format_with_camelcase() {
    let input = r#"{
        "hooks": {
            "sessionStart": [{ "bash": "echo hi" }]
        }
    }"#;
    let result = convert(input).unwrap();
    // camelCase without version -> CopilotCli, returned as-is but with warning
    assert!(result.json.get("version").is_none());
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
    let result = convert(input).unwrap();
    assert_eq!(result.json["version"], 1);
    assert!(result.json["hooks"].get("sessionStart").is_some());
}

#[test]
fn test_detect_format_empty_hooks() {
    let input = r#"{ "hooks": {} }"#;
    let result = convert(input).unwrap();
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
    let result = convert(input).unwrap();
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
    let result = convert(input).unwrap();
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
    let result = convert(input).unwrap();
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
    let result = convert(input).unwrap();
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
    let result = convert(input).unwrap();
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
    let result = convert(input).unwrap();
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
                    "matcher": "*.rs",
                    "hooks": [{ "type": "command", "command": "cargo check" }]
                }
            ]
        }
    }"#;
    let result = convert(input).unwrap();
    let hooks = &result.json["hooks"]["preToolUse"];
    let arr = hooks.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert!(arr[0].get("steps").is_none());
    assert_eq!(arr[0]["bash"], "cargo check");
    // matcher is dropped with warning
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
                    "matcher": "*.rs",
                    "hooks": [
                        { "type": "command", "command": "cargo check" }
                    ]
                },
                {
                    "matcher": "*.ts",
                    "hooks": [
                        { "type": "command", "command": "tsc --noEmit" }
                    ]
                }
            ]
        }
    }"#;
    let result = convert(input).unwrap();
    let arr = result.json["hooks"]["preToolUse"].as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert!(arr[0].get("steps").is_none());
    assert_eq!(arr[0]["bash"], "cargo check");
    assert!(arr[1].get("steps").is_none());
    assert_eq!(arr[1]["bash"], "tsc --noEmit");
    // 2 matcher warnings
    assert_eq!(
        result
            .warnings
            .iter()
            .filter(|w| matches!(w, ConversionWarning::RemovedField { field, .. } if field == "matcher"))
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
    let result = convert(input).unwrap();
    let arr = result.json["hooks"]["preToolUse"].as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert!(arr[0].get("steps").is_none());
    assert_eq!(arr[0]["bash"], "echo all");
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
    let result = convert(input).unwrap();
    let hook = &result.json["hooks"]["sessionStart"][0];
    assert_eq!(hook["bash"], "./script.sh");
    assert!(hook.get("command").is_none());
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
    let result = convert(input).unwrap();
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
    let result = convert(input).unwrap();
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
    let result = convert(input).unwrap();
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
    let result = convert(input).unwrap();
    let hook = &result.json["hooks"]["sessionStart"][0];
    assert!(hook.get("once").is_none());
}

// ============================================================================
// BL-006: Hook type conversion
// ============================================================================

#[test]
fn test_command_hook_passthrough() {
    let input = r#"{
        "hooks": {
            "SessionStart": [
                { "hooks": [{ "type": "command", "command": "./run.sh", "timeout": 10 }] }
            ]
        }
    }"#;
    let result = convert(input).unwrap();
    let hook = &result.json["hooks"]["sessionStart"][0];
    assert_eq!(hook["bash"], "./run.sh");
    assert_eq!(hook["timeoutSec"], 10);
    assert_eq!(hook["type"], "command");
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
    let result = convert(input).unwrap();
    let hook = &result.json["hooks"]["postToolUse"][0];
    assert!(hook["bash"].as_str().unwrap().ends_with(".sh"));
    assert_eq!(hook["type"], "command");

    assert_eq!(result.wrapper_scripts.len(), 1);
    let script = &result.wrapper_scripts[0];
    assert!(script.content.contains("curl"));
    assert!(script.content.contains("https://example.com/webhook"));
    assert!(script.content.contains("HOOK_INPUT=$(cat)"));
    assert!(script.content.contains("CLAUDE_PROJECT_DIR"));
    assert!(script.content.contains("CLAUDE_PLUGIN_ROOT=\".\""));
    assert!(script.content.contains("Bearer token123"));
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
    let result = convert(input).unwrap();
    let hook = &result.json["hooks"]["preToolUse"][0];
    assert!(hook["bash"].as_str().unwrap().ends_with(".sh"));
    assert_eq!(hook["type"], "command");

    assert_eq!(result.wrapper_scripts.len(), 1);
    let script = &result.wrapper_scripts[0];
    assert!(script.content.contains("STUB"));
    assert!(script.content.contains("HOOK_INPUT=$(cat)"));
    assert!(script.content.contains("CLAUDE_PROJECT_DIR"));
    assert!(script.content.contains("CLAUDE_PLUGIN_ROOT=\".\""));

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
    let result = convert(input).unwrap();
    let hook = &result.json["hooks"]["agentStop"][0];
    assert!(hook["bash"].as_str().unwrap().ends_with(".sh"));
    assert_eq!(hook["type"], "command");

    assert!(result.warnings.iter().any(|w| matches!(
        w,
        ConversionWarning::PromptAgentHookStub { hook_type, .. } if hook_type == "agent"
    )));
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
    let result = convert(input).unwrap();
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
    let result = convert(input);
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
    let result = convert(input).unwrap();
    let script = &result.wrapper_scripts[0];
    // Single quote in URL should be escaped
    assert!(script.content.contains("'\\''"));
    assert!(!script.content.contains("it's'"));
}

// ============================================================================
// Error cases
// ============================================================================

#[test]
fn test_invalid_json_error() {
    let result = convert("not valid json");
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::HookConversion(msg) => assert!(msg.contains("Invalid JSON")),
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
    let result = convert(input);
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
    let result = convert(input);
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::HookConversion(msg) => assert!(msg.contains("url")),
        other => panic!("Expected HookConversion, got {:?}", other),
    }
}

#[test]
fn test_missing_hooks_field() {
    let input = r#"{ "version": 1 }"#;
    let result = convert(input);
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::HookConversion(msg) => assert!(msg.contains("hooks")),
        other => panic!("Expected HookConversion, got {:?}", other),
    }
}

#[test]
fn test_hooks_not_object() {
    let input = r#"{ "hooks": [1, 2, 3] }"#;
    let result = convert(input);
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
                    "matcher": "*.rs",
                    "hooks": [
                        { "type": "command", "command": "cargo check", "timeout": 30 }
                    ]
                },
                {
                    "matcher": "*.ts",
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

    let result = convert(input).unwrap();

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

    // PreToolUse has 2 hooks (from 2 matcher groups, matchers dropped with warnings)
    let pre_tool = hooks["preToolUse"].as_array().unwrap();
    assert_eq!(pre_tool.len(), 2);
    assert!(pre_tool[0].get("steps").is_none());
    assert_eq!(pre_tool[0]["bash"], "cargo check");
    assert_eq!(pre_tool[0]["timeoutSec"], 30);
    assert!(pre_tool[1].get("steps").is_none());
    assert_eq!(pre_tool[1]["bash"], "tsc --noEmit");

    // SessionStart key conversion
    let session = hooks["sessionStart"].as_array().unwrap();
    assert_eq!(session[0]["bash"], "echo start");
    assert_eq!(session[0]["comment"], "Starting...");

    // http hook -> wrapper script
    assert!(result
        .wrapper_scripts
        .iter()
        .any(|s| s.content.contains("curl")));

    // prompt hook -> stub
    assert!(result
        .wrapper_scripts
        .iter()
        .any(|s| s.content.contains("STUB")));

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
    let result = convert(input).unwrap();
    assert_eq!(result.json["version"], 1);
    assert_eq!(result.json["hooks"]["sessionStart"][0]["bash"], "echo hi");
    assert!(result.warnings.is_empty());
    assert!(result.wrapper_scripts.is_empty());
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
