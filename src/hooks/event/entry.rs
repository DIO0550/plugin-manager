/// Claude Code side hook event enum.
///
/// Known events have dedicated variants. Unknown or excluded events
/// (PostToolUseFailure, PreCompact, etc.) use `Other(String)`,
/// preserving the original name for diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HookEvent {
    SessionStart,
    SessionEnd,
    PreToolUse,
    PostToolUse,
    UserPromptSubmit,
    Stop,
    SubagentStop,
    Other(String),
}

impl HookEvent {
    /// Parse a Claude Code event name string into a `HookEvent`.
    /// Unknown or excluded events become `Other(s)`.
    pub fn from_str(s: &str) -> Self {
        match s {
            "SessionStart" => Self::SessionStart,
            "SessionEnd" => Self::SessionEnd,
            "PreToolUse" => Self::PreToolUse,
            "PostToolUse" => Self::PostToolUse,
            "UserPromptSubmit" => Self::UserPromptSubmit,
            "Stop" => Self::Stop,
            "SubagentStop" => Self::SubagentStop,
            other => Self::Other(other.to_string()),
        }
    }
}

/// A single event name mapping entry (always 1:1).
pub(crate) struct HookEventEntry {
    pub event: HookEvent,
    pub target: &'static str,
}

/// Forward lookup: HookEvent -> target event name.
/// Returns `None` for `Other` variants (not in table).
pub(crate) fn to_target_event(table: &[HookEventEntry], event: &HookEvent) -> Option<&'static str> {
    table.iter().find(|e| e.event == *event).map(|e| e.target)
}

/// Reverse lookup: target event name -> HookEvent.
pub(crate) fn to_source_event(table: &[HookEventEntry], target_name: &str) -> Option<HookEvent> {
    table
        .iter()
        .find(|e| e.target == target_name)
        .map(|e| e.event.clone())
}
