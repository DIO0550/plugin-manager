//! Codex skeleton implementation of the hook conversion layers.
//!
//! EventMap is in `event/codex.rs`; ToolMap is in `tool/codex.rs`.
//! KeyMap passes through as-is, StructureConverter performs no transformation,
//! and ScriptGenerator returns empty stubs (or errors for http).

use serde_json::Value;

use super::super::model::{CommandHook, HttpHook, StubHook};
use crate::error::PlmError;
use crate::hooks::converter::{ConversionWarning, ScriptInfo, SourceFormat};

use super::converter::{KeyMap, ScriptGenerator, StructureConverter};

pub(crate) struct CodexKeyMap;

impl KeyMap for CodexKeyMap {
    fn map_keys(&self, hook: &Value, _hook_type: &str) -> (Value, Vec<ConversionWarning>) {
        (hook.clone(), vec![])
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
        (value.clone(), vec![])
    }
}

pub(crate) struct CodexScriptGenerator;

impl ScriptGenerator for CodexScriptGenerator {
    fn generate_command_script(
        &self,
        _hook: &CommandHook<'_>,
        _event: &str,
        _matcher: Option<&str>,
        _index: usize,
    ) -> ScriptInfo {
        ScriptInfo {
            path: String::new(),
            content: String::new(),
            original_config: Value::Null,
            matcher: None,
        }
    }

    fn generate_http_script(
        &self,
        _hook: &HttpHook<'_>,
        _event: &str,
        _matcher: Option<&str>,
        _index: usize,
    ) -> Result<ScriptInfo, PlmError> {
        Err(PlmError::HookConversion(
            "Codex hook conversion is not yet implemented".to_string(),
        ))
    }

    fn generate_stub_script(
        &self,
        _hook: &StubHook<'_>,
        _event: &str,
        _matcher: Option<&str>,
        _index: usize,
    ) -> ScriptInfo {
        ScriptInfo {
            path: String::new(),
            content: String::new(),
            original_config: Value::Null,
            matcher: None,
        }
    }
}
