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

struct ToolRow {
    claude_code: &'static str,
    copilot: Option<&'static str>,
    /// 将来拡張用（Codex Tool 名マッピングが仕様化されたら Some(...) を埋める）
    #[allow(dead_code)]
    codex: Option<&'static str>,
    reverse_canonical: bool,
}

type ToolColumn = fn(&ToolRow) -> Option<&'static str>;

fn tool_col_claude_code(row: &ToolRow) -> Option<&'static str> {
    Some(row.claude_code)
}
fn tool_col_copilot(row: &ToolRow) -> Option<&'static str> {
    row.copilot
}
#[allow(dead_code)]
fn tool_col_codex(row: &ToolRow) -> Option<&'static str> {
    row.codex
}

const TOOL_TABLE: &[ToolRow] = &[
    ToolRow {
        claude_code: "Read",
        copilot: Some("codebase"),
        codex: None,
        reverse_canonical: true,
    },
    ToolRow {
        claude_code: "Write",
        copilot: Some("codebase"),
        codex: None,
        reverse_canonical: false,
    },
    ToolRow {
        claude_code: "Edit",
        copilot: Some("codebase"),
        codex: None,
        reverse_canonical: false,
    },
    ToolRow {
        claude_code: "Grep",
        copilot: Some("search/codebase"),
        codex: None,
        reverse_canonical: true,
    },
    ToolRow {
        claude_code: "Glob",
        copilot: Some("search/codebase"),
        codex: None,
        reverse_canonical: false,
    },
    ToolRow {
        claude_code: "Bash",
        copilot: Some("terminal"),
        codex: None,
        reverse_canonical: true,
    },
    ToolRow {
        claude_code: "WebFetch",
        copilot: Some("fetch"),
        codex: None,
        reverse_canonical: true,
    },
    ToolRow {
        claude_code: "WebSearch",
        copilot: Some("websearch"),
        codex: None,
        reverse_canonical: true,
    },
];

fn find_forward_tool(key: &str, from_col: ToolColumn, to_col: ToolColumn) -> Option<&'static str> {
    TOOL_TABLE
        .iter()
        .find(|row| from_col(row) == Some(key))
        .and_then(to_col)
}

fn find_reverse_canonical_tool(
    value: &str,
    from_col: ToolColumn,
    to_col: ToolColumn,
) -> Option<&'static str> {
    TOOL_TABLE
        .iter()
        .find(|row| row.reverse_canonical && from_col(row) == Some(value))
        .and_then(to_col)
}

fn apply_tool_special_forward(trimmed: &str) -> Option<String> {
    if trimmed.starts_with("Bash(git") {
        Some("githubRepo".to_string())
    } else {
        None
    }
}

fn apply_tool_special_reverse(trimmed: &str) -> Option<String> {
    if trimmed == "githubRepo" {
        Some("Bash".to_string())
    } else {
        None
    }
}

/// Tool name conversion between Claude Code and Copilot (Prompt/Agent context).
///
/// N:1 reverse lookups return the first table entry as the representative value
/// (e.g., "codebase" -> "Read", "search/codebase" -> "Grep").
///
/// # Arguments
///
/// * `tool` - Tool name to convert (leading/trailing whitespace is trimmed).
/// * `from` - Source format of the input tool name.
/// * `to` - Destination format to convert the tool name into.
pub(crate) fn map_tool(tool: &str, from: Format, to: Format) -> String {
    let trimmed = tool.trim();
    match (from, to) {
        (Format::ClaudeCode, Format::Copilot) => {
            if let Some(v) = find_forward_tool(trimmed, tool_col_claude_code, tool_col_copilot) {
                return v.to_string();
            }
            if let Some(v) = apply_tool_special_forward(trimmed) {
                return v;
            }
            trimmed.to_string()
        }
        (Format::Copilot, Format::ClaudeCode) => {
            if let Some(v) = apply_tool_special_reverse(trimmed) {
                return v;
            }
            find_reverse_canonical_tool(trimmed, tool_col_copilot, tool_col_claude_code)
                .map(|v| v.to_string())
                .unwrap_or_else(|| trimmed.to_string())
        }
        // For other format pairs (including Codex), leave the tool name unchanged.
        _ => trimmed.to_string(),
    }
}

/// Convert tool array with deduplication.
///
/// # Arguments
///
/// * `tools` - Tool names to convert.
/// * `from` - Source format of the input tool names.
/// * `to` - Destination format to convert the tool names into.
pub(crate) fn map_tools(tools: &[String], from: Format, to: Format) -> Vec<String> {
    let mut result: Vec<String> = tools.iter().map(|t| map_tool(t, from, to)).collect();
    result.sort();
    result.dedup();
    result
}

struct ModelRow {
    claude_code: &'static str,
    copilot: Option<&'static str>,
    codex: Option<&'static str>,
}

type ModelColumn = fn(&ModelRow) -> Option<&'static str>;

fn model_col_claude_code(row: &ModelRow) -> Option<&'static str> {
    Some(row.claude_code)
}
fn model_col_copilot(row: &ModelRow) -> Option<&'static str> {
    row.copilot
}
fn model_col_codex(row: &ModelRow) -> Option<&'static str> {
    row.codex
}

const MODEL_TABLE: &[ModelRow] = &[
    ModelRow {
        claude_code: "haiku",
        copilot: Some("GPT-4o-mini"),
        codex: Some("gpt-4.1-mini"),
    },
    ModelRow {
        claude_code: "sonnet",
        copilot: Some("GPT-4o"),
        codex: Some("gpt-4.1"),
    },
    ModelRow {
        claude_code: "opus",
        copilot: Some("o1"),
        codex: Some("o3"),
    },
];

/// `from_col(row)` を lowercase 化した値が `key_lower` と一致する最初の行の `to_col` を返す。
/// 入力 `key_lower` は呼び出し側で lowercase 化済みの前提（map_model で `to_lowercase()` 済み）。
/// 列値が大文字混じり（例: "GPT-4o-mini"）の Copilot 列でも、lowercase 比較で双方向に引ける。
fn find_model_lower(
    key_lower: &str,
    from_col: ModelColumn,
    to_col: ModelColumn,
) -> Option<&'static str> {
    MODEL_TABLE
        .iter()
        .find(|row| from_col(row).map(|s| s.to_lowercase()).as_deref() == Some(key_lower))
        .and_then(to_col)
}

/// Model name conversion between formats.
///
/// Input is normalized to lowercase before lookup. If a mapping table exists
/// for the given `(from, to)` pair, the normalized name is looked up there; if
/// no table exists (unsupported format pair) or the key is missing, the
/// normalized (lowercase) value is returned unchanged.
///
/// # Arguments
///
/// * `model` - Model name to convert (normalized to lowercase before lookup).
/// * `from` - Source format of the input model name.
/// * `to` - Destination format to convert the model name into.
pub(crate) fn map_model(model: &str, from: Format, to: Format) -> String {
    let normalized = model.to_lowercase();
    let columns: Option<(ModelColumn, ModelColumn)> = match (from, to) {
        (Format::ClaudeCode, Format::Copilot) => Some((model_col_claude_code, model_col_copilot)),
        (Format::Copilot, Format::ClaudeCode) => Some((model_col_copilot, model_col_claude_code)),
        (Format::ClaudeCode, Format::Codex) => Some((model_col_claude_code, model_col_codex)),
        _ => None,
    };

    match columns {
        Some((from_col, to_col)) => find_model_lower(&normalized, from_col, to_col)
            .map(|v| v.to_string())
            .unwrap_or(normalized),
        None => normalized,
    }
}

/// Body variable conversion: Claude Code -> Copilot.
///
/// Converts `$ARGUMENTS` to `${arguments}` and `$1`-`$9` to `${arg1}`-`${arg9}`.
/// Note: Replaces from $9 to $1 to avoid partial replacement issues.
///
/// # Arguments
///
/// * `body` - Claude Code prompt body containing `$ARGUMENTS` / `$1`-`$9` placeholders.
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
///
/// # Arguments
///
/// * `body` - Copilot prompt body containing `${arguments}` / `${arg1}`-`${arg9}` placeholders.
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
///
/// # Arguments
///
/// * `tools` - Comma-separated tool list (e.g. `"Bash(git:*), Read"`).
pub fn parse_allowed_tools(tools: &str) -> Vec<String> {
    tools
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Format tool array as allowed-tools string.
///
/// # Arguments
///
/// * `tools` - Tool names to join into a comma-separated list.
pub fn format_allowed_tools(tools: &[String]) -> String {
    tools.join(", ")
}

/// Escape YAML string value.
///
/// Wraps in double quotes and escapes special characters if needed.
///
/// # Arguments
///
/// * `s` - String value to be embedded as a YAML scalar.
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
