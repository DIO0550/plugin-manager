//! Tests for convert module.

use super::claude_code::ClaudeCodeCommand;
use super::convert::*;

// ============================================================================
// Tool name conversion tests
// ============================================================================

#[test]
fn test_tool_claude_to_copilot_file_operations() {
    assert_eq!(tool_claude_to_copilot("Read"), "codebase");
    assert_eq!(tool_claude_to_copilot("Write"), "codebase");
    assert_eq!(tool_claude_to_copilot("Edit"), "codebase");
}

#[test]
fn test_tool_claude_to_copilot_search_operations() {
    assert_eq!(tool_claude_to_copilot("Grep"), "search/codebase");
    assert_eq!(tool_claude_to_copilot("Glob"), "search/codebase");
}

#[test]
fn test_tool_claude_to_copilot_bash() {
    assert_eq!(tool_claude_to_copilot("Bash"), "terminal");
    assert_eq!(tool_claude_to_copilot("Bash(git:*)"), "githubRepo");
    assert_eq!(tool_claude_to_copilot("Bash(git commit*)"), "githubRepo");
}

#[test]
fn test_tool_claude_to_copilot_web() {
    assert_eq!(tool_claude_to_copilot("WebFetch"), "fetch");
    assert_eq!(tool_claude_to_copilot("WebSearch"), "websearch");
}

#[test]
fn test_tool_claude_to_copilot_unknown() {
    assert_eq!(tool_claude_to_copilot("UnknownTool"), "UnknownTool");
    assert_eq!(tool_claude_to_copilot("CustomMCP"), "CustomMCP");
}

#[test]
fn test_tool_claude_to_copilot_with_whitespace() {
    assert_eq!(tool_claude_to_copilot(" Read "), "codebase");
    assert_eq!(tool_claude_to_copilot("  Bash  "), "terminal");
}

#[test]
fn test_tool_copilot_to_claude() {
    assert_eq!(tool_copilot_to_claude("codebase"), "Read");
    assert_eq!(tool_copilot_to_claude("search/codebase"), "Grep");
    assert_eq!(tool_copilot_to_claude("terminal"), "Bash");
    assert_eq!(tool_copilot_to_claude("githubRepo"), "Bash");
    assert_eq!(tool_copilot_to_claude("fetch"), "WebFetch");
    assert_eq!(tool_copilot_to_claude("websearch"), "WebSearch");
}

#[test]
fn test_tool_copilot_to_claude_unknown() {
    assert_eq!(tool_copilot_to_claude("unknownTool"), "unknownTool");
}

#[test]
fn test_tools_claude_to_copilot_deduplication() {
    let tools = vec![
        "Read".to_string(),
        "Write".to_string(),
        "Edit".to_string(),
        "Bash".to_string(),
    ];
    let result = tools_claude_to_copilot(&tools);
    // Read, Write, Edit all map to "codebase", so should be deduplicated
    assert_eq!(result.len(), 2);
    assert!(result.contains(&"codebase".to_string()));
    assert!(result.contains(&"terminal".to_string()));
}

#[test]
fn test_tools_claude_to_copilot_sorted() {
    let tools = vec!["WebSearch".to_string(), "Read".to_string()];
    let result = tools_claude_to_copilot(&tools);
    // Should be sorted alphabetically
    assert_eq!(result, vec!["codebase", "websearch"]);
}

// ============================================================================
// Model name conversion tests
// ============================================================================

#[test]
fn test_model_claude_to_copilot() {
    assert_eq!(model_claude_to_copilot("haiku"), "GPT-4o-mini");
    assert_eq!(model_claude_to_copilot("sonnet"), "GPT-4o");
    assert_eq!(model_claude_to_copilot("opus"), "o1");
}

#[test]
fn test_model_claude_to_copilot_case_insensitive() {
    assert_eq!(model_claude_to_copilot("Haiku"), "GPT-4o-mini");
    assert_eq!(model_claude_to_copilot("SONNET"), "GPT-4o");
    assert_eq!(model_claude_to_copilot("OPUS"), "o1");
}

#[test]
fn test_model_claude_to_copilot_unknown() {
    assert_eq!(model_claude_to_copilot("custom-model"), "custom-model");
}

#[test]
fn test_model_copilot_to_claude() {
    assert_eq!(model_copilot_to_claude("GPT-4o-mini"), "haiku");
    assert_eq!(model_copilot_to_claude("GPT-4o"), "sonnet");
    assert_eq!(model_copilot_to_claude("o1"), "opus");
}

#[test]
fn test_model_copilot_to_claude_case_insensitive() {
    assert_eq!(model_copilot_to_claude("gpt-4o-mini"), "haiku");
    assert_eq!(model_copilot_to_claude("gpt-4o"), "sonnet");
    assert_eq!(model_copilot_to_claude("O1"), "opus");
}

#[test]
fn test_model_copilot_to_claude_unknown() {
    assert_eq!(model_copilot_to_claude("custom-model"), "custom-model");
}

#[test]
fn test_model_claude_to_codex() {
    assert_eq!(model_claude_to_codex("haiku"), "gpt-4.1-mini");
    assert_eq!(model_claude_to_codex("sonnet"), "gpt-4.1");
    assert_eq!(model_claude_to_codex("opus"), "o3");
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
