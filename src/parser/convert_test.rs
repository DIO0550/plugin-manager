//! Tests for convert module.

use super::claude_code::ClaudeCodeCommand;
use super::convert::*;

// ============================================================================
// Tool name conversion tests
// ============================================================================

#[test]
fn test_map_tool_claude_to_copilot_file_operations() {
    assert_eq!(
        map_tool("Read", Format::ClaudeCode, Format::Copilot),
        "codebase"
    );
    assert_eq!(
        map_tool("Write", Format::ClaudeCode, Format::Copilot),
        "codebase"
    );
    assert_eq!(
        map_tool("Edit", Format::ClaudeCode, Format::Copilot),
        "codebase"
    );
}

#[test]
fn test_map_tool_claude_to_copilot_search_operations() {
    assert_eq!(
        map_tool("Grep", Format::ClaudeCode, Format::Copilot),
        "search/codebase"
    );
    assert_eq!(
        map_tool("Glob", Format::ClaudeCode, Format::Copilot),
        "search/codebase"
    );
}

#[test]
fn test_map_tool_claude_to_copilot_bash() {
    assert_eq!(
        map_tool("Bash", Format::ClaudeCode, Format::Copilot),
        "terminal"
    );
    assert_eq!(
        map_tool("Bash(git:*)", Format::ClaudeCode, Format::Copilot),
        "githubRepo"
    );
    assert_eq!(
        map_tool("Bash(git commit*)", Format::ClaudeCode, Format::Copilot),
        "githubRepo"
    );
}

#[test]
fn test_map_tool_claude_to_copilot_web() {
    assert_eq!(
        map_tool("WebFetch", Format::ClaudeCode, Format::Copilot),
        "fetch"
    );
    assert_eq!(
        map_tool("WebSearch", Format::ClaudeCode, Format::Copilot),
        "websearch"
    );
}

#[test]
fn test_map_tool_claude_to_copilot_unknown() {
    assert_eq!(
        map_tool("UnknownTool", Format::ClaudeCode, Format::Copilot),
        "UnknownTool"
    );
    assert_eq!(
        map_tool("CustomMCP", Format::ClaudeCode, Format::Copilot),
        "CustomMCP"
    );
}

#[test]
fn test_map_tool_claude_to_copilot_with_whitespace() {
    assert_eq!(
        map_tool(" Read ", Format::ClaudeCode, Format::Copilot),
        "codebase"
    );
    assert_eq!(
        map_tool("  Bash  ", Format::ClaudeCode, Format::Copilot),
        "terminal"
    );
}

#[test]
fn test_map_tool_copilot_to_claude() {
    assert_eq!(
        map_tool("codebase", Format::Copilot, Format::ClaudeCode),
        "Read"
    );
    assert_eq!(
        map_tool("search/codebase", Format::Copilot, Format::ClaudeCode),
        "Grep"
    );
    assert_eq!(
        map_tool("terminal", Format::Copilot, Format::ClaudeCode),
        "Bash"
    );
    assert_eq!(
        map_tool("githubRepo", Format::Copilot, Format::ClaudeCode),
        "Bash"
    );
    assert_eq!(
        map_tool("fetch", Format::Copilot, Format::ClaudeCode),
        "WebFetch"
    );
    assert_eq!(
        map_tool("websearch", Format::Copilot, Format::ClaudeCode),
        "WebSearch"
    );
}

#[test]
fn test_map_tool_copilot_to_claude_unknown() {
    assert_eq!(
        map_tool("unknownTool", Format::Copilot, Format::ClaudeCode),
        "unknownTool"
    );
}

#[test]
fn test_map_tools_deduplication() {
    let tools = vec![
        "Read".to_string(),
        "Write".to_string(),
        "Edit".to_string(),
        "Bash".to_string(),
    ];
    let result = map_tools(&tools, Format::ClaudeCode, Format::Copilot);
    // Read, Write, Edit all map to "codebase", so should be deduplicated
    assert_eq!(result.len(), 2);
    assert!(result.contains(&"codebase".to_string()));
    assert!(result.contains(&"terminal".to_string()));
}

#[test]
fn test_map_tools_sorted() {
    let tools = vec!["WebSearch".to_string(), "Read".to_string()];
    let result = map_tools(&tools, Format::ClaudeCode, Format::Copilot);
    // Should be sorted alphabetically
    assert_eq!(result, vec!["codebase", "websearch"]);
}

#[test]
fn test_map_tool_copilot_to_claude_n_to_1_representative() {
    // N:1 reverse lookup returns first table entry as representative
    assert_eq!(
        map_tool("codebase", Format::Copilot, Format::ClaudeCode),
        "Read"
    );
}

#[test]
fn test_map_tool_claude_to_copilot_n_to_1_forward_all_rows() {
    // forward は全行で機能する（Read/Write/Edit すべて codebase に飛ぶ）
    assert_eq!(
        map_tool("Read", Format::ClaudeCode, Format::Copilot),
        "codebase"
    );
    assert_eq!(
        map_tool("Write", Format::ClaudeCode, Format::Copilot),
        "codebase"
    );
    assert_eq!(
        map_tool("Edit", Format::ClaudeCode, Format::Copilot),
        "codebase"
    );
    assert_eq!(
        map_tool("Grep", Format::ClaudeCode, Format::Copilot),
        "search/codebase"
    );
    assert_eq!(
        map_tool("Glob", Format::ClaudeCode, Format::Copilot),
        "search/codebase"
    );
}

#[test]
fn test_map_tool_copilot_to_claude_n_to_1_canonical_grep() {
    // reverse は代表行のみがヒット
    assert_eq!(
        map_tool("search/codebase", Format::Copilot, Format::ClaudeCode),
        "Grep"
    );
    assert_eq!(
        map_tool("codebase", Format::Copilot, Format::ClaudeCode),
        "Read"
    );
}

#[test]
fn test_map_tool_claude_to_copilot_bash_git_prefix_match() {
    // PIN: Bash(git で始まる入力は githubRepo に変換される（既存挙動）
    assert_eq!(
        map_tool("Bash(git:*)", Format::ClaudeCode, Format::Copilot),
        "githubRepo"
    );
    assert_eq!(
        map_tool("Bash(git commit*)", Format::ClaudeCode, Format::Copilot),
        "githubRepo"
    );

    // NOTE: Bash(github) のような入力も現行の starts_with("Bash(git") では
    // 偶発的に githubRepo にマッチする。これは本リファクタの保証範囲外
    // (non-guaranteed incidental match) であり、将来仕様化する余地を残す。
    // 意図的に assert はせず、振る舞いをコメントとして記録する。
}

// ============================================================================
// Model name conversion tests
// ============================================================================

#[test]
fn test_map_model_claude_to_copilot() {
    assert_eq!(
        map_model("haiku", Format::ClaudeCode, Format::Copilot),
        "GPT-4o-mini"
    );
    assert_eq!(
        map_model("sonnet", Format::ClaudeCode, Format::Copilot),
        "GPT-4o"
    );
    assert_eq!(map_model("opus", Format::ClaudeCode, Format::Copilot), "o1");
}

#[test]
fn test_map_model_claude_to_copilot_case_insensitive() {
    assert_eq!(
        map_model("Haiku", Format::ClaudeCode, Format::Copilot),
        "GPT-4o-mini"
    );
    assert_eq!(
        map_model("SONNET", Format::ClaudeCode, Format::Copilot),
        "GPT-4o"
    );
    assert_eq!(map_model("OPUS", Format::ClaudeCode, Format::Copilot), "o1");
}

#[test]
fn test_map_model_claude_to_copilot_unknown() {
    assert_eq!(
        map_model("custom-model", Format::ClaudeCode, Format::Copilot),
        "custom-model"
    );
}

#[test]
fn test_map_model_copilot_to_claude() {
    assert_eq!(
        map_model("GPT-4o-mini", Format::Copilot, Format::ClaudeCode),
        "haiku"
    );
    assert_eq!(
        map_model("GPT-4o", Format::Copilot, Format::ClaudeCode),
        "sonnet"
    );
    assert_eq!(map_model("o1", Format::Copilot, Format::ClaudeCode), "opus");
}

#[test]
fn test_map_model_copilot_to_claude_case_insensitive() {
    assert_eq!(
        map_model("gpt-4o-mini", Format::Copilot, Format::ClaudeCode),
        "haiku"
    );
    assert_eq!(
        map_model("gpt-4o", Format::Copilot, Format::ClaudeCode),
        "sonnet"
    );
    assert_eq!(map_model("O1", Format::Copilot, Format::ClaudeCode), "opus");
}

#[test]
fn test_map_model_copilot_to_claude_unknown() {
    assert_eq!(
        map_model("custom-model", Format::Copilot, Format::ClaudeCode),
        "custom-model"
    );
}

#[test]
fn test_map_model_claude_to_codex() {
    assert_eq!(
        map_model("haiku", Format::ClaudeCode, Format::Codex),
        "gpt-4.1-mini"
    );
    assert_eq!(
        map_model("sonnet", Format::ClaudeCode, Format::Codex),
        "gpt-4.1"
    );
    assert_eq!(map_model("opus", Format::ClaudeCode, Format::Codex), "o3");
}

#[test]
fn test_map_model_passthrough_normalization() {
    // Passthrough returns lowercase-normalized value
    assert_eq!(
        map_model("UNKNOWN", Format::ClaudeCode, Format::Copilot),
        "unknown"
    );
}

// ============================================================================
// Body variable conversion tests
// ============================================================================

#[test]
fn test_body_claude_to_copilot_arguments() {
    let body = "Process $ARGUMENTS";
    assert_eq!(body_claude_to_copilot(body), "Process ${arguments}");
}

#[test]
fn test_body_claude_to_copilot_numbered_args() {
    let body = "First: $1, Second: $2, Third: $3";
    assert_eq!(
        body_claude_to_copilot(body),
        "First: ${arg1}, Second: ${arg2}, Third: ${arg3}"
    );
}

#[test]
fn test_body_claude_to_copilot_all_args() {
    let body = "$1 $2 $3 $4 $5 $6 $7 $8 $9";
    let expected = "${arg1} ${arg2} ${arg3} ${arg4} ${arg5} ${arg6} ${arg7} ${arg8} ${arg9}";
    assert_eq!(body_claude_to_copilot(body), expected);
}

#[test]
fn test_body_claude_to_copilot_no_variables() {
    let body = "Just plain text";
    assert_eq!(body_claude_to_copilot(body), "Just plain text");
}

#[test]
fn test_body_copilot_to_claude_arguments() {
    let body = "Process ${arguments}";
    assert_eq!(body_copilot_to_claude(body), "Process $ARGUMENTS");
}

#[test]
fn test_body_copilot_to_claude_numbered_args() {
    let body = "First: ${arg1}, Second: ${arg2}, Third: ${arg3}";
    assert_eq!(
        body_copilot_to_claude(body),
        "First: $1, Second: $2, Third: $3"
    );
}

#[test]
fn test_body_copilot_to_claude_all_args() {
    let body = "${arg1} ${arg2} ${arg3} ${arg4} ${arg5} ${arg6} ${arg7} ${arg8} ${arg9}";
    let expected = "$1 $2 $3 $4 $5 $6 $7 $8 $9";
    assert_eq!(body_copilot_to_claude(body), expected);
}

// ============================================================================
// Allowed tools parsing tests
// ============================================================================

#[test]
fn test_parse_allowed_tools() {
    let tools = "Read, Write, Bash";
    let result = parse_allowed_tools(tools);
    assert_eq!(result, vec!["Read", "Write", "Bash"]);
}

#[test]
fn test_parse_allowed_tools_with_patterns() {
    let tools = "Bash(git:*), Read, Edit";
    let result = parse_allowed_tools(tools);
    assert_eq!(result, vec!["Bash(git:*)", "Read", "Edit"]);
}

#[test]
fn test_parse_allowed_tools_empty() {
    let tools = "";
    let result = parse_allowed_tools(tools);
    assert!(result.is_empty());
}

#[test]
fn test_parse_allowed_tools_whitespace_only() {
    let tools = "  ,  , ";
    let result = parse_allowed_tools(tools);
    assert!(result.is_empty());
}

#[test]
fn test_format_allowed_tools() {
    let tools = vec!["Read".to_string(), "Write".to_string(), "Bash".to_string()];
    assert_eq!(format_allowed_tools(&tools), "Read, Write, Bash");
}

#[test]
fn test_format_allowed_tools_empty() {
    let tools: Vec<String> = vec![];
    assert_eq!(format_allowed_tools(&tools), "");
}

// ============================================================================
// YAML escape tests
// ============================================================================

#[test]
fn test_escape_yaml_string_plain() {
    assert_eq!(escape_yaml_string("simple text"), "simple text");
}

#[test]
fn test_escape_yaml_string_with_colon() {
    assert_eq!(escape_yaml_string("key: value"), "\"key: value\"");
}

#[test]
fn test_escape_yaml_string_with_quotes() {
    assert_eq!(escape_yaml_string("say \"hello\""), "\"say \\\"hello\\\"\"");
}

#[test]
fn test_escape_yaml_string_with_newline() {
    assert_eq!(escape_yaml_string("line1\nline2"), "\"line1\\nline2\"");
}

#[test]
fn test_escape_yaml_string_with_hash() {
    assert_eq!(escape_yaml_string("text # comment"), "\"text # comment\"");
}

#[test]
fn test_escape_yaml_string_with_leading_space() {
    assert_eq!(escape_yaml_string(" leading"), "\" leading\"");
}

#[test]
fn test_escape_yaml_string_with_trailing_space() {
    assert_eq!(escape_yaml_string("trailing "), "\"trailing \"");
}

#[test]
fn test_escape_yaml_string_with_backslash() {
    // Backslash alone doesn't need quoting in YAML
    assert_eq!(escape_yaml_string("path\\to\\file"), "path\\to\\file");
}

#[test]
fn test_escape_yaml_string_backslash_with_special_char() {
    // When quoting is needed for other reasons, backslash is also escaped
    assert_eq!(escape_yaml_string("path\\to: file"), "\"path\\\\to: file\"");
}

// ============================================================================
// to_format conversion tests
// ============================================================================

#[test]
fn to_format_claude_code_to_copilot() {
    let cmd = ClaudeCodeCommand {
        name: Some("commit".to_string()),
        description: Some("Create commit".to_string()),
        allowed_tools: Some("Read, Write, Bash".to_string()),
        argument_hint: Some("[message]".to_string()),
        model: Some("haiku".to_string()),
        disable_model_invocation: Some(false),
        user_invocable: Some(true),
        body: "Commit with $ARGUMENTS".to_string(),
    };

    let md = cmd.to_format(TargetType::Copilot).unwrap().to_markdown();

    assert!(md.contains("name: commit"));
    assert!(md.contains("description: Create commit"));
    assert!(md.contains("tools: ['codebase', 'terminal']"));
    assert!(md.contains("hint: Enter message"));
    assert!(md.contains("model: GPT-4o-mini"));
    assert!(md.contains("Commit with ${arguments}"));
}

#[test]
fn to_format_claude_code_to_codex() {
    let cmd = ClaudeCodeCommand {
        name: Some("deploy".to_string()),
        description: Some("Deploy app".to_string()),
        allowed_tools: Some("Bash".to_string()),
        argument_hint: Some("[env]".to_string()),
        model: Some("sonnet".to_string()),
        disable_model_invocation: None,
        user_invocable: None,
        body: "Deploy to $1".to_string(),
    };

    let md = cmd.to_format(TargetType::Codex).unwrap().to_markdown();

    assert!(md.contains("description: Deploy app"));
    // Codex doesn't include name in frontmatter
    assert!(!md.contains("name:"));
    assert!(md.contains("Deploy to $1"));
}

#[test]
fn to_format_tools_deduplication() {
    // Multiple Claude Code tools mapping to same Copilot tool
    let cmd = ClaudeCodeCommand {
        name: Some("test".to_string()),
        description: None,
        allowed_tools: Some("Read, Write, Edit".to_string()),
        argument_hint: None,
        model: None,
        disable_model_invocation: None,
        user_invocable: None,
        body: "Body".to_string(),
    };

    let md = cmd.to_format(TargetType::Copilot).unwrap().to_markdown();

    // All three map to "codebase" and should be deduplicated
    assert!(md.contains("tools: ['codebase']"));
}
