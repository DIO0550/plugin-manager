//! Tests for CopilotToolMap.

use super::copilot::CopilotToolMap;
use crate::hooks::converter::ToolMap;
use std::collections::HashMap;

// ============================================================================
// Forward: Claude Code -> Copilot (all known tools)
// ============================================================================

#[test]
fn test_copilot_tool_map_forward_all_known() {
    let map = CopilotToolMap;
    assert_eq!(map.map_tool("Bash"), "bash");
    assert_eq!(map.map_tool("Read"), "view");
    assert_eq!(map.map_tool("Write"), "create");
    assert_eq!(map.map_tool("Edit"), "edit");
    assert_eq!(map.map_tool("MultiEdit"), "edit");
    assert_eq!(map.map_tool("Glob"), "glob");
    assert_eq!(map.map_tool("Grep"), "grep");
    assert_eq!(map.map_tool("WebFetch"), "web_fetch");
    assert_eq!(map.map_tool("Agent"), "task");
}

// ============================================================================
// Forward: N:1 -- Edit and MultiEdit both map to "edit"
// ============================================================================

#[test]
fn test_copilot_tool_map_n_to_1() {
    let map = CopilotToolMap;
    assert_eq!(map.map_tool("Edit"), map.map_tool("MultiEdit"));
}

// ============================================================================
// Forward: unknown tools pass through
// ============================================================================

#[test]
fn test_copilot_tool_map_forward_passthrough() {
    let map = CopilotToolMap;
    assert_eq!(map.map_tool("mcp__server__tool"), "mcp__server__tool");
    assert_eq!(map.map_tool("WebSearch"), "WebSearch");
    assert_eq!(map.map_tool("UnknownTool"), "UnknownTool");
}

// ============================================================================
// Forward: whitespace trimming
// ============================================================================

#[test]
fn test_copilot_tool_map_forward_whitespace() {
    let map = CopilotToolMap;
    assert_eq!(map.map_tool(" Bash "), "bash");
    assert_eq!(map.map_tool("  Read  "), "view");
}

// ============================================================================
// Forward: empty string
// ============================================================================

#[test]
fn test_copilot_tool_map_forward_empty() {
    let map = CopilotToolMap;
    assert_eq!(map.map_tool(""), "");
}

// ============================================================================
// Reverse: Copilot -> Claude Code
// ============================================================================

#[test]
fn test_copilot_tool_map_reverse_known() {
    let map = CopilotToolMap;
    assert_eq!(map.reverse_map_tool("bash"), "Bash");
    assert_eq!(map.reverse_map_tool("view"), "Read");
    assert_eq!(map.reverse_map_tool("create"), "Write");
    assert_eq!(map.reverse_map_tool("edit"), "Edit"); // representative
    assert_eq!(map.reverse_map_tool("glob"), "Glob");
    assert_eq!(map.reverse_map_tool("grep"), "Grep");
    assert_eq!(map.reverse_map_tool("web_fetch"), "WebFetch");
    assert_eq!(map.reverse_map_tool("task"), "Agent");
}

#[test]
fn test_copilot_tool_map_reverse_powershell() {
    let map = CopilotToolMap;
    assert_eq!(map.reverse_map_tool("powershell"), "Bash");
}

#[test]
fn test_copilot_tool_map_reverse_passthrough() {
    let map = CopilotToolMap;
    assert_eq!(map.reverse_map_tool("ask_user"), "ask_user");
    assert_eq!(map.reverse_map_tool("memory"), "memory");
    assert_eq!(map.reverse_map_tool("unknown_tool"), "unknown_tool");
}

#[test]
fn test_copilot_tool_map_reverse_empty() {
    let map = CopilotToolMap;
    assert_eq!(map.reverse_map_tool(""), "");
}

// ============================================================================
// Round-trip: Claude Code -> Copilot -> Claude Code
// ============================================================================

#[test]
fn test_copilot_tool_map_round_trip() {
    let map = CopilotToolMap;
    let tools = [
        "Bash", "Read", "Write", "Edit", "Glob", "Grep", "WebFetch", "Agent",
    ];
    for tool in &tools {
        let copilot = map.map_tool(tool);
        let back = map.reverse_map_tool(&copilot);
        assert_eq!(*tool, back, "Round-trip failed for {}", tool);
    }
}

// ============================================================================
// Tool mapping cross-check: verify that CopilotToolMap.reverse_map_tool()
// agrees with the expected jq mapping from copilot.rs build_env_bridge().
//
// NOTE: This test uses a manually maintained mirror of the jq object
// literal, NOT a parsed extraction from the source. If build_env_bridge()
// is updated, this mirror must be updated in tandem. The test guards
// against Rust-side drift but cannot detect jq-side-only changes.
// ============================================================================

#[test]
fn test_reverse_map_tool_matches_expected_jq_mapping() {
    // Manually maintained mirror of the jq object literal in
    // copilot.rs build_env_bridge(). Keep in sync when editing either side.
    let expected_jq_mapping: HashMap<&str, &str> = HashMap::from([
        ("bash", "Bash"),
        ("powershell", "Bash"),
        ("view", "Read"),
        ("create", "Write"),
        ("edit", "Edit"),
        ("glob", "Glob"),
        ("grep", "Grep"),
        ("web_fetch", "WebFetch"),
        ("task", "Agent"),
    ]);

    let map = CopilotToolMap;

    // Every expected jq entry must match reverse_map_tool
    for (copilot_name, expected_cc_name) in &expected_jq_mapping {
        let actual = map.reverse_map_tool(copilot_name);
        assert_eq!(
            actual, *expected_cc_name,
            "jq mapping '{} -> {}' disagrees with reverse_map_tool (got '{}')",
            copilot_name, expected_cc_name, actual
        );
    }

    // Every Rust forward entry must have a counterpart in the expected mapping
    use super::copilot::COPILOT_TOOL_ENTRIES;
    for entry in COPILOT_TOOL_ENTRIES {
        assert!(
            expected_jq_mapping.contains_key(entry.target_name),
            "Rust table has target '{}' but expected jq mapping does not",
            entry.target_name
        );
    }
}
