//! Tests for CodexEventMap.

use super::codex::CodexEventMap;
use crate::hooks::converter::EventMap;

#[test]
fn test_codex_event_map_keeps_supported_events() {
    let map = CodexEventMap;
    assert_eq!(map.map_event("SessionStart"), Some("SessionStart"));
    assert_eq!(map.map_event("PreToolUse"), Some("PreToolUse"));
    assert_eq!(map.map_event("PostToolUse"), Some("PostToolUse"));
    assert_eq!(map.map_event("UserPromptSubmit"), Some("UserPromptSubmit"));
    assert_eq!(map.map_event("Stop"), Some("Stop"));
    assert_eq!(
        map.map_event("PermissionRequest"),
        Some("PermissionRequest")
    );
    assert_eq!(map.map_event("PreCompact"), Some("PreCompact"));
    assert_eq!(map.map_event("PostCompact"), Some("PostCompact"));
    assert_eq!(map.map_event("SubagentStart"), Some("SubagentStart"));
    assert_eq!(map.map_event("SubagentStop"), Some("SubagentStop"));
}

#[test]
fn test_codex_event_map_rejects_unknown_events() {
    let map = CodexEventMap;
    assert_eq!(map.map_event("unknown"), None);
    assert_eq!(map.map_event(""), None);
    assert_eq!(map.map_event("NotAnEvent"), None);
}
