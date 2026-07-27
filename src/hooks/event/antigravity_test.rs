//! Unit tests for Antigravity EventMap.

use super::antigravity::AntigravityEventMap;
use crate::hooks::converter::EventMap;

#[test]
fn test_antigravity_event_map_direct_events() {
    let map = AntigravityEventMap;
    assert_eq!(map.map_event("PreToolUse"), Some("PreToolUse"));
    assert_eq!(map.map_event("PostToolUse"), Some("PostToolUse"));
    assert_eq!(map.map_event("Stop"), Some("Stop"));
}

#[test]
fn test_antigravity_event_map_approximate_events() {
    let map = AntigravityEventMap;
    assert_eq!(map.map_event("SessionStart"), Some("PreInvocation"));
    assert_eq!(map.map_event("UserPromptSubmit"), Some("PreInvocation"));
    assert_eq!(map.map_event("SessionEnd"), Some("Stop"));
}

#[test]
fn test_antigravity_event_map_unsupported_events() {
    let map = AntigravityEventMap;
    assert_eq!(map.map_event("SubagentStop"), None);
    assert_eq!(map.map_event("SubagentStart"), None);
    assert_eq!(map.map_event("PreCompact"), None);
    assert_eq!(map.map_event("Notification"), None);
    assert_eq!(map.map_event("PermissionRequest"), None);
    assert_eq!(map.map_event("UnknownEvent"), None);
}

#[test]
fn test_antigravity_event_map_preserve_matcher_groups() {
    let map = AntigravityEventMap;
    assert!(map.preserve_matcher_groups("PreToolUse"));
    assert!(map.preserve_matcher_groups("PostToolUse"));
    assert!(!map.preserve_matcher_groups("PreInvocation"));
    assert!(!map.preserve_matcher_groups("PostInvocation"));
    assert!(!map.preserve_matcher_groups("Stop"));
}
