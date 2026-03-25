//! Codex skeleton implementation of the 4 hook conversion layers.
//!
//! These are placeholder implementations. `EventMap` returns `None` for all events,
//! `KeyMap` passes through as-is, `StructureConverter` performs no transformation,
//! and `ScriptGenerator` returns empty stubs (or errors for http).
//! These skeletons are not wired into production via `create_layers()`;
//! they exist as a structural foundation for future Codex hook conversion.

use serde_json::Value;

use crate::error::PlmError;
use crate::hooks::converter::{ConversionWarning, ScriptInfo, SourceFormat};

use super::converter::{EventMap, KeyMap, ScriptGenerator, StructureConverter};

// ============================================================================
// EventMap
// ============================================================================

pub(crate) struct CodexEventMap;

impl EventMap for CodexEventMap {
    fn map_event(&self, _event: &str) -> Option<&'static str> {
        None
    }
}

// ============================================================================
// KeyMap
// ============================================================================

pub(crate) struct CodexKeyMap;

impl KeyMap for CodexKeyMap {
    fn map_keys(&self, hook: &Value, _hook_type: &str) -> (Value, Vec<ConversionWarning>) {
        (hook.clone(), vec![])
    }
}

// ============================================================================
// StructureConverter
// ============================================================================

pub(crate) struct CodexStructureConverter;

impl StructureConverter for CodexStructureConverter {
    fn detect_format(&self, _value: &Value) -> SourceFormat {
        SourceFormat::ClaudeCode
    }

    fn handle_target_format(&self, value: Value) -> (Value, Vec<ConversionWarning>) {
        (value, vec![])
    }

    fn convert_top_level(&self, value: &Value) -> (Value, Vec<ConversionWarning>) {
        (value.clone(), vec![])
    }
}

// ============================================================================
// ScriptGenerator
// ============================================================================

pub(crate) struct CodexScriptGenerator;

impl ScriptGenerator for CodexScriptGenerator {
    fn generate_command_script(
        &self,
        _command: &str,
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
        _hook: &Value,
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
        _hook: &Value,
        _hook_type: &str,
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
