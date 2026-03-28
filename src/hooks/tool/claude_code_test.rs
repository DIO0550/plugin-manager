//! Tests for HookTool enum, ToolBridge, and helper functions.

use super::claude_code::*;

// ============================================================================
// HookTool::from_str
// ============================================================================

#[test]
fn test_from_str_all_known_tools() {
    assert_eq!(HookTool::from_str("Bash"), HookTool::Bash);
    assert_eq!(HookTool::from_str("Read"), HookTool::Read);
    assert_eq!(HookTool::from_str("Write"), HookTool::Write);
    assert_eq!(HookTool::from_str("Edit"), HookTool::Edit);
    assert_eq!(HookTool::from_str("MultiEdit"), HookTool::MultiEdit);
    assert_eq!(HookTool::from_str("Glob"), HookTool::Glob);
    assert_eq!(HookTool::from_str("Grep"), HookTool::Grep);
    assert_eq!(HookTool::from_str("WebFetch"), HookTool::WebFetch);
    assert_eq!(HookTool::from_str("Agent"), HookTool::Agent);
}

#[test]
fn test_from_str_unknown_tools_become_other() {
    assert_eq!(
        HookTool::from_str("mcp__server__tool"),
        HookTool::Other("mcp__server__tool".into())
    );
    assert_eq!(HookTool::from_str(""), HookTool::Other("".into()));
}

// ============================================================================
// to_target_tool
// ============================================================================

#[test]
fn test_to_target_tool_1_to_1() {
    let table = &[ToolBridge {
        claude_code_tools: &[HookTool::Bash],
        target_name: "bash",
        representative_index: 0,
    }];
    assert_eq!(to_target_tool(table, &HookTool::Bash), Some("bash"));
}

#[test]
fn test_to_target_tool_n_to_1() {
    let table = &[ToolBridge {
        claude_code_tools: &[HookTool::Edit, HookTool::MultiEdit],
        target_name: "edit",
        representative_index: 0,
    }];
    assert_eq!(to_target_tool(table, &HookTool::Edit), Some("edit"));
    assert_eq!(to_target_tool(table, &HookTool::MultiEdit), Some("edit"));
}

#[test]
fn test_to_target_tool_other_returns_none() {
    let table = &[ToolBridge {
        claude_code_tools: &[HookTool::Bash],
        target_name: "bash",
        representative_index: 0,
    }];
    assert_eq!(
        to_target_tool(table, &HookTool::Other("mcp__tool".into())),
        None
    );
}

#[test]
fn test_to_target_tool_empty_table() {
    let table: &[ToolBridge] = &[];
    assert_eq!(to_target_tool(table, &HookTool::Bash), None);
}

// ============================================================================
// to_source_tool
// ============================================================================

#[test]
fn test_to_source_tool_representative() {
    let table = &[ToolBridge {
        claude_code_tools: &[HookTool::Edit, HookTool::MultiEdit],
        target_name: "edit",
        representative_index: 0,
    }];
    assert_eq!(to_source_tool(table, "edit"), Some(&HookTool::Edit));
}

#[test]
fn test_to_source_tool_representative_index_1() {
    let table = &[ToolBridge {
        claude_code_tools: &[HookTool::Edit, HookTool::MultiEdit],
        target_name: "edit",
        representative_index: 1,
    }];
    assert_eq!(to_source_tool(table, "edit"), Some(&HookTool::MultiEdit));
}

#[test]
fn test_to_source_tool_not_found() {
    let table = &[ToolBridge {
        claude_code_tools: &[HookTool::Bash],
        target_name: "bash",
        representative_index: 0,
    }];
    assert_eq!(to_source_tool(table, "unknown"), None);
}

#[test]
fn test_to_source_tool_empty_table() {
    let table: &[ToolBridge] = &[];
    assert_eq!(to_source_tool(table, "bash"), None);
}

// ============================================================================
// representative_index validation
// ============================================================================

/// Validate that every entry in a tool table has a valid representative_index.
fn validate_tool_entries(table: &[ToolBridge]) {
    for (i, entry) in table.iter().enumerate() {
        assert!(
            !entry.claude_code_tools.is_empty(),
            "Entry {} ('{}') has empty claude_code_tools",
            i,
            entry.target_name
        );
        assert!(
            entry.representative_index < entry.claude_code_tools.len(),
            "Entry {} ('{}') has representative_index {} but only {} tools",
            i,
            entry.target_name,
            entry.representative_index,
            entry.claude_code_tools.len()
        );
    }
}

#[test]
fn test_validate_copilot_tool_entries() {
    use super::copilot::COPILOT_TOOL_ENTRIES;
    validate_tool_entries(COPILOT_TOOL_ENTRIES);
}

#[test]
#[should_panic(expected = "representative_index")]
fn test_validate_catches_invalid_index() {
    let bad_table = &[ToolBridge {
        claude_code_tools: &[HookTool::Bash],
        target_name: "bash",
        representative_index: 5, // out of bounds
    }];
    validate_tool_entries(bad_table);
}

#[test]
#[should_panic(expected = "empty claude_code_tools")]
fn test_validate_catches_empty_tools() {
    let bad_table = &[ToolBridge {
        claude_code_tools: &[],
        target_name: "empty",
        representative_index: 0,
    }];
    validate_tool_entries(bad_table);
}
