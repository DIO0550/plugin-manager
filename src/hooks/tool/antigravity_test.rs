//! Unit tests for Antigravity ToolMap.

use super::antigravity::AntigravityToolMap;
use crate::hooks::converter::ToolMap;

#[test]
fn test_antigravity_tool_map_known_tools() {
    let map = AntigravityToolMap;
    assert_eq!(map.map_tool("Bash"), "run_command");
    assert_eq!(map.map_tool("Read"), "view_file");
    assert_eq!(map.map_tool("Write"), "write_to_file");
    assert_eq!(map.map_tool("Edit"), "replace_file_content");
    assert_eq!(map.map_tool("MultiEdit"), "multi_replace_file_content");
    assert_eq!(map.map_tool("Glob"), "find_by_name");
    assert_eq!(map.map_tool("Grep"), "grep_search");
    assert_eq!(map.map_tool("WebFetch"), "read_url_content");
    assert_eq!(map.map_tool("Agent"), "invoke_subagent");
}

#[test]
fn test_antigravity_tool_map_unknown_passthrough() {
    let map = AntigravityToolMap;
    assert_eq!(map.map_tool("run_command"), "run_command");
    assert_eq!(map.map_tool("CustomTool"), "CustomTool");
}
