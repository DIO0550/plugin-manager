//! Codex implementation of the hook conversion layers.
//!
//! EventMap is in `event/codex.rs`; ToolMap is in `tool/codex.rs`.
//! Codex Hooks keep the Claude Code-style PascalCase event and matcher-group
//! structure, so this layer mostly validates and strips unsupported fields.

use serde_json::Value;

use super::super::model::{CommandHook, HttpHook, StubHook};
use crate::error::PlmError;
use crate::hooks::converter::{ConversionWarning, ScriptInfo, SourceFormat};

use super::converter::{KeyMap, ScriptGenerator, StructureConverter};

pub(crate) use super::super::event::codex::CodexEventMap;

pub(crate) struct CodexKeyMap;

impl KeyMap for CodexKeyMap {
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
                    reason: "Codex hooks do not support async hooks".to_string(),
                }),
                "once" => warnings.push(ConversionWarning::RemovedField {
                    field: "once".to_string(),
                    reason: "Codex hooks do not support once hooks".to_string(),
                }),
                // Copilot-specific fields should not leak into Codex hooks.
                "bash" => warnings.push(ConversionWarning::RemovedField {
                    field: "bash".to_string(),
                    reason: "Codex hooks use command, not bash wrapper paths".to_string(),
                }),
                "timeoutSec" if !output.contains_key("timeout") => {
                    output.insert("timeout".to_string(), value.clone());
                }
                "timeoutSec" => {}
                "comment" => {
                    output.insert("statusMessage".to_string(), value.clone());
                }
                _ => {
                    output.insert(key.clone(), value.clone());
                }
            }
        }

        if hook_type == "command" {
            output.insert("type".to_string(), Value::from("command"));
        }

        (Value::Object(output), warnings)
    }
}

pub(crate) struct CodexStructureConverter;

impl StructureConverter for CodexStructureConverter {
    fn detect_format(&self, _value: &Value) -> SourceFormat {
        SourceFormat::ClaudeCode
    }

    fn handle_target_format(
        &self,
        value: Value,
    ) -> Result<(Value, Vec<ConversionWarning>), PlmError> {
        Ok((value, vec![]))
    }

    fn convert_top_level(&self, value: &Value) -> (Value, Vec<ConversionWarning>) {
        let mut result = value.clone();
        let mut warnings = Vec::new();

        let Some(obj) = result.as_object_mut() else {
            return (result, warnings);
        };

        if obj.remove("version").is_some() {
            warnings.push(ConversionWarning::RemovedField {
                field: "version".to_string(),
                reason: "Codex hooks do not use Copilot CLI's version field".to_string(),
            });
        }

        if obj.remove("disableAllHooks").is_some() {
            warnings.push(ConversionWarning::RemovedField {
                field: "disableAllHooks".to_string(),
                reason: "Codex hooks do not support disableAllHooks".to_string(),
            });
        }

        (result, warnings)
    }
}

pub(crate) struct CodexScriptGenerator;

impl ScriptGenerator for CodexScriptGenerator {
    fn generate_command_script(
        &self,
        hook: &CommandHook<'_>,
        _event: &str,
        _matcher: Option<&str>,
        _index: usize,
    ) -> ScriptInfo {
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
}
