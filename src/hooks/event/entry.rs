/// Claude Code side hook event enum.
///
/// Only includes the 7 events that have Copilot CLI equivalents.
/// Excluded events (PostToolUseFailure, PreCompact, etc.) are not represented;
/// `from_str` returns `None` for them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HookEvent {
    SessionStart,
    SessionEnd,
    PreToolUse,
    PostToolUse,
    UserPromptSubmit,
    Stop,
    SubagentStop,
}

impl HookEvent {
    /// Parse a Claude Code event name string into a `HookEvent`.
    /// Returns `None` for excluded or unknown events.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "SessionStart" => Some(Self::SessionStart),
            "SessionEnd" => Some(Self::SessionEnd),
            "PreToolUse" => Some(Self::PreToolUse),
            "PostToolUse" => Some(Self::PostToolUse),
            "UserPromptSubmit" => Some(Self::UserPromptSubmit),
            "Stop" => Some(Self::Stop),
            "SubagentStop" => Some(Self::SubagentStop),
            _ => None,
        }
    }
}

/// A single event name mapping entry (always 1:1).
pub(crate) struct HookEventEntry {
    pub event: HookEvent,
    pub target: &'static str,
}

/// Forward lookup: HookEvent -> target event name.
pub(crate) fn to_target_event(table: &[HookEventEntry], event: HookEvent) -> Option<&'static str> {
    table.iter().find(|e| e.event == event).map(|e| e.target)
}

/// Reverse lookup: target event name -> HookEvent.
pub(crate) fn to_source_event(table: &[HookEventEntry], target_name: &str) -> Option<HookEvent> {
    table
        .iter()
        .find(|e| e.target == target_name)
        .map(|e| e.event)
}
