use crate::hooks::converter::ToolMap;

/// Codex skeleton: passes through all tool names (trimmed).
pub(crate) struct CodexToolMap;

impl ToolMap for CodexToolMap {
    fn map_tool(&self, tool: &str) -> String {
        tool.trim().to_string()
    }
}
