use crate::hooks::converter::ToolMap;
use crate::hooks::tool::claude_code::{to_target_tool, HookTool, ToolBridge};

/// Cursor agent tool names used in hook matchers (PascalCase).
///
/// Official matchers include `Shell`, `Read`, `Write`, `Grep`, `Delete`,
/// `Task`, and `MCP:<tool_name>`.
pub(crate) const CURSOR_TOOL_ENTRIES: &[ToolBridge] = &[
    ToolBridge {
        claude_code_tools: &[HookTool::Bash],
        target_name: "Shell",
        representative_index: 0,
    },
    ToolBridge {
        claude_code_tools: &[HookTool::Read],
        target_name: "Read",
        representative_index: 0,
    },
    ToolBridge {
        claude_code_tools: &[HookTool::Write],
        target_name: "Write",
        representative_index: 0,
    },
    ToolBridge {
        claude_code_tools: &[HookTool::Edit, HookTool::MultiEdit],
        target_name: "Write",
        representative_index: 0,
    },
    ToolBridge {
        claude_code_tools: &[HookTool::Grep],
        target_name: "Grep",
        representative_index: 0,
    },
    ToolBridge {
        claude_code_tools: &[HookTool::Agent],
        target_name: "Task",
        representative_index: 0,
    },
];

pub(crate) struct CursorToolMap;

impl ToolMap for CursorToolMap {
    fn map_tool(&self, tool: &str) -> String {
        let trimmed = tool.trim();
        let hook_tool = HookTool::from_str(trimmed);
        match to_target_tool(CURSOR_TOOL_ENTRIES, &hook_tool) {
            Some(target) => target.to_string(),
            None => trimmed.to_string(),
        }
    }
}
