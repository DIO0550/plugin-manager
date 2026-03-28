//! Tests for CodexEventMap (skeleton).

use super::codex::CodexEventMap;
use crate::hooks::converter::EventMap;

#[test]
fn test_codex_event_map_returns_none_for_all() {
    let map = CodexEventMap;
    assert_eq!(map.map_event("SessionStart"), None);
    assert_eq!(map.map_event("PreToolUse"), None);
    assert_eq!(map.map_event("PostToolUse"), None);
    assert_eq!(map.map_event("Stop"), None);
    assert_eq!(map.map_event("unknown"), None);
    assert_eq!(map.map_event(""), None);
}
