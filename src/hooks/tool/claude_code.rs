/// Claude Code side hook tool enum.
///
/// Known tools have dedicated variants; unknown tools use `Other(String)`.
/// `Other` cannot appear in static mapping tables (non-`'static`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HookTool {
    Bash,
    Read,
    Write,
    Edit,
    MultiEdit,
    Glob,
    Grep,
    WebFetch,
    Agent,
    Other(String),
}

impl HookTool {
    /// Parse a Claude Code tool name string into a `HookTool`.
    /// Unknown tools become `Other(s)`.
    ///
    /// # Arguments
    ///
    /// * `s` - raw tool name string from the hook payload
    pub fn from_str(s: &str) -> Self {
        match s {
            "Bash" => Self::Bash,
            "Read" => Self::Read,
            "Write" => Self::Write,
            "Edit" => Self::Edit,
            "MultiEdit" => Self::MultiEdit,
            "Glob" => Self::Glob,
            "Grep" => Self::Grep,
            "WebFetch" => Self::WebFetch,
            "Agent" => Self::Agent,
            other => Self::Other(other.to_string()),
        }
    }

    /// Return the Claude Code name for this tool.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Bash => "Bash",
            Self::Read => "Read",
            Self::Write => "Write",
            Self::Edit => "Edit",
            Self::MultiEdit => "MultiEdit",
            Self::Glob => "Glob",
            Self::Grep => "Grep",
            Self::WebFetch => "WebFetch",
            Self::Agent => "Agent",
            Self::Other(s) => s,
        }
    }
}

/// Tool name mapping supporting N:1 (multiple CC tools -> one target name).
pub(crate) struct ToolBridge {
    pub claude_code_tools: &'static [HookTool],
    pub target_name: &'static str,
    pub representative_index: usize,
}

/// Forward lookup: HookTool -> target tool name.
/// Returns `None` for `Other` variants (not in table).
///
/// # Arguments
///
/// * `table` - tool bridge table to search
/// * `tool` - Claude Code side hook tool to look up
pub(crate) fn to_target_tool(table: &[ToolBridge], tool: &HookTool) -> Option<&'static str> {
    table
        .iter()
        .find(|entry| entry.claude_code_tools.contains(tool))
        .map(|entry| entry.target_name)
}

/// Reverse lookup: target tool name -> representative HookTool.
///
/// # Arguments
///
/// * `table` - tool bridge table to search
/// * `target_name` - target-side tool name to resolve
pub(crate) fn to_source_tool<'a>(
    table: &'a [ToolBridge],
    target_name: &str,
) -> Option<&'a HookTool> {
    table
        .iter()
        .find(|entry| entry.target_name == target_name)
        .map(|entry| &entry.claude_code_tools[entry.representative_index])
}
