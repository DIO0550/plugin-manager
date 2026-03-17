//! Event name and tool name mapping for Hooks context.
//!
//! This module handles mappings between Claude Code and Copilot CLI
//! for the Hooks stdin `hook_event_name` / `tool_name` / `toolName` fields.
//!
//! Note: This is distinct from `parser::convert` which handles
//! Prompt/Agent file `tools` arrays (N:1 mapping like Read/Write/Edit -> codebase).
//! The hooks context uses 1:1 mappings (view -> Read, create -> Write, edit -> Edit).

/// Convert a Claude Code event name (PascalCase) to Copilot CLI event name (camelCase).
///
/// Returns `None` for excluded events (no Copilot CLI equivalent)
/// and unknown event names.
///
/// # Supported events (7):
/// - SessionStart -> sessionStart
/// - SessionEnd -> sessionEnd
/// - PreToolUse -> preToolUse
/// - PostToolUse -> postToolUse
/// - UserPromptSubmit -> userPromptSubmitted
/// - Stop -> agentStop
/// - SubagentStop -> subagentStop
///
/// # Excluded events (14):
/// PostToolUseFailure, PreCompact, PostCompact, PermissionRequest,
/// Notification, SubagentStart, TeammateIdle, TaskCompleted,
/// InstructionsLoaded, ConfigChange, WorktreeCreate, WorktreeRemove,
/// Elicitation, ElicitationResult
pub fn event_claude_to_copilot(event: &str) -> Option<&'static str> {
    match event.trim() {
        "SessionStart" => Some("sessionStart"),
        "SessionEnd" => Some("sessionEnd"),
        "PreToolUse" => Some("preToolUse"),
        "PostToolUse" => Some("postToolUse"),
        "UserPromptSubmit" => Some("userPromptSubmitted"),
        "Stop" => Some("agentStop"),
        "SubagentStop" => Some("subagentStop"),
        _ => None,
    }
}

/// Convert a Copilot CLI event name (camelCase) to Claude Code event name (PascalCase).
///
/// Returns `None` for unknown event names.
pub fn event_copilot_to_claude(event: &str) -> Option<&'static str> {
    match event.trim() {
        "sessionStart" => Some("SessionStart"),
        "sessionEnd" => Some("SessionEnd"),
        "preToolUse" => Some("PreToolUse"),
        "postToolUse" => Some("PostToolUse"),
        "userPromptSubmitted" => Some("UserPromptSubmit"),
        "agentStop" => Some("Stop"),
        "subagentStop" => Some("SubagentStop"),
        _ => None,
    }
}

/// Convert a Copilot CLI tool name to Claude Code tool name (Hooks context).
///
/// Unknown tool names are returned as-is (forward compatibility).
/// This is distinct from `parser::convert::tool_copilot_to_claude` which handles
/// Prompt/Agent file tools (e.g., "codebase" -> "Read").
pub fn tool_copilot_to_claude(tool: &str) -> String {
    match tool.trim() {
        "bash" => "Bash".to_string(),
        "view" => "Read".to_string(),
        "create" => "Write".to_string(),
        "edit" => "Edit".to_string(),
        "glob" => "Glob".to_string(),
        "grep" => "Grep".to_string(),
        "web_fetch" => "WebFetch".to_string(),
        "task" => "Agent".to_string(),
        "powershell" => "Bash".to_string(),
        other => other.to_string(),
    }
}

/// Convert a Claude Code tool name to Copilot CLI tool name (Hooks context).
///
/// Unknown tool names are returned as-is (forward compatibility).
/// This is distinct from `parser::convert::tool_claude_to_copilot` which handles
/// Prompt/Agent file tools (e.g., "Read" -> "codebase").
pub fn tool_claude_to_copilot(tool: &str) -> String {
    match tool.trim() {
        "Bash" => "bash".to_string(),
        "Read" => "view".to_string(),
        "Write" => "create".to_string(),
        "Edit" | "MultiEdit" => "edit".to_string(),
        "Glob" => "glob".to_string(),
        "Grep" => "grep".to_string(),
        "WebFetch" => "web_fetch".to_string(),
        "Agent" => "task".to_string(),
        other => other.to_string(),
    }
}
