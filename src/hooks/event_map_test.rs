//! Tests for hooks event_map module.

use super::event_map::*;

// ============================================================================
// Event name mapping: Claude Code -> Copilot CLI
// ============================================================================

#[test]
fn test_event_claude_to_copilot_supported_events() {
    assert_eq!(
        event_claude_to_copilot("SessionStart"),
        Some("sessionStart")
    );
    assert_eq!(event_claude_to_copilot("SessionEnd"), Some("sessionEnd"));
    assert_eq!(event_claude_to_copilot("PreToolUse"), Some("preToolUse"));
    assert_eq!(event_claude_to_copilot("PostToolUse"), Some("postToolUse"));
    assert_eq!(
        event_claude_to_copilot("UserPromptSubmit"),
        Some("userPromptSubmitted")
    );
    assert_eq!(event_claude_to_copilot("Stop"), Some("agentStop"));
    assert_eq!(
        event_claude_to_copilot("SubagentStop"),
        Some("subagentStop")
    );
}

#[test]
fn test_event_claude_to_copilot_excluded_events() {
    assert_eq!(event_claude_to_copilot("PostToolUseFailure"), None);
    assert_eq!(event_claude_to_copilot("PreCompact"), None);
    assert_eq!(event_claude_to_copilot("PostCompact"), None);
    assert_eq!(event_claude_to_copilot("PermissionRequest"), None);
    assert_eq!(event_claude_to_copilot("Notification"), None);
    assert_eq!(event_claude_to_copilot("SubagentStart"), None);
    assert_eq!(event_claude_to_copilot("TeammateIdle"), None);
    assert_eq!(event_claude_to_copilot("TaskCompleted"), None);
    assert_eq!(event_claude_to_copilot("InstructionsLoaded"), None);
    assert_eq!(event_claude_to_copilot("ConfigChange"), None);
    assert_eq!(event_claude_to_copilot("WorktreeCreate"), None);
    assert_eq!(event_claude_to_copilot("WorktreeRemove"), None);
    assert_eq!(event_claude_to_copilot("Elicitation"), None);
    assert_eq!(event_claude_to_copilot("ElicitationResult"), None);
}

#[test]
fn test_event_claude_to_copilot_unknown() {
    assert_eq!(event_claude_to_copilot("SomeNewEvent"), None);
    assert_eq!(event_claude_to_copilot(""), None);
}

// ============================================================================
// Event name mapping: Copilot CLI -> Claude Code
// ============================================================================

#[test]
fn test_event_copilot_to_claude_supported_events() {
    assert_eq!(
        event_copilot_to_claude("sessionStart"),
        Some("SessionStart")
    );
    assert_eq!(event_copilot_to_claude("sessionEnd"), Some("SessionEnd"));
    assert_eq!(event_copilot_to_claude("preToolUse"), Some("PreToolUse"));
    assert_eq!(event_copilot_to_claude("postToolUse"), Some("PostToolUse"));
    assert_eq!(
        event_copilot_to_claude("userPromptSubmitted"),
        Some("UserPromptSubmit")
    );
    assert_eq!(event_copilot_to_claude("agentStop"), Some("Stop"));
    assert_eq!(
        event_copilot_to_claude("subagentStop"),
        Some("SubagentStop")
    );
}

#[test]
fn test_event_copilot_to_claude_unknown() {
    assert_eq!(event_copilot_to_claude("errorOccurred"), None);
    assert_eq!(event_copilot_to_claude("someNewEvent"), None);
    assert_eq!(event_copilot_to_claude(""), None);
}

// ============================================================================
// Event name round-trip
// ============================================================================

#[test]
fn test_event_round_trip_claude_copilot_claude() {
    let events = [
        "SessionStart",
        "SessionEnd",
        "PreToolUse",
        "PostToolUse",
        "UserPromptSubmit",
        "Stop",
        "SubagentStop",
    ];
    for event in &events {
        let copilot = event_claude_to_copilot(event).unwrap();
        let back = event_copilot_to_claude(copilot).unwrap();
        assert_eq!(*event, back, "Round-trip failed for {}", event);
    }
}

// ============================================================================
// Tool name mapping: Copilot CLI -> Claude Code (Hooks context)
// ============================================================================

#[test]
fn test_tool_copilot_to_claude_known_tools() {
    assert_eq!(tool_copilot_to_claude("bash"), "Bash");
    assert_eq!(tool_copilot_to_claude("view"), "Read");
    assert_eq!(tool_copilot_to_claude("create"), "Write");
    assert_eq!(tool_copilot_to_claude("edit"), "Edit");
    assert_eq!(tool_copilot_to_claude("glob"), "Glob");
    assert_eq!(tool_copilot_to_claude("grep"), "Grep");
    assert_eq!(tool_copilot_to_claude("web_fetch"), "WebFetch");
    assert_eq!(tool_copilot_to_claude("task"), "Agent");
    assert_eq!(tool_copilot_to_claude("powershell"), "Bash");
}

#[test]
fn test_tool_copilot_to_claude_passthrough() {
    assert_eq!(tool_copilot_to_claude("ask_user"), "ask_user");
    assert_eq!(tool_copilot_to_claude("memory"), "memory");
    assert_eq!(tool_copilot_to_claude("unknown_tool"), "unknown_tool");
}

// ============================================================================
// Tool name mapping: Claude Code -> Copilot CLI (Hooks context)
// ============================================================================

#[test]
fn test_tool_claude_to_copilot_known_tools() {
    assert_eq!(tool_claude_to_copilot("Bash"), "bash");
    assert_eq!(tool_claude_to_copilot("Read"), "view");
    assert_eq!(tool_claude_to_copilot("Write"), "create");
    assert_eq!(tool_claude_to_copilot("Edit"), "edit");
    assert_eq!(tool_claude_to_copilot("Glob"), "glob");
    assert_eq!(tool_claude_to_copilot("Grep"), "grep");
    assert_eq!(tool_claude_to_copilot("WebFetch"), "web_fetch");
    assert_eq!(tool_claude_to_copilot("Agent"), "task");
}

#[test]
fn test_tool_claude_to_copilot_passthrough() {
    assert_eq!(tool_claude_to_copilot("WebSearch"), "WebSearch");
    assert_eq!(
        tool_claude_to_copilot("mcp__server__tool"),
        "mcp__server__tool"
    );
    assert_eq!(tool_claude_to_copilot("UnknownTool"), "UnknownTool");
}

// ============================================================================
// Tool name edge cases
// ============================================================================

#[test]
fn test_tool_copilot_to_claude_with_whitespace() {
    assert_eq!(tool_copilot_to_claude(" bash "), "Bash");
    assert_eq!(tool_copilot_to_claude("  view  "), "Read");
}

#[test]
fn test_tool_claude_to_copilot_with_whitespace() {
    assert_eq!(tool_claude_to_copilot(" Bash "), "bash");
    assert_eq!(tool_claude_to_copilot("  Read  "), "view");
}

#[test]
fn test_tool_empty_string() {
    assert_eq!(tool_copilot_to_claude(""), "");
    assert_eq!(tool_claude_to_copilot(""), "");
}

// ============================================================================
// Tool name round-trip (Claude -> Copilot -> Claude)
// ============================================================================

#[test]
fn test_tool_round_trip_claude_copilot_claude() {
    let tools = [
        "Bash", "Read", "Write", "Edit", "Glob", "Grep", "WebFetch", "Agent",
    ];
    for tool in &tools {
        let copilot = tool_claude_to_copilot(tool);
        let back = tool_copilot_to_claude(&copilot);
        assert_eq!(*tool, back, "Round-trip failed for {}", tool);
    }
}
