//! Tests for CursorToolMap.

use super::cursor::CursorToolMap;
use crate::hooks::converter::ToolMap;

#[test]
fn test_cursor_tool_map_known_tools() {
    let map = CursorToolMap;
    assert_eq!(map.map_tool("Bash"), "Shell");
    assert_eq!(map.map_tool("Read"), "Read");
    assert_eq!(map.map_tool("Write"), "Write");
    assert_eq!(map.map_tool("Edit"), "Write");
    assert_eq!(map.map_tool("MultiEdit"), "Write");
    assert_eq!(map.map_tool("Grep"), "Grep");
    assert_eq!(map.map_tool("Agent"), "Task");
}

#[test]
fn test_cursor_tool_map_passthrough_unknown() {
    let map = CursorToolMap;
    assert_eq!(map.map_tool("Glob"), "Glob");
    assert_eq!(map.map_tool("WebFetch"), "WebFetch");
    assert_eq!(map.map_tool("mcp__server__tool"), "mcp__server__tool");
}
