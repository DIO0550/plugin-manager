use crate::hooks::converter::ToolMap;
use crate::hooks::tool::claude_code::{to_target_tool, HookTool, ToolBridge};

/// Claude Code → Antigravity tool names for matcher remapping.
///
/// 出典: https://antigravity.google/docs/hooks 、docs/reference/hooks-schema-mapping.md §10.4
pub(crate) const ANTIGRAVITY_TOOL_ENTRIES: &[ToolBridge] = &[
    ToolBridge {
        claude_code_tools: &[HookTool::Bash],
        target_name: "run_command",
        representative_index: 0,
    },
    ToolBridge {
        claude_code_tools: &[HookTool::Read],
        target_name: "view_file",
        representative_index: 0,
    },
    ToolBridge {
        claude_code_tools: &[HookTool::Write],
        target_name: "write_to_file",
        representative_index: 0,
    },
    ToolBridge {
        claude_code_tools: &[HookTool::Edit],
        target_name: "replace_file_content",
        representative_index: 0,
    },
    ToolBridge {
        claude_code_tools: &[HookTool::MultiEdit],
        target_name: "multi_replace_file_content",
        representative_index: 0,
    },
    ToolBridge {
        claude_code_tools: &[HookTool::Glob],
        target_name: "find_by_name",
        representative_index: 0,
    },
    ToolBridge {
        claude_code_tools: &[HookTool::Grep],
        target_name: "grep_search",
        representative_index: 0,
    },
    ToolBridge {
        claude_code_tools: &[HookTool::WebFetch],
        target_name: "read_url_content",
        representative_index: 0,
    },
    ToolBridge {
        claude_code_tools: &[HookTool::Agent],
        target_name: "invoke_subagent",
        representative_index: 0,
    },
];

pub(crate) struct AntigravityToolMap;

impl ToolMap for AntigravityToolMap {
    fn map_tool(&self, tool: &str) -> String {
        let trimmed = tool.trim();
        let hook_tool = HookTool::from_str(trimmed);
        match to_target_tool(ANTIGRAVITY_TOOL_ENTRIES, &hook_tool) {
            Some(target) => target.to_string(),
            None => trimmed.to_string(),
        }
    }
}
