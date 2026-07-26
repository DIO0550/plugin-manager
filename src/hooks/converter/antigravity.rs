//! Antigravity implementation of the hook conversion layers.
//!
//! EventMap is in `event/antigravity.rs`; ToolMap is in `tool/antigravity.rs`.
//! Antigravity uses a top-level **named hook → event map** structure (unlike
//! Claude Code / Copilot / Codex `hooks.<Event>`). Matcher-group vs flat
//! handler shape is decided by [`AntigravityEventMap::preserve_matcher_groups`].

use serde_json::Value;

use super::super::model::{CommandHook, HttpHook, StubHook};
use crate::error::PlmError;
use crate::hooks::converter::{ConversionWarning, ScriptInfo, SourceFormat};

use super::converter::{KeyMap, ScriptGenerator, StructureConverter};

pub(crate) use super::super::event::antigravity::AntigravityEventMap;

/// Default key used when wrapping converted events under a named hook.
/// Deploy renames the single root key to the component's sanitized name.
pub(crate) const DEFAULT_HOOK_NAME: &str = "converted";

pub(crate) struct AntigravityKeyMap;

impl KeyMap for AntigravityKeyMap {
    fn map_keys(&self, hook: &Value, hook_type: &str) -> (Value, Vec<ConversionWarning>) {
        let Some(hook_obj) = hook.as_object() else {
            return (hook.clone(), vec![]);
        };

        let mut output = serde_json::Map::new();
        let mut warnings = Vec::new();

        for (key, value) in hook_obj {
            match key.as_str() {
                "async" => warnings.push(ConversionWarning::RemovedField {
                    field: "async".to_string(),
                    reason: "Antigravity hooks do not support async hooks".to_string(),
                }),
                "once" => warnings.push(ConversionWarning::RemovedField {
                    field: "once".to_string(),
                    reason: "Antigravity hooks do not support once hooks".to_string(),
                }),
                "bash" => warnings.push(ConversionWarning::RemovedField {
                    field: "bash".to_string(),
                    reason: "Antigravity hooks use command, not bash".to_string(),
                }),
                "timeoutSec" if !output.contains_key("timeout") => {
                    output.insert("timeout".to_string(), value.clone());
                }
                "timeoutSec" => {}
                "statusMessage" | "comment" => {
                    warnings.push(ConversionWarning::RemovedField {
                        field: key.clone(),
                        reason: "Antigravity hooks do not support statusMessage/comment"
                            .to_string(),
                    });
                }
                "command_windows" | "commandWindows" => {
                    warnings.push(ConversionWarning::RemovedField {
                        field: key.clone(),
                        reason: "Antigravity hooks do not support Windows command fields"
                            .to_string(),
                    });
                }
                _ => {
                    output.insert(key.clone(), value.clone());
                }
            }
        }

        if hook_type == "command" {
            if let Some(command) = hook_obj.get("command") {
                output.insert("command".to_string(), command.clone());
            }
            output.insert("type".to_string(), Value::from("command"));
        }

        (Value::Object(output), warnings)
    }
}

pub(crate) struct AntigravityStructureConverter;

impl StructureConverter for AntigravityStructureConverter {
    fn validate_input(&self, value: &Value) -> Result<(), PlmError> {
        // Native format is a named-hook map (no top-level `hooks`). Claude Code
        // input still has `hooks` and is accepted here; ClaudeCode branch
        // re-validates that object before conversion.
        if value.is_object() {
            Ok(())
        } else {
            Err(PlmError::HookConversion(
                "Hooks config must be a JSON object".to_string(),
            ))
        }
    }

    fn detect_format(&self, value: &Value) -> SourceFormat {
        // Claude Code / Copilot / Codex shaped input has a top-level `hooks` object.
        if value.get("hooks").and_then(|h| h.as_object()).is_some() {
            return SourceFormat::ClaudeCode;
        }
        // Native Antigravity: top-level named-hook map (no `hooks` wrapper).
        SourceFormat::TargetFormat
    }

    fn handle_target_format(
        &self,
        value: Value,
    ) -> Result<(Value, Vec<ConversionWarning>), PlmError> {
        Ok((value, vec![]))
    }

    fn convert_top_level(&self, value: &Value) -> (Value, Vec<ConversionWarning>) {
        let mut warnings = Vec::new();

        if value.get("version").is_some() {
            warnings.push(ConversionWarning::RemovedField {
                field: "version".to_string(),
                reason: "Antigravity hooks do not use Copilot CLI's version field".to_string(),
            });
        }

        if value.get("disableAllHooks").is_some() {
            warnings.push(ConversionWarning::RemovedField {
                field: "disableAllHooks".to_string(),
                reason: "Antigravity hooks use per-hook enabled:false instead of disableAllHooks"
                    .to_string(),
            });
        }

        // `assemble` replaces the root with a named-hook map.
        (Value::Object(serde_json::Map::new()), warnings)
    }

    fn assemble(&self, _top_level: Value, events: Value) -> Value {
        let mut root = serde_json::Map::new();
        root.insert(DEFAULT_HOOK_NAME.to_string(), events);
        Value::Object(root)
    }
}

pub(crate) struct AntigravityScriptGenerator;

impl ScriptGenerator for AntigravityScriptGenerator {
    fn generate_command_script(
        &self,
        hook: &CommandHook<'_>,
        _event: &str,
        _matcher: Option<&str>,
        _index: usize,
    ) -> ScriptInfo {
        // Keep commands inline (structure conversion only).
        // stdin/stdout bridging wrappers are a follow-up if needed.
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
        false
    }
}
