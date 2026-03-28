use crate::hooks::converter::ToolMap;
use crate::hooks::tool::claude_code::{to_source_tool, to_target_tool, HookTool, ToolBridge};

pub(crate) const COPILOT_TOOL_ENTRIES: &[ToolBridge] = &[
    ToolBridge {
        claude_code_tools: &[HookTool::Bash],
        target_name: "bash",
        representative_index: 0,
    },
    ToolBridge {
        claude_code_tools: &[HookTool::Read],
        target_name: "view",
        representative_index: 0,
    },
    ToolBridge {
        claude_code_tools: &[HookTool::Write],
        target_name: "create",
        representative_index: 0,
    },
    ToolBridge {
        claude_code_tools: &[HookTool::Edit, HookTool::MultiEdit],
        target_name: "edit",
        representative_index: 0,
    },
    ToolBridge {
        claude_code_tools: &[HookTool::Glob],
        target_name: "glob",
        representative_index: 0,
    },
    ToolBridge {
        claude_code_tools: &[HookTool::Grep],
        target_name: "grep",
        representative_index: 0,
    },
    ToolBridge {
        claude_code_tools: &[HookTool::WebFetch],
        target_name: "web_fetch",
        representative_index: 0,
    },
    ToolBridge {
        claude_code_tools: &[HookTool::Agent],
        target_name: "task",
        representative_index: 0,
    },
];

/// Reverse-only entries: target names that map to CC tools but have no forward mapping.
const COPILOT_REVERSE_ONLY_TOOL_ENTRIES: &[ToolBridge] = &[ToolBridge {
    claude_code_tools: &[HookTool::Bash],
    target_name: "powershell",
    representative_index: 0,
}];

pub(crate) struct CopilotToolMap;

impl ToolMap for CopilotToolMap {
    fn map_tool(&self, tool: &str) -> String {
        let trimmed = tool.trim();
        let hook_tool = HookTool::from_str(trimmed);
        match to_target_tool(COPILOT_TOOL_ENTRIES, &hook_tool) {
            Some(target) => target.to_string(),
            None => trimmed.to_string(),
        }
    }
}

impl CopilotToolMap {
    /// Reverse lookup: Copilot tool name -> Claude Code tool name.
    /// Searches both the main table and reverse-only entries.
    /// Unknown names are passed through.
    pub fn reverse_map_tool(&self, tool: &str) -> String {
        let trimmed = tool.trim();
        if let Some(hook_tool) = to_source_tool(COPILOT_TOOL_ENTRIES, trimmed) {
            return hook_tool.as_str().to_string();
        }
        if let Some(hook_tool) = to_source_tool(COPILOT_REVERSE_ONLY_TOOL_ENTRIES, trimmed) {
            return hook_tool.as_str().to_string();
        }
        trimmed.to_string()
    }
}
