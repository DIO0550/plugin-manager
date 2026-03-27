//! Conversion utilities for SlashCommand formats.
//!
//! Provides mappings between Claude Code, Copilot, and Codex tool/model names.

/// Target format trait for conversion output.
///
/// Implemented by all format types that can be conversion targets.
pub trait TargetFormat {
    /// Serialize to Markdown format (infallible).
    fn to_markdown(&self) -> String;
}

/// Target type for format conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetType {
    /// Copilot prompt format
    Copilot,
    /// Codex prompt format
    Codex,
}

pub use crate::format::Format;

fn lookup_forward<'a>(map: &'a [(&'a str, &'a str)], key: &str) -> Option<&'a str> {
    map.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

fn lookup_reverse<'a>(map: &'a [(&'a str, &'a str)], key: &str) -> Option<&'a str> {
    map.iter().find(|(_, v)| *v == key).map(|(k, _)| *k)
}

const PROMPT_TOOL_MAP: &[(&str, &str)] = &[
    ("Read", "codebase"), // representative for reverse lookup
    ("Write", "codebase"),
    ("Edit", "codebase"),
    ("Grep", "search/codebase"), // representative for reverse lookup
    ("Glob", "search/codebase"),
    ("Bash", "terminal"),
    ("WebFetch", "fetch"),
    ("WebSearch", "websearch"),
];

/// Tool name conversion between Claude Code and Copilot (Prompt/Agent context).
///
/// N:1 reverse lookups return the first table entry as the representative value
/// (e.g., "codebase" -> "Read", "search/codebase" -> "Grep").
pub(crate) fn map_tool(tool: &str, from: Format, to: Format) -> String {
    let trimmed = tool.trim();
    match (from, to) {
        (Format::ClaudeCode, Format::Copilot) => {
            if let Some(v) = lookup_forward(PROMPT_TOOL_MAP, trimmed) {
                return v.to_string();
            }
            if trimmed.starts_with("Bash(git") {
                return "githubRepo".to_string();
            }
            trimmed.to_string()
        }
        (Format::Copilot, Format::ClaudeCode) => {
            if trimmed == "githubRepo" {
                return "Bash".to_string();
            }
            lookup_reverse(PROMPT_TOOL_MAP, trimmed)
                .map(|v| v.to_string())
                .unwrap_or_else(|| trimmed.to_string())
        }
        _ => unreachable!("map_tool: unsupported conversion ({:?}, {:?})", from, to),
    }
}

/// Convert tool array with deduplication.
pub(crate) fn map_tools(tools: &[String], from: Format, to: Format) -> Vec<String> {
    let mut result: Vec<String> = tools.iter().map(|t| map_tool(t, from, to)).collect();
    result.sort();
    result.dedup();
    result
}

/// Keys are lowercase-normalized
const MODEL_CLAUDE_COPILOT_MAP: &[(&str, &str)] = &[
    ("haiku", "GPT-4o-mini"),
    ("sonnet", "GPT-4o"),
    ("opus", "o1"),
];

/// Keys are lowercase-normalized (reverse of MODEL_CLAUDE_COPILOT_MAP)
const MODEL_COPILOT_CLAUDE_MAP: &[(&str, &str)] = &[
    ("gpt-4o-mini", "haiku"),
    ("gpt-4o", "sonnet"),
    ("o1", "opus"),
];

/// Keys are lowercase-normalized
const MODEL_CLAUDE_CODEX_MAP: &[(&str, &str)] = &[
    ("haiku", "gpt-4.1-mini"),
    ("sonnet", "gpt-4.1"),
    ("opus", "o3"),
];

/// Model name conversion between formats.
///
/// Input is normalized to lowercase before lookup. Passthrough returns the
/// normalized (lowercase) value.
pub(crate) fn map_model(model: &str, from: Format, to: Format) -> String {
    let normalized = model.to_lowercase();
    let table = match (from, to) {
        (Format::ClaudeCode, Format::Copilot) => MODEL_CLAUDE_COPILOT_MAP,
        (Format::Copilot, Format::ClaudeCode) => MODEL_COPILOT_CLAUDE_MAP,
        (Format::ClaudeCode, Format::Codex) => MODEL_CLAUDE_CODEX_MAP,
        _ => unreachable!("map_model: unsupported conversion ({:?}, {:?})", from, to),
    };
    lookup_forward(table, &normalized)
        .map(|v| v.to_string())
        .unwrap_or(normalized)
}

/// Body variable conversion: Claude Code -> Copilot.
///
/// Converts `$ARGUMENTS` to `${arguments}` and `$1`-`$9` to `${arg1}`-`${arg9}`.
/// Note: Replaces from $9 to $1 to avoid partial replacement issues.
pub fn body_claude_to_copilot(body: &str) -> String {
    body.replace("$ARGUMENTS", "${arguments}")
        .replace("$9", "${arg9}")
        .replace("$8", "${arg8}")
        .replace("$7", "${arg7}")
        .replace("$6", "${arg6}")
        .replace("$5", "${arg5}")
        .replace("$4", "${arg4}")
        .replace("$3", "${arg3}")
        .replace("$2", "${arg2}")
        .replace("$1", "${arg1}")
}

/// Body variable conversion: Copilot -> Claude Code.
///
/// Converts `${arguments}` to `$ARGUMENTS` and `${arg1}`-`${arg9}` to `$1`-`$9`.
/// Note: Replaces from ${arg9} to ${arg1} to avoid partial replacement issues.
pub fn body_copilot_to_claude(body: &str) -> String {
    body.replace("${arguments}", "$ARGUMENTS")
        .replace("${arg9}", "$9")
        .replace("${arg8}", "$8")
        .replace("${arg7}", "$7")
        .replace("${arg6}", "$6")
        .replace("${arg5}", "$5")
        .replace("${arg4}", "$4")
        .replace("${arg3}", "$3")
        .replace("${arg2}", "$2")
        .replace("${arg1}", "$1")
}

/// Parse allowed-tools string (comma-separated).
pub fn parse_allowed_tools(tools: &str) -> Vec<String> {
    tools
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Format tool array as allowed-tools string.
pub fn format_allowed_tools(tools: &[String]) -> String {
    tools.join(", ")
}

/// Escape YAML string value.
///
/// Wraps in double quotes and escapes special characters if needed.
pub fn escape_yaml_string(s: &str) -> String {
    let needs_quote = s.contains(':')
        || s.contains('"')
        || s.contains('#')
        || s.contains('\n')
        || s.starts_with(' ')
        || s.ends_with(' ');

    if needs_quote {
        let escaped = s
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n");
        format!("\"{}\"", escaped)
    } else {
        s.to_string()
    }
}
