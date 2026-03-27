//! Event name and tool name mapping for Hooks context.
//!
//! This module handles mappings between Claude Code and Copilot CLI
//! for the Hooks stdin `hook_event_name` / `tool_name` / `toolName` fields.
//!
//! Note: This is distinct from `parser::convert` which handles
//! Prompt/Agent file `tools` arrays (N:1 mapping like Read/Write/Edit -> codebase).
//! The hooks context uses 1:1 mappings (view -> Read, create -> Write, edit -> Edit).

use crate::format::Format;

fn lookup_forward(map: &[(&'static str, &'static str)], key: &str) -> Option<&'static str> {
    map.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

fn lookup_reverse(map: &[(&'static str, &'static str)], key: &str) -> Option<&'static str> {
    map.iter().find(|(_, v)| *v == key).map(|(k, _)| *k)
}

const HOOK_EVENT_MAP: &[(&str, &str)] = &[
    ("SessionStart", "sessionStart"),
    ("SessionEnd", "sessionEnd"),
    ("PreToolUse", "preToolUse"),
    ("PostToolUse", "postToolUse"),
    ("UserPromptSubmit", "userPromptSubmitted"),
    ("Stop", "agentStop"),
    ("SubagentStop", "subagentStop"),
];

// NOTE: A similar (but not identical) tool mapping exists inline as a jq
// object literal in hooks/copilot.rs build_env_bridge(). Known difference:
// jq includes "powershell" -> "Bash" which is not in this table (passthrough
// here). When modifying this table, check that jq template as well.
const HOOK_TOOL_MAP: &[(&str, &str)] = &[
    ("Bash", "bash"),
    ("Read", "view"),
    ("Write", "create"),
    ("Edit", "edit"), // representative for reverse lookup
    ("MultiEdit", "edit"),
    ("Glob", "glob"),
    ("Grep", "grep"),
    ("WebFetch", "web_fetch"),
    ("Agent", "task"),
];

/// Event name conversion between Claude Code and Copilot CLI (Hooks context).
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
pub(crate) fn map_event(event: &str, from: Format, to: Format) -> Option<&'static str> {
    debug_assert!(
        matches!(
            (from, to),
            (Format::ClaudeCode, Format::Copilot) | (Format::Copilot, Format::ClaudeCode)
        ),
        "map_event: unsupported conversion ({:?}, {:?})",
        from,
        to
    );
    let trimmed = event.trim();
    match (from, to) {
        (Format::ClaudeCode, Format::Copilot) => lookup_forward(HOOK_EVENT_MAP, trimmed),
        (Format::Copilot, Format::ClaudeCode) => lookup_reverse(HOOK_EVENT_MAP, trimmed),
        _ => None,
    }
}

/// Tool name conversion between Claude Code and Copilot CLI (Hooks context).
///
/// This is a best-effort mapping. Input is trimmed before matching.
/// Unknown tool names are passed through in trimmed form for forward
/// compatibility.
///
/// N:1 reverse lookup returns the first table entry as the representative
/// value (e.g., "edit" -> "Edit").
///
/// This is distinct from `parser::convert::map_tool` which handles
/// Prompt/Agent file tools (e.g., "Read" -> "codebase").
pub(crate) fn map_tool(tool: &str, from: Format, to: Format) -> String {
    debug_assert!(
        matches!(
            (from, to),
            (Format::ClaudeCode, Format::Copilot) | (Format::Copilot, Format::ClaudeCode)
        ),
        "map_tool: unsupported conversion ({:?}, {:?})",
        from,
        to
    );
    let trimmed = tool.trim();
    match (from, to) {
        (Format::ClaudeCode, Format::Copilot) => lookup_forward(HOOK_TOOL_MAP, trimmed)
            .map(|v| v.to_string())
            .unwrap_or_else(|| trimmed.to_string()),
        (Format::Copilot, Format::ClaudeCode) => lookup_reverse(HOOK_TOOL_MAP, trimmed)
            .map(|v| v.to_string())
            .unwrap_or_else(|| trimmed.to_string()),
        _ => trimmed.to_string(),
    }
}
