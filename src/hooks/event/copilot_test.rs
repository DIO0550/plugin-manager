//! Tests for CopilotEventMap.

use super::copilot::CopilotEventMap;
use crate::hooks::converter::EventMap;

// ============================================================================
// Supported events (10)
// ============================================================================

#[test]
fn test_copilot_event_map_all_supported() {
    let map = CopilotEventMap;
    assert_eq!(map.map_event("SessionStart"), Some("sessionStart"));
    assert_eq!(map.map_event("SessionEnd"), Some("sessionEnd"));
    assert_eq!(map.map_event("PreToolUse"), Some("preToolUse"));
    assert_eq!(map.map_event("PostToolUse"), Some("postToolUse"));
    assert_eq!(
        map.map_event("PostToolUseFailure"),
        Some("postToolUseFailure")
    );
    assert_eq!(
        map.map_event("UserPromptSubmit"),
        Some("userPromptSubmitted")
    );
    assert_eq!(map.map_event("Stop"), Some("agentStop"));
    assert_eq!(map.map_event("SubagentStart"), Some("subagentStart"));
    assert_eq!(map.map_event("SubagentStop"), Some("subagentStop"));
    assert_eq!(map.map_event("PreCompact"), Some("preCompact"));
}

// ============================================================================
// Excluded events (11)
// ============================================================================

#[test]
fn test_copilot_event_map_excluded() {
    let map = CopilotEventMap;
    let excluded = [
        "PostCompact",
        "PermissionRequest",
        "Notification",
        "TeammateIdle",
        "TaskCompleted",
        "InstructionsLoaded",
        "ConfigChange",
        "WorktreeCreate",
        "WorktreeRemove",
        "Elicitation",
        "ElicitationResult",
    ];
    for event in &excluded {
        assert_eq!(map.map_event(event), None, "Expected None for: {}", event);
    }
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn test_copilot_event_map_unknown_and_empty() {
    let map = CopilotEventMap;
    assert_eq!(map.map_event("SomeNewEvent"), None);
    assert_eq!(map.map_event(""), None);
}

#[test]
fn test_copilot_event_map_with_whitespace() {
    let map = CopilotEventMap;
    assert_eq!(map.map_event(" SessionStart "), Some("sessionStart"));
    assert_eq!(map.map_event("  Stop  "), Some("agentStop"));
}
