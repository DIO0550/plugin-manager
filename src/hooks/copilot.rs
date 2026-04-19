//! Copilot CLI implementation of the hook conversion layers.
//!
//! EventMap is in `event/copilot.rs`; ToolMap is in `tool/copilot.rs`.
//! This file retains KeyMap, StructureConverter, and ScriptGenerator.

use serde_json::Value;

use crate::error::PlmError;
use crate::hooks::converter::{
    generate_matcher_filter, shell_escape, ConversionWarning, ScriptInfo, SourceFormat, SCRIPTS_DIR,
};
use crate::hooks::hook_definition::{CommandHook, HttpHook, StubHook};

use super::converter::{KeyMap, ScriptGenerator, StructureConverter};
pub(crate) use super::event::copilot::CopilotEventMap;

/// Exit code and stdout conversion logic for command hook scripts.
const EXIT_CODE_HANDLER: &str = r#"# --- execute original command and capture result ---
PLM_STDOUT_FILE="$(mktemp)"
PLM_STDERR_FILE="$(mktemp)"
trap 'rm -f "$PLM_STDOUT_FILE" "$PLM_STDERR_FILE"' EXIT
set +e
printf '%s' "$CLAUDE_INPUT" | eval "$ORIGINAL_CMD" >"$PLM_STDOUT_FILE" 2>"$PLM_STDERR_FILE"
EXIT_CODE=$?
set -e
RESULT=$(cat "$PLM_STDOUT_FILE" 2>/dev/null || echo "")
STDERR=$(cat "$PLM_STDERR_FILE" 2>/dev/null || echo "")

# --- exit code + stdout conversion ---
# Copilot CLI only parses stdout for preToolUse; other events ignore it.
if [ "$EXIT_CODE" -eq 0 ] && [ -n "$RESULT" ] && [ "$HOOK_EVENT" = "preToolUse" ]; then
  if command -v jq >/dev/null 2>&1; then
    printf '%s' "$RESULT" | jq '
      if .hookSpecificOutput then
        .hookSpecificOutput
        | del(.hookEventName, .updatedInput, .additionalContext, .continue, .stopReason, .suppressOutput, .systemMessage)
        | if has("permissionDecision") then
            {permissionDecision, permissionDecisionReason}
          else . end
      else . end
    ' 2>/dev/null || true
  else
    echo "plm: warning: jq not found; suppressing hook stdout to avoid invalid Copilot JSON" >&2
  fi
elif [ "$EXIT_CODE" -eq 2 ] && [ "$HOOK_EVENT" = "preToolUse" ]; then
  REASON="${STDERR:-Blocked by hook}"
  if command -v jq >/dev/null 2>&1; then
    jq -n --arg reason "$REASON" '{"permissionDecision":"deny","permissionDecisionReason":$reason}'
  else
    printf '%s\n' '{"permissionDecision":"deny"}'
  fi
fi
exit 0"#;

pub(crate) struct CopilotKeyMap;

impl KeyMap for CopilotKeyMap {
    fn map_keys(&self, hook: &Value, hook_type: &str) -> (Value, Vec<ConversionWarning>) {
        let Some(hook_obj) = hook.as_object() else {
            return (hook.clone(), vec![]);
        };

        let mut output = serde_json::Map::new();
        let mut warnings = Vec::new();

        // Keys that are handled by ScriptGenerator or orchestrator — skip them
        let type_specific_keys: &[&str] = match hook_type {
            "command" => &["command", "type"],
            "http" => &["type", "url", "method", "headers", "body"],
            "prompt" => &["type", "prompt"],
            "agent" => &["type", "agent"],
            _ => &["type"],
        };

        for (key, value) in hook_obj {
            if type_specific_keys.contains(&key.as_str()) {
                continue;
            }

            match key.as_str() {
                "once" => {
                    warnings.push(ConversionWarning::RemovedField {
                        field: "once".to_string(),
                        reason: "Copilot CLI does not support once hooks".to_string(),
                    });
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
                _ => {
                    output.insert(key.clone(), value.clone());
                }
            }
        }

        (Value::Object(output), warnings)
    }
}

pub(crate) struct CopilotStructureConverter;

impl StructureConverter for CopilotStructureConverter {
    fn detect_format(&self, value: &Value) -> SourceFormat {
        if value.get("version").is_some() {
            return SourceFormat::TargetFormat;
        }

        let Some(hooks_obj) = value.get("hooks").and_then(|h| h.as_object()) else {
            return SourceFormat::TargetFormat;
        };

        let has_pascal = hooks_obj
            .keys()
            .any(|key| key.chars().next().is_some_and(|c| c.is_uppercase()));

        if has_pascal {
            SourceFormat::ClaudeCode
        } else {
            SourceFormat::TargetFormat
        }
    }

    fn handle_target_format(
        &self,
        value: Value,
    ) -> Result<(Value, Vec<ConversionWarning>), PlmError> {
        let mut warnings = Vec::new();
        let mut result = value;

        match result.get("version") {
            Some(v) => {
                let num = v.as_i64().ok_or_else(|| {
                    PlmError::HookConversion(format!(
                        "Invalid 'version' type: expected integer 1, got {}",
                        v
                    ))
                })?;
                if num != 1 {
                    return Err(PlmError::HookConversion(format!(
                        "Unsupported hooks config 'version': {} (only 1 is supported)",
                        num
                    )));
                }
            }
            None => {
                warnings.push(ConversionWarning::MissingVersion);
                if let Some(obj) = result.as_object_mut() {
                    obj.insert("version".to_string(), Value::from(1));
                }
            }
        }

        Ok((result, warnings))
    }

    fn convert_top_level(&self, value: &Value) -> (Value, Vec<ConversionWarning>) {
        let mut result = value.clone();
        let mut warnings = Vec::new();

        let Some(obj) = result.as_object_mut() else {
            return (result, warnings);
        };

        obj.insert("version".to_string(), Value::from(1));

        if obj.remove("disableAllHooks").is_some() {
            warnings.push(ConversionWarning::RemovedField {
                field: "disableAllHooks".to_string(),
                reason: "Copilot CLI does not support disableAllHooks".to_string(),
            });
        }

        (result, warnings)
    }
}

pub(crate) struct CopilotScriptGenerator;

impl ScriptGenerator for CopilotScriptGenerator {
    fn generate_command_script(
        &self,
        hook: &CommandHook<'_>,
        event: &str,
        matcher: Option<&str>,
        index: usize,
    ) -> ScriptInfo {
        let script_name = format!("cmd-{}-{}.sh", event, index);
        let matcher_filter = generate_matcher_filter(matcher);

        let script_content = format!(
            "#!/bin/bash\nset -euo pipefail\n\nHOOK_EVENT='{}'\n\n{}\n{}\nORIGINAL_CMD='{}'\n\n{}\n",
            shell_escape(event),
            &build_env_bridge(event),
            matcher_filter,
            shell_escape(hook.command),
            EXIT_CODE_HANDLER
        );

        ScriptInfo {
            path: format!("{}/{}", SCRIPTS_DIR, script_name),
            content: script_content,
            original_config: Value::Null,
            matcher: matcher.map(|s| s.to_string()),
        }
    }

    fn generate_http_script(
        &self,
        hook: &HttpHook<'_>,
        event: &str,
        matcher: Option<&str>,
        index: usize,
    ) -> Result<ScriptInfo, PlmError> {
        let script_name = format!("http-{}-{}.sh", event, index);
        let headers_lines = build_headers_lines(&hook.headers);
        let matcher_filter = generate_matcher_filter(matcher);

        let script_content = format!(
            r#"#!/bin/bash
set -euo pipefail

HOOK_EVENT='{}'

{}
{}
# --- http hook: {} {} ---
{}

HTTP_BODY=$(printf '%s' "$HTTP_RESPONSE" | sed '$d')
HTTP_CODE=$(printf '%s' "$HTTP_RESPONSE" | tail -1)

if [ "$HTTP_CODE" -ge 200 ] 2>/dev/null && [ "$HTTP_CODE" -lt 300 ] 2>/dev/null; then
  if [ -n "$HTTP_BODY" ] && [ "$HOOK_EVENT" = "preToolUse" ] && command -v jq >/dev/null 2>&1; then
    printf '%s' "$HTTP_BODY" | jq '
      if .hookSpecificOutput then
        .hookSpecificOutput
        | del(.hookEventName, .updatedInput, .additionalContext, .continue, .stopReason, .suppressOutput, .systemMessage)
        | if has("permissionDecision") then
            {{permissionDecision, permissionDecisionReason}}
          else . end
      else . end
    ' 2>/dev/null || true
  fi
else
  echo "plm: warning: http hook returned status $HTTP_CODE" >&2
fi
exit 0
"#,
            shell_escape(event),
            &build_env_bridge(event),
            matcher_filter,
            hook.method,
            hook.url.replace('\n', "\\n").replace('\r', "\\r"),
            {
                let escaped_url = shell_escape(hook.url);
                if matches!(hook.method.as_str(), "GET" | "HEAD" | "OPTIONS") {
                    format!(
                        "HTTP_RESPONSE=$(curl -s -w '\\n%{{http_code}}' -X {} \\\n{}  '{}' 2>/dev/null || printf '\\n000')",
                        hook.method, headers_lines, escaped_url
                    )
                } else {
                    format!(
                        "HTTP_RESPONSE=$(printf '%s' \"$CLAUDE_INPUT\" | curl -s -w '\\n%{{http_code}}' -X {} \\\n{}  -d @- \\\n  '{}' 2>/dev/null || printf '\\n000')",
                        hook.method, headers_lines, escaped_url
                    )
                }
            }
        );

        Ok(ScriptInfo {
            path: format!("{}/{}", SCRIPTS_DIR, script_name),
            content: script_content,
            original_config: hook.raw.clone(),
            matcher: matcher.map(|s| s.to_string()),
        })
    }

    fn generate_stub_script(
        &self,
        hook: &StubHook<'_>,
        event: &str,
        matcher: Option<&str>,
        index: usize,
    ) -> ScriptInfo {
        let script_name = format!("{}-{}-{}.sh", hook.hook_type, event, index);
        let original_json = serde_json::to_string_pretty(hook.raw).unwrap_or_default();
        let matcher_filter = generate_matcher_filter(matcher);

        let script_content = format!(
            "#!/bin/bash\nset -euo pipefail\n\n{}\n{}\n# TODO: This is a stub for a Claude Code '{}' hook.\n# prompt/agent hooks are Claude Code-specific features.\n# Please manually rewrite as scripts.\n#\n# Original configuration:\n# {}\n\necho \"STUB: {} hook for event '{}' - please implement manually\" >&2\nexit 0\n",
            &build_env_bridge(event),
            matcher_filter,
            hook.hook_type,
            original_json.replace('\n', "\n# "),
            hook.hook_type,
            event
        );

        ScriptInfo {
            path: format!("{}/{}", SCRIPTS_DIR, script_name),
            content: script_content,
            original_config: hook.raw.clone(),
            matcher: matcher.map(|s| s.to_string()),
        }
    }
}

/// Build event-specific environment variable bridge for scripts.
///
/// # Arguments
///
/// * `event` - Hook event name (e.g. `preToolUse`, `postToolUse`, `sessionStart`).
fn build_env_bridge(event: &str) -> String {
    let base_jq = r#"    . as $in | $in
    # Remove Copilot-specific timestamp
    | del(.timestamp)
    # Normalize session identifier
    | .session_id = ($in.sessionId // $in.session_id // "plm-bridge")"#;

    let tool_common_jq = r#"
    # Map tool name (BL-002)
    | .tool_name = (
        if $in.toolName then
          {bash:"Bash",powershell:"Bash",view:"Read",create:"Write",edit:"Edit",
           glob:"Glob",grep:"Grep",web_fetch:"WebFetch",task:"Agent"}[$in.toolName] // $in.toolName
        else ($in.tool_name // null) end
      )
    # Normalize tool input
    | .tool_input = (
        if $in.toolArgs then ($in.toolArgs | try fromjson catch {})
        elif $in.tool_input then $in.tool_input
        else null end
      )"#;

    let tool_response_jq = r#"
    # Normalize tool response (PostToolUse only)
    | .tool_response = (
        if $in.toolResult then $in.toolResult
        elif $in.tool_response then $in.tool_response
        else null end
      )"#;

    let jq_body = match event {
        "preToolUse" => format!(
            "{}{}\n    # Clean up Copilot-specific keys that have been normalized\n    | del(.toolName, .toolArgs, .sessionId)",
            base_jq, tool_common_jq
        ),
        "postToolUse" => format!(
            "{}{}{}\n    # Clean up Copilot-specific keys that have been normalized\n    | del(.toolName, .toolArgs, .toolResult, .sessionId)",
            base_jq, tool_common_jq, tool_response_jq
        ),
        _ => format!(
            "{}\n    # Clean up Copilot-specific keys\n    | del(.sessionId)",
            base_jq
        ),
    };

    let session_start_extra = if event == "sessionStart" {
        r#"
# sessionStart-specific: source mapping + del(.initialPrompt)
if [ "$HAS_JQ" = true ]; then
  if SESSION_TRANSFORMED=$(printf '%s' "$CLAUDE_INPUT" | jq '
    (if has("source") then .source |= (if . == "new" then "startup" elif . == "resume" then "resume" else . end) else . end)
    | del(.initialPrompt)
  ' 2>/dev/null); then
    CLAUDE_INPUT="$SESSION_TRANSFORMED"
  else
    echo "plm: warning: failed to apply sessionStart-specific transformation" >&2
  fi
fi
"#
    } else {
        ""
    };

    format!(
        r#"COPILOT_INPUT=$(cat)
CLAUDE_INPUT="$COPILOT_INPUT"

HAS_JQ=false
command -v jq >/dev/null 2>&1 && HAS_JQ=true

if [ "$HAS_JQ" = true ]; then
  if TRANSFORMED=$(printf '%s' "$COPILOT_INPUT" | jq '
{}
  ' 2>/dev/null); then
    CLAUDE_INPUT="$TRANSFORMED"
  else
    echo "plm: warning: failed to transform hook stdin JSON; passing through original payload" >&2
  fi
else
  echo "plm: warning: jq not found; passing through original hook stdin JSON" >&2
fi
{}
CLAUDE_PROJECT_DIR=""
if [ "$HAS_JQ" = true ]; then
  CLAUDE_PROJECT_DIR=$(printf '%s' "$CLAUDE_INPUT" | jq -r '.cwd // empty' 2>/dev/null || echo "")
fi
export CLAUDE_PROJECT_DIR
export CLAUDE_PLUGIN_ROOT="@@PLUGIN_ROOT@@""#,
        jq_body, session_start_extra
    )
}

/// Build HTTP headers lines for curl command from validated headers.
///
/// # Arguments
///
/// * `headers` - Slice of `(name, value)` header pairs to emit as `-H` flags.
fn build_headers_lines(headers: &[(&str, &str)]) -> String {
    let mut headers_lines = String::new();

    if headers.is_empty() {
        return "  -H \"Content-Type: application/json\" \\\n".to_string();
    }

    for (k, v) in headers {
        let escaped_value = v.replace('\\', "\\\\").replace('"', "\\\"");
        headers_lines.push_str(&format!("  -H \"{}: {}\" \\\n", k, escaped_value));
    }

    let has_content_type = headers
        .iter()
        .any(|(k, _)| k.eq_ignore_ascii_case("content-type"));
    if !has_content_type {
        headers_lines = format!(
            "  -H \"Content-Type: application/json\" \\\n{}",
            headers_lines
        );
    }

    headers_lines
}
