//! Tests for CodexToolMap (skeleton).

use super::codex::CodexToolMap;
use crate::hooks::converter::ToolMap;

#[test]
fn test_codex_tool_map_passthrough() {
    let map = CodexToolMap;
    assert_eq!(map.map_tool("Bash"), "Bash");
    assert_eq!(map.map_tool("Read"), "Read");
    assert_eq!(map.map_tool("unknown"), "unknown");
    assert_eq!(map.map_tool(""), "");
}

#[test]
fn test_codex_tool_map_trims_whitespace() {
    let map = CodexToolMap;
    assert_eq!(map.map_tool(" Bash "), "Bash");
}
