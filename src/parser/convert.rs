//! Conversion utilities for SlashCommand formats.
//!
//! Provides mappings between Claude Code, Copilot, and Codex tool/model names.

/// Claude Code -> Copilot tool name conversion.
pub fn tool_claude_to_copilot(tool: &str) -> String {
    match tool.trim() {
        "Read" | "Write" | "Edit" => "codebase".to_string(),
        "Grep" | "Glob" => "search/codebase".to_string(),
        "Bash" => "terminal".to_string(),
        t if t.starts_with("Bash(git") => "githubRepo".to_string(),
        "WebFetch" => "fetch".to_string(),
        "WebSearch" => "websearch".to_string(),
        other => other.to_string(),
    }
}

/// Copilot -> Claude Code tool name conversion.
///
/// Note: 1:N mappings (codebase -> Read, Write, Edit) return a representative value.
/// Complete restoration is not guaranteed in round-trips.
pub fn tool_copilot_to_claude(tool: &str) -> String {
    match tool.trim() {
        "codebase" => "Read".to_string(),
        "search/codebase" => "Grep".to_string(),
        "terminal" => "Bash".to_string(),
        "githubRepo" => "Bash".to_string(),
        "fetch" => "WebFetch".to_string(),
        "websearch" => "WebSearch".to_string(),
        other => other.to_string(),
    }
}

/// Convert tool array with deduplication.
pub fn tools_claude_to_copilot(tools: &[String]) -> Vec<String> {
    let mut result: Vec<String> = tools.iter().map(|t| tool_claude_to_copilot(t)).collect();
    result.sort();
    result.dedup();
    result
}

/// Claude Code -> Copilot model name conversion.
pub fn model_claude_to_copilot(model: &str) -> String {
    match model.to_lowercase().as_str() {
        "haiku" => "GPT-4o-mini".to_string(),
        "sonnet" => "GPT-4o".to_string(),
        "opus" => "o1".to_string(),
        other => other.to_string(),
    }
}

/// Copilot -> Claude Code model name conversion.
pub fn model_copilot_to_claude(model: &str) -> String {
    match model.to_lowercase().as_str() {
        "gpt-4o-mini" => "haiku".to_string(),
        "gpt-4o" => "sonnet".to_string(),
        "o1" => "opus".to_string(),
        other => other.to_string(),
    }
}

/// Claude Code -> Codex model name conversion.
pub fn model_claude_to_codex(model: &str) -> String {
    match model.to_lowercase().as_str() {
        "haiku" => "gpt-4.1-mini".to_string(),
        "sonnet" => "gpt-4.1".to_string(),
        "opus" => "o3".to_string(),
        other => other.to_string(),
    }
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
