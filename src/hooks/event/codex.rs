use crate::hooks::converter::EventMap;

/// Codex Hooks use PascalCase event names, matching the Claude Code shape for
/// the events that both runtimes support.
pub(crate) struct CodexEventMap;

impl EventMap for CodexEventMap {
    fn map_event(&self, event: &str) -> Option<&'static str> {
        match event {
            "SessionStart" => Some("SessionStart"),
            "PreToolUse" => Some("PreToolUse"),
            "PostToolUse" => Some("PostToolUse"),
            "UserPromptSubmit" => Some("UserPromptSubmit"),
            "Stop" => Some("Stop"),
            "PermissionRequest" => Some("PermissionRequest"),
            _ => None,
        }
    }
}
