//! Hook configuration converter: Claude Code Hooks JSON to Copilot CLI format.
//!
//! Converts Claude Code hooks configuration to Copilot CLI format,
//! handling event name mapping, key name translation, and hook type conversion.

use std::fmt;

use serde_json::Value;

use crate::error::PlmError;
use crate::hooks::event_map::event_claude_to_copilot;

/// Conversion result containing the transformed JSON, warnings, and wrapper script info.
#[derive(Debug, Clone)]
pub struct ConvertResult {
    pub json: Value,
    pub warnings: Vec<ConversionWarning>,
    pub wrapper_scripts: Vec<WrapperScriptInfo>,
}

/// Warnings emitted during conversion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConversionWarning {
    UnsupportedEvent { event: String },
    UnsupportedHookType { hook_type: String, event: String },
    RemovedField { field: String, reason: String },
    PromptAgentHookStub { event: String, hook_type: String },
}

impl fmt::Display for ConversionWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversionWarning::UnsupportedEvent { event } => {
                write!(
                    f,
                    "Event '{}' is not supported in Copilot CLI and was excluded",
                    event
                )
            }
            ConversionWarning::UnsupportedHookType { hook_type, event } => {
                write!(
                    f,
                    "Hook type '{}' for event '{}' is not supported and was excluded",
                    hook_type, event
                )
            }
            ConversionWarning::RemovedField { field, reason } => {
                write!(f, "Field '{}' was removed: {}", field, reason)
            }
            ConversionWarning::PromptAgentHookStub { event, hook_type } => {
                write!(
                    f,
                    "'{}' hook for event '{}' is a Claude Code-specific feature. A stub script was generated; please manually rewrite.",
                    hook_type, event
                )
            }
        }
    }
}

/// Information about a wrapper script that needs to be created.
#[derive(Debug, Clone)]
pub struct WrapperScriptInfo {
    pub path: String,
    pub content: String,
    pub original_config: Value,
    pub matcher: Option<String>,
}

/// Detected source format.
enum SourceFormat {
    ClaudeCode,
    CopilotCli,
}

/// Environment variable bridge lines for wrapper scripts.
/// Copilot CLI passes hook payload via stdin (not env var), so we read it first.
const ENV_BRIDGE: &str = r#"HOOK_INPUT=$(cat)
export CLAUDE_PROJECT_DIR=$(echo "$HOOK_INPUT" | jq -r '.cwd // empty')
export CLAUDE_PLUGIN_ROOT=".""#;

/// Convert Claude Code hooks JSON to Copilot CLI format.
///
/// If the input is already in Copilot CLI format, it is returned as-is.
pub fn convert(input: &str) -> Result<ConvertResult, PlmError> {
    let value: Value = serde_json::from_str(input)
        .map_err(|e| PlmError::HookConversion(format!("Invalid JSON: {}", e)))?;

    let hooks_value = value
        .get("hooks")
        .ok_or_else(|| PlmError::HookConversion("Missing 'hooks' field".to_string()))?;

    if !hooks_value.is_object() {
        return Err(PlmError::HookConversion(
            "'hooks' field must be an object".to_string(),
        ));
    }

    match detect_format(&value) {
        SourceFormat::CopilotCli => Ok(ConvertResult {
            json: value,
            warnings: vec![],
            wrapper_scripts: vec![],
        }),
        SourceFormat::ClaudeCode => {
            let mut result = value.clone();
            let mut warnings = Vec::new();
            let mut wrapper_scripts = Vec::new();

            convert_top_level(&mut result, &mut warnings);

            let new_hooks = convert_event_hooks(hooks_value, &mut warnings, &mut wrapper_scripts)?;
            result
                .as_object_mut()
                .unwrap()
                .insert("hooks".to_string(), new_hooks);

            Ok(ConvertResult {
                json: result,
                warnings,
                wrapper_scripts,
            })
        }
    }
}

/// BL-001: Detect input format.
///
/// Rules:
/// 1. If `version` key exists -> CopilotCli
/// 2. If first event key in `hooks` starts with uppercase -> ClaudeCode
/// 3. Otherwise -> CopilotCli
fn detect_format(value: &Value) -> SourceFormat {
    if value.get("version").is_some() {
        return SourceFormat::CopilotCli;
    }

    let is_pascal = value
        .get("hooks")
        .and_then(|h| h.as_object())
        .and_then(|hooks| hooks.keys().next())
        .is_some_and(|key| key.starts_with(|c: char| c.is_uppercase()));

    if is_pascal {
        SourceFormat::ClaudeCode
    } else {
        SourceFormat::CopilotCli
    }
}

/// BL-002: Convert top-level structure.
///
/// - Add `version: 1`
/// - Remove `disableAllHooks` with warning
fn convert_top_level(value: &mut Value, warnings: &mut Vec<ConversionWarning>) {
    let Some(obj) = value.as_object_mut() else {
        return;
    };

    obj.insert("version".to_string(), Value::from(1));

    if obj.remove("disableAllHooks").is_some() {
        warnings.push(ConversionWarning::RemovedField {
            field: "disableAllHooks".to_string(),
            reason: "Copilot CLI does not support disableAllHooks".to_string(),
        });
    }
}

/// BL-003 + BL-004: Convert event hooks with event name mapping and matcher flattening.
fn convert_event_hooks(
    hooks: &Value,
    warnings: &mut Vec<ConversionWarning>,
    wrapper_scripts: &mut Vec<WrapperScriptInfo>,
) -> Result<Value, PlmError> {
    let hooks_obj = hooks.as_object().unwrap();
    let mut output = serde_json::Map::new();

    for (event_name, event_value) in hooks_obj {
        match event_claude_to_copilot(event_name) {
            Some(copilot_event) => {
                let converted_hooks =
                    flatten_matchers(event_value, copilot_event, warnings, wrapper_scripts)?;
                if !converted_hooks.is_empty() {
                    output.insert(copilot_event.to_string(), Value::Array(converted_hooks));
                }
            }
            None => {
                warnings.push(ConversionWarning::UnsupportedEvent {
                    event: event_name.clone(),
                });
            }
        }
    }

    Ok(Value::Object(output))
}

/// BL-004: Flatten matcher groups into a flat array of hook definitions.
///
/// Claude Code format uses matcher groups:
/// ```json
/// { "matcher": "*.rs", "hooks": [{ ... }] }
/// ```
/// Copilot CLI uses a flat array with optional `steps` (file patterns).
fn flatten_matchers(
    groups: &Value,
    event: &str,
    warnings: &mut Vec<ConversionWarning>,
    wrapper_scripts: &mut Vec<WrapperScriptInfo>,
) -> Result<Vec<Value>, PlmError> {
    let mut result = Vec::new();

    let groups_arr = match groups.as_array() {
        Some(arr) => arr,
        None => return Ok(result),
    };

    for group in groups_arr {
        let matcher = group
            .get("matcher")
            .and_then(|m| m.as_str())
            .filter(|s| !s.is_empty());

        let hooks = match group.get("hooks").and_then(|h| h.as_array()) {
            Some(arr) => arr,
            None => continue,
        };

        for hook in hooks {
            if let Some(converted) =
                convert_hook_definition(hook, matcher, event, warnings, wrapper_scripts)?
            {
                result.push(converted);
            }
        }
    }

    Ok(result)
}

/// BL-005 + BL-006: Convert an individual hook definition.
fn convert_hook_definition(
    hook: &Value,
    matcher: Option<&str>,
    event: &str,
    warnings: &mut Vec<ConversionWarning>,
    wrapper_scripts: &mut Vec<WrapperScriptInfo>,
) -> Result<Option<Value>, PlmError> {
    let hook_type = hook
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("command");

    if let Some(m) = matcher {
        warnings.push(ConversionWarning::RemovedField {
            field: "matcher".to_string(),
            reason: format!(
                "Copilot CLI does not support matcher patterns; '{}' was dropped",
                m
            ),
        });
    }

    match hook_type {
        "command" => {
            let converted = convert_command_hook(hook, warnings)?;
            Ok(Some(converted))
        }
        "http" => {
            let converted = convert_http_hook(hook, event, matcher, wrapper_scripts)?;
            Ok(Some(converted))
        }
        "prompt" | "agent" => {
            let converted = convert_prompt_agent_hook(
                hook,
                hook_type,
                event,
                matcher,
                warnings,
                wrapper_scripts,
            );
            Ok(Some(converted))
        }
        unknown => {
            warnings.push(ConversionWarning::UnsupportedHookType {
                hook_type: unknown.to_string(),
                event: event.to_string(),
            });
            Ok(None)
        }
    }
}

/// BL-005: Convert a command-type hook (key name mapping).
///
/// - `command` -> `bash`
/// - `timeout` -> `timeoutSec`
/// - `statusMessage` -> `comment`
/// - `async` -> removed with warning
/// - `once` -> removed
/// - `type` -> removed (implicit in Copilot CLI)
fn convert_command_hook(
    hook: &Value,
    warnings: &mut Vec<ConversionWarning>,
) -> Result<Value, PlmError> {
    let hook_obj = hook
        .as_object()
        .ok_or_else(|| PlmError::HookConversion("Hook definition must be an object".to_string()))?;

    if !hook_obj.contains_key("command") {
        return Err(PlmError::HookConversion(
            "command hook missing required 'command' field".to_string(),
        ));
    }

    let mut output = serde_json::Map::new();

    for (key, value) in hook_obj {
        match key.as_str() {
            "command" => {
                output.insert("bash".to_string(), value.clone());
            }
            "timeout" => {
                output.insert("timeoutSec".to_string(), value.clone());
            }
            "statusMessage" => {
                output.insert("comment".to_string(), value.clone());
            }
            "async" => {
                warnings.push(ConversionWarning::RemovedField {
                    field: "async".to_string(),
                    reason: "Copilot CLI does not support async hooks".to_string(),
                });
            }
            "once" => {
                // Silently removed - not supported in Copilot CLI
            }
            "type" => {
                // Copilot CLI requires "type": "command"
                output.insert("type".to_string(), Value::from("command"));
            }
            _ => {
                output.insert(key.clone(), value.clone());
            }
        }
    }

    Ok(Value::Object(output))
}

/// BL-006: Convert an http-type hook to a curl wrapper script.
fn convert_http_hook(
    hook: &Value,
    event: &str,
    matcher: Option<&str>,
    wrapper_scripts: &mut Vec<WrapperScriptInfo>,
) -> Result<Value, PlmError> {
    let hook_obj = hook
        .as_object()
        .ok_or_else(|| PlmError::HookConversion("Hook definition must be an object".to_string()))?;

    let url = hook_obj
        .get("url")
        .and_then(|u| u.as_str())
        .ok_or_else(|| {
            PlmError::HookConversion("http hook missing required 'url' field".to_string())
        })?;

    let method = hook_obj
        .get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("POST");

    let script_name = format!("http-{}-{}.sh", event, wrapper_scripts.len());

    let mut headers_lines = String::new();
    if let Some(headers) = hook_obj.get("headers").and_then(|h| h.as_object()) {
        for (k, v) in headers {
            if let Some(v_str) = v.as_str() {
                headers_lines.push_str(&format!("  -H '{}: {}' \\\n", k, v_str));
            }
        }
    }

    let body_line = if hook_obj.get("body").is_some() {
        "  -d \"$HOOK_INPUT\" \\\n"
    } else {
        ""
    };

    let script_content = format!(
        "#!/bin/bash\nset -euo pipefail\n\n{}\n\ncurl -s -X {} \\\n{}{}  '{}'\n",
        ENV_BRIDGE, method, headers_lines, body_line, url
    );

    wrapper_scripts.push(WrapperScriptInfo {
        path: script_name.clone(),
        content: script_content,
        original_config: hook.clone(),
        matcher: matcher.map(|s| s.to_string()),
    });

    let mut output = serde_json::Map::new();
    output.insert("type".to_string(), Value::from("command"));
    output.insert(
        "bash".to_string(),
        Value::from(format!("./{}", script_name)),
    );

    if let Some(timeout) = hook_obj.get("timeout") {
        output.insert("timeoutSec".to_string(), timeout.clone());
    }

    Ok(Value::Object(output))
}

/// BL-006: Convert a prompt/agent hook to a stub script with warning.
fn convert_prompt_agent_hook(
    hook: &Value,
    hook_type: &str,
    event: &str,
    matcher: Option<&str>,
    warnings: &mut Vec<ConversionWarning>,
    wrapper_scripts: &mut Vec<WrapperScriptInfo>,
) -> Value {
    let script_name = format!("{}-{}-{}.sh", hook_type, event, wrapper_scripts.len());

    let original_json = serde_json::to_string_pretty(hook).unwrap_or_default();

    let script_content = format!(
        "#!/bin/bash\nset -euo pipefail\n\n{}\n\n# TODO: This is a stub for a Claude Code '{}' hook.\n# prompt/agent hooks are Claude Code-specific features.\n# Please manually rewrite as scripts.\n#\n# Original configuration:\n# {}\n\necho \"STUB: {} hook for event '{}' - please implement manually\" >&2\nexit 1\n",
        ENV_BRIDGE,
        hook_type,
        original_json.replace('\n', "\n# "),
        hook_type,
        event
    );

    wrapper_scripts.push(WrapperScriptInfo {
        path: script_name.clone(),
        content: script_content,
        original_config: hook.clone(),
        matcher: matcher.map(|s| s.to_string()),
    });

    warnings.push(ConversionWarning::PromptAgentHookStub {
        event: event.to_string(),
        hook_type: hook_type.to_string(),
    });

    let mut output = serde_json::Map::new();
    output.insert("type".to_string(), Value::from("command"));
    output.insert(
        "bash".to_string(),
        Value::from(format!("./{}", script_name)),
    );
    Value::Object(output)
}
