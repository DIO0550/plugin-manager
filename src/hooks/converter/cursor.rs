//! Cursor implementation of the hook conversion layers.
//!
//! EventMap is in `event/cursor.rs`; ToolMap is in `tool/cursor.rs`.
//! Cursor `hooks.json` looks like Copilot (`version: 1` + camelCase events)
//! but keeps Claude-compatible field names (`command`, `timeout`) and
//! supports entry-level `matcher` on the flat array.

use serde_json::Value;

use super::super::model::{CommandHook, HttpHook, StubHook};
use crate::error::PlmError;
use crate::hooks::converter::{ConversionWarning, ScriptInfo, SourceFormat};

use super::converter::{KeyMap, ScriptGenerator, StructureConverter};

pub(crate) use super::super::event::cursor::CursorEventMap;

pub(crate) struct CursorKeyMap;

impl KeyMap for CursorKeyMap {
    fn map_keys(&self, hook: &Value, hook_type: &str) -> (Value, Vec<ConversionWarning>) {
        let Some(hook_obj) = hook.as_object() else {
            return (hook.clone(), vec![]);
        };

        let mut output = serde_json::Map::new();
        let mut warnings = Vec::new();

        // Fields consumed by ScriptGenerator / orchestrator when wrappers exist.
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
                "async" => {
                    warnings.push(ConversionWarning::RemovedField {
                        field: "async".to_string(),
                        reason: "Cursor hooks do not support async hooks".to_string(),
                    });
                }
                "once" => {
                    warnings.push(ConversionWarning::RemovedField {
                        field: "once".to_string(),
                        reason: "Cursor hooks do not support once hooks".to_string(),
                    });
                }
                "bash" => {
                    warnings.push(ConversionWarning::RemovedField {
                        field: "bash".to_string(),
                        reason: "Cursor hooks use command, not bash".to_string(),
                    });
                }
                "timeoutSec" if !output.contains_key("timeout") => {
                    output.insert("timeout".to_string(), value.clone());
                }
                "timeoutSec" => {}
                "statusMessage" | "comment" => {
                    warnings.push(ConversionWarning::RemovedField {
                        field: key.clone(),
                        reason: "Cursor hooks do not support statusMessage/comment".to_string(),
                    });
                }
                _ => {
                    output.insert(key.clone(), value.clone());
                }
            }
        }

        // Restore type-specific required fields for inline Cursor config.
        match hook_type {
            "command" => {
                if let Some(command) = hook_obj.get("command") {
                    output.insert("command".to_string(), command.clone());
                }
                output.insert("type".to_string(), Value::from("command"));
            }
            "prompt" => {
                if let Some(prompt) = hook_obj.get("prompt") {
                    output.insert("prompt".to_string(), prompt.clone());
                }
                output.insert("type".to_string(), Value::from("prompt"));
            }
            "agent" => {
                if let Some(agent) = hook_obj.get("agent") {
                    output.insert("agent".to_string(), agent.clone());
                }
                output.insert("type".to_string(), Value::from("agent"));
            }
            _ => {}
        }

        (Value::Object(output), warnings)
    }
}

pub(crate) struct CursorStructureConverter;

impl StructureConverter for CursorStructureConverter {
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
                reason: "Cursor hooks do not support disableAllHooks".to_string(),
            });
        }

        (result, warnings)
    }
}

pub(crate) struct CursorScriptGenerator;

impl ScriptGenerator for CursorScriptGenerator {
    fn generate_command_script(
        &self,
        hook: &CommandHook<'_>,
        _event: &str,
        _matcher: Option<&str>,
        _index: usize,
    ) -> ScriptInfo {
        // Cursor stdin/stdout and exit code 2 are Claude-compatible, so keep
        // the original command inline without a wrapper script.
        ScriptInfo {
            path: String::new(),
            content: String::new(),
            original_config: hook.raw.clone(),
            matcher: None,
        }
    }

    fn generate_http_script(
        &self,
        hook: &HttpHook<'_>,
        _event: &str,
        matcher: Option<&str>,
        _index: usize,
    ) -> Result<ScriptInfo, PlmError> {
        // Phase A: http hooks are unsupported (empty path → excluded).
        Ok(ScriptInfo {
            path: String::new(),
            content: String::new(),
            original_config: hook.raw.clone(),
            matcher: matcher.map(|s| s.to_string()),
        })
    }

    fn generate_stub_script(
        &self,
        hook: &StubHook<'_>,
        _event: &str,
        matcher: Option<&str>,
        _index: usize,
    ) -> ScriptInfo {
        ScriptInfo {
            path: String::new(),
            content: String::new(),
            original_config: hook.raw.clone(),
            matcher: matcher.map(|s| s.to_string()),
        }
    }

    fn preserves_stub_inline(&self) -> bool {
        // Cursor natively supports prompt hooks; agent is kept inline with a
        // warning so users can rewrite manually (same pattern as Codex).
        true
    }
}
