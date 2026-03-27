//! Tests for hooks event_map module.

use super::event_map::*;
use crate::format::Format;

// ============================================================================
// Event name mapping: Claude Code -> Copilot CLI
// ============================================================================

#[test]
fn test_map_event_claude_to_copilot_supported_events() {
    assert_eq!(
        map_event("SessionStart", Format::ClaudeCode, Format::Copilot),
        Some("sessionStart")
    );
    assert_eq!(
        map_event("SessionEnd", Format::ClaudeCode, Format::Copilot),
        Some("sessionEnd")
    );
    assert_eq!(
        map_event("PreToolUse", Format::ClaudeCode, Format::Copilot),
        Some("preToolUse")
    );
    assert_eq!(
        map_event("PostToolUse", Format::ClaudeCode, Format::Copilot),
        Some("postToolUse")
    );
    assert_eq!(
        map_event("UserPromptSubmit", Format::ClaudeCode, Format::Copilot),
        Some("userPromptSubmitted")
    );
    assert_eq!(
        map_event("Stop", Format::ClaudeCode, Format::Copilot),
        Some("agentStop")
    );
    assert_eq!(
        map_event("SubagentStop", Format::ClaudeCode, Format::Copilot),
        Some("subagentStop")
    );
}

#[test]
fn test_map_event_claude_to_copilot_excluded_events() {
    assert_eq!(
        map_event("PostToolUseFailure", Format::ClaudeCode, Format::Copilot),
        None
    );
    assert_eq!(
        map_event("PreCompact", Format::ClaudeCode, Format::Copilot),
        None
    );
    assert_eq!(
        map_event("PostCompact", Format::ClaudeCode, Format::Copilot),
        None
    );
    assert_eq!(
        map_event("PermissionRequest", Format::ClaudeCode, Format::Copilot),
        None
    );
    assert_eq!(
        map_event("Notification", Format::ClaudeCode, Format::Copilot),
        None
    );
    assert_eq!(
        map_event("SubagentStart", Format::ClaudeCode, Format::Copilot),
        None
    );
    assert_eq!(
        map_event("TeammateIdle", Format::ClaudeCode, Format::Copilot),
        None
    );
    assert_eq!(
        map_event("TaskCompleted", Format::ClaudeCode, Format::Copilot),
        None
    );
    assert_eq!(
        map_event("InstructionsLoaded", Format::ClaudeCode, Format::Copilot),
        None
    );
    assert_eq!(
        map_event("ConfigChange", Format::ClaudeCode, Format::Copilot),
        None
    );
    assert_eq!(
        map_event("WorktreeCreate", Format::ClaudeCode, Format::Copilot),
        None
    );
    assert_eq!(
        map_event("WorktreeRemove", Format::ClaudeCode, Format::Copilot),
        None
    );
    assert_eq!(
        map_event("Elicitation", Format::ClaudeCode, Format::Copilot),
        None
    );
    assert_eq!(
        map_event("ElicitationResult", Format::ClaudeCode, Format::Copilot),
        None
    );
}

#[test]
fn test_map_event_claude_to_copilot_unknown() {
    assert_eq!(
        map_event("SomeNewEvent", Format::ClaudeCode, Format::Copilot),
        None
    );
    assert_eq!(map_event("", Format::ClaudeCode, Format::Copilot), None);
}

// ============================================================================
// Event name mapping: Copilot CLI -> Claude Code
// ============================================================================

#[test]
fn test_map_event_copilot_to_claude_supported_events() {
    assert_eq!(
        map_event("sessionStart", Format::Copilot, Format::ClaudeCode),
        Some("SessionStart")
    );
    assert_eq!(
        map_event("sessionEnd", Format::Copilot, Format::ClaudeCode),
        Some("SessionEnd")
    );
    assert_eq!(
        map_event("preToolUse", Format::Copilot, Format::ClaudeCode),
        Some("PreToolUse")
    );
    assert_eq!(
        map_event("postToolUse", Format::Copilot, Format::ClaudeCode),
        Some("PostToolUse")
    );
    assert_eq!(
        map_event("userPromptSubmitted", Format::Copilot, Format::ClaudeCode),
        Some("UserPromptSubmit")
    );
    assert_eq!(
        map_event("agentStop", Format::Copilot, Format::ClaudeCode),
        Some("Stop")
    );
    assert_eq!(
        map_event("subagentStop", Format::Copilot, Format::ClaudeCode),
        Some("SubagentStop")
    );
}

#[test]
fn test_map_event_copilot_to_claude_unknown() {
    assert_eq!(
        map_event("errorOccurred", Format::Copilot, Format::ClaudeCode),
        None
    );
    assert_eq!(
        map_event("someNewEvent", Format::Copilot, Format::ClaudeCode),
        None
    );
    assert_eq!(map_event("", Format::Copilot, Format::ClaudeCode), None);
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
        let copilot = map_event(event, Format::ClaudeCode, Format::Copilot).unwrap();
        let back = map_event(copilot, Format::Copilot, Format::ClaudeCode).unwrap();
        assert_eq!(*event, back, "Round-trip failed for {}", event);
    }
}

// ============================================================================
// Tool name mapping: Copilot CLI -> Claude Code (Hooks context)
// ============================================================================

#[test]
fn test_map_tool_copilot_to_claude_known_tools() {
    assert_eq!(
        map_tool("bash", Format::Copilot, Format::ClaudeCode),
        "Bash"
    );
    assert_eq!(
        map_tool("view", Format::Copilot, Format::ClaudeCode),
        "Read"
    );
    assert_eq!(
        map_tool("create", Format::Copilot, Format::ClaudeCode),
        "Write"
    );
    assert_eq!(
        map_tool("edit", Format::Copilot, Format::ClaudeCode),
        "Edit"
    );
    assert_eq!(
        map_tool("glob", Format::Copilot, Format::ClaudeCode),
        "Glob"
    );
    assert_eq!(
        map_tool("grep", Format::Copilot, Format::ClaudeCode),
        "Grep"
    );
    assert_eq!(
        map_tool("web_fetch", Format::Copilot, Format::ClaudeCode),
        "WebFetch"
    );
    assert_eq!(
        map_tool("task", Format::Copilot, Format::ClaudeCode),
        "Agent"
    );
}

#[test]
fn test_map_tool_copilot_to_claude_passthrough() {
    assert_eq!(
        map_tool("ask_user", Format::Copilot, Format::ClaudeCode),
        "ask_user"
    );
    assert_eq!(
        map_tool("memory", Format::Copilot, Format::ClaudeCode),
        "memory"
    );
    assert_eq!(
        map_tool("powershell", Format::Copilot, Format::ClaudeCode),
        "powershell"
    );
    assert_eq!(
        map_tool("unknown_tool", Format::Copilot, Format::ClaudeCode),
        "unknown_tool"
    );
}

// ============================================================================
// Tool name mapping: Claude Code -> Copilot CLI (Hooks context)
// ============================================================================

#[test]
fn test_map_tool_claude_to_copilot_known_tools() {
    assert_eq!(
        map_tool("Bash", Format::ClaudeCode, Format::Copilot),
        "bash"
    );
    assert_eq!(
        map_tool("Read", Format::ClaudeCode, Format::Copilot),
        "view"
    );
    assert_eq!(
        map_tool("Write", Format::ClaudeCode, Format::Copilot),
        "create"
    );
    assert_eq!(
        map_tool("Edit", Format::ClaudeCode, Format::Copilot),
        "edit"
    );
    assert_eq!(
        map_tool("MultiEdit", Format::ClaudeCode, Format::Copilot),
        "edit"
    );
    assert_eq!(
        map_tool("Glob", Format::ClaudeCode, Format::Copilot),
        "glob"
    );
    assert_eq!(
        map_tool("Grep", Format::ClaudeCode, Format::Copilot),
        "grep"
    );
    assert_eq!(
        map_tool("WebFetch", Format::ClaudeCode, Format::Copilot),
        "web_fetch"
    );
    assert_eq!(
        map_tool("Agent", Format::ClaudeCode, Format::Copilot),
        "task"
    );
}

#[test]
fn test_map_tool_claude_to_copilot_passthrough() {
    assert_eq!(
        map_tool("WebSearch", Format::ClaudeCode, Format::Copilot),
        "WebSearch"
    );
    assert_eq!(
        map_tool("mcp__server__tool", Format::ClaudeCode, Format::Copilot),
        "mcp__server__tool"
    );
    assert_eq!(
        map_tool("UnknownTool", Format::ClaudeCode, Format::Copilot),
        "UnknownTool"
    );
}

// ============================================================================
// Tool name edge cases
// ============================================================================

#[test]
fn test_map_tool_copilot_to_claude_with_whitespace() {
    assert_eq!(
        map_tool(" bash ", Format::Copilot, Format::ClaudeCode),
        "Bash"
    );
    assert_eq!(
        map_tool("  view  ", Format::Copilot, Format::ClaudeCode),
        "Read"
    );
}

#[test]
fn test_map_tool_claude_to_copilot_with_whitespace() {
    assert_eq!(
        map_tool(" Bash ", Format::ClaudeCode, Format::Copilot),
        "bash"
    );
    assert_eq!(
        map_tool("  Read  ", Format::ClaudeCode, Format::Copilot),
        "view"
    );
}

#[test]
fn test_map_tool_empty_string() {
    assert_eq!(map_tool("", Format::Copilot, Format::ClaudeCode), "");
    assert_eq!(map_tool("", Format::ClaudeCode, Format::Copilot), "");
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
        let copilot = map_tool(tool, Format::ClaudeCode, Format::Copilot);
        let back = map_tool(&copilot, Format::Copilot, Format::ClaudeCode);
        assert_eq!(*tool, back, "Round-trip failed for {}", tool);
    }
}

// ============================================================================
// N:1 representative value tests
// ============================================================================

#[test]
fn test_map_tool_n_to_1_representative() {
    // "edit" reverse lookup returns "Edit" (first table entry), not "MultiEdit"
    assert_eq!(
        map_tool("edit", Format::Copilot, Format::ClaudeCode),
        "Edit"
    );
}
