//! Tests for HookEvent enum, EventBridge, and helper functions.

use super::claude_code::*;

// ============================================================================
// HookEvent::from_str
// ============================================================================

#[test]
fn test_from_str_all_supported_events() {
    assert_eq!(HookEvent::from_str("SessionStart"), HookEvent::SessionStart);
    assert_eq!(HookEvent::from_str("SessionEnd"), HookEvent::SessionEnd);
    assert_eq!(HookEvent::from_str("PreToolUse"), HookEvent::PreToolUse);
    assert_eq!(HookEvent::from_str("PostToolUse"), HookEvent::PostToolUse);
    assert_eq!(
        HookEvent::from_str("UserPromptSubmit"),
        HookEvent::UserPromptSubmit
    );
    assert_eq!(HookEvent::from_str("Stop"), HookEvent::Stop);
    assert_eq!(HookEvent::from_str("SubagentStop"), HookEvent::SubagentStop);
}

#[test]
fn test_from_str_excluded_events_become_other() {
    let excluded = [
        "PostToolUseFailure",
        "PreCompact",
        "PostCompact",
        "PermissionRequest",
        "Notification",
        "SubagentStart",
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
        assert_eq!(
            HookEvent::from_str(event),
            HookEvent::Other((*event).to_string()),
            "Expected Other for excluded event: {}",
            event
        );
    }
}

#[test]
fn test_from_str_unknown_and_empty() {
    assert_eq!(
        HookEvent::from_str("SomeNewEvent"),
        HookEvent::Other("SomeNewEvent".into())
    );
    assert_eq!(HookEvent::from_str(""), HookEvent::Other("".into()));
}

// ============================================================================
// to_target_event
// ============================================================================

#[test]
fn test_to_target_event_found() {
    let table = &[
        EventBridge {
            event: HookEvent::SessionStart,
            target: "sessionStart",
        },
        EventBridge {
            event: HookEvent::PreToolUse,
            target: "preToolUse",
        },
        EventBridge {
            event: HookEvent::Stop,
            target: "agentStop",
        },
    ];
    assert_eq!(
        to_target_event(table, &HookEvent::SessionStart),
        Some("sessionStart")
    );
    assert_eq!(
        to_target_event(table, &HookEvent::PreToolUse),
        Some("preToolUse")
    );
    assert_eq!(to_target_event(table, &HookEvent::Stop), Some("agentStop"));
}

#[test]
fn test_to_target_event_not_found() {
    let table = &[EventBridge {
        event: HookEvent::SessionStart,
        target: "sessionStart",
    }];
    assert_eq!(to_target_event(table, &HookEvent::Stop), None);
}

#[test]
fn test_to_target_event_empty_table() {
    let table: &[EventBridge] = &[];
    assert_eq!(to_target_event(table, &HookEvent::SessionStart), None);
}

#[test]
fn test_to_target_event_other_returns_none() {
    let table = &[EventBridge {
        event: HookEvent::SessionStart,
        target: "sessionStart",
    }];
    assert_eq!(
        to_target_event(table, &HookEvent::Other("PostToolUseFailure".into())),
        None
    );
}

// ============================================================================
// to_source_event
// ============================================================================

#[test]
fn test_to_source_event_found() {
    let table = &[
        EventBridge {
            event: HookEvent::SessionStart,
            target: "sessionStart",
        },
        EventBridge {
            event: HookEvent::PreToolUse,
            target: "preToolUse",
        },
    ];
    assert_eq!(
        to_source_event(table, "sessionStart"),
        Some(HookEvent::SessionStart)
    );
    assert_eq!(
        to_source_event(table, "preToolUse"),
        Some(HookEvent::PreToolUse)
    );
}

#[test]
fn test_to_source_event_not_found() {
    let table = &[EventBridge {
        event: HookEvent::SessionStart,
        target: "sessionStart",
    }];
    assert_eq!(to_source_event(table, "agentStop"), None);
}

#[test]
fn test_to_source_event_empty_table() {
    let table: &[EventBridge] = &[];
    assert_eq!(to_source_event(table, "sessionStart"), None);
}
