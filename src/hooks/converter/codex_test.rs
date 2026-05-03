//! Unit tests for Codex skeleton conversion layers.

use serde_json::json;

use super::super::event::codex::CodexEventMap;
use super::super::model::HttpHook;
use super::codex::{CodexKeyMap, CodexScriptGenerator, CodexStructureConverter};
use super::converter::{EventMap, KeyMap, ScriptGenerator, SourceFormat, StructureConverter};

#[test]
fn test_codex_event_map_returns_none() {
    let map = CodexEventMap;
    assert_eq!(map.map_event("SessionStart"), None);
    assert_eq!(map.map_event("PreToolUse"), None);
}

#[test]
fn test_codex_key_map_passthrough() {
    let map = CodexKeyMap;
    let hook = json!({"type": "command", "command": "echo hi", "timeout": 10});
    let (mapped, warnings) = map.map_keys(&hook, "command");
    assert_eq!(mapped, hook);
    assert!(warnings.is_empty());
}

#[test]
fn test_codex_structure_converter_detects_claude_code() {
    let conv = CodexStructureConverter;
    let value = json!({"hooks": {"SessionStart": []}});
    assert!(matches!(
        conv.detect_format(&value),
        SourceFormat::ClaudeCode
    ));
}

#[test]
fn test_codex_structure_converter_top_level_passthrough() {
    let conv = CodexStructureConverter;
    let value = json!({"hooks": {"SessionStart": []}});
    let (result, warnings) = conv.convert_top_level(&value);
    assert_eq!(result, value);
    assert!(warnings.is_empty());
}

#[test]
fn test_codex_script_generator_http_returns_error() {
    let gen = CodexScriptGenerator;
    let hook_value = json!({"type": "http", "url": "https://example.com"});
    let hook_obj = hook_value.as_object().unwrap();
    let http = HttpHook::new(hook_obj, &hook_value).unwrap();
    let result = gen.generate_http_script(&http, "sessionStart", None, 0);
    assert!(result.is_err());
}
