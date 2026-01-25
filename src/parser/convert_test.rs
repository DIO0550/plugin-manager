//! Tests for convert module.

use super::claude_code::ClaudeCodeCommand;
use super::codex::CodexPrompt;
use super::convert::*;
use super::copilot::CopilotPrompt;

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
// From trait tests
// ============================================================================

#[test]
fn from_claude_code_to_copilot() {
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

    let copilot = CopilotPrompt::from(&cmd);

    assert_eq!(copilot.name, Some("commit".to_string()));
    assert_eq!(copilot.description, Some("Create commit".to_string()));
    assert_eq!(
        copilot.tools,
        Some(vec!["codebase".to_string(), "terminal".to_string()])
    );
    assert_eq!(copilot.hint, Some("Enter message".to_string()));
    assert_eq!(copilot.model, Some("GPT-4o-mini".to_string()));
    assert_eq!(copilot.agent, None);
    assert_eq!(copilot.body, "Commit with ${arguments}");
}

#[test]
fn from_claude_code_to_codex() {
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

    let codex = CodexPrompt::from(&cmd);

    assert_eq!(codex.name, Some("deploy".to_string()));
    assert_eq!(codex.description, Some("Deploy app".to_string()));
    assert_eq!(codex.body, "Deploy to $1");
}

#[test]
fn from_copilot_to_claude_code() {
    let prompt = CopilotPrompt {
        name: Some("review".to_string()),
        description: Some("Review code".to_string()),
        tools: Some(vec!["codebase".to_string(), "terminal".to_string()]),
        hint: Some("Enter file path".to_string()),
        model: Some("GPT-4o".to_string()),
        agent: Some("code-reviewer".to_string()),
        body: "Review ${arg1} with ${arguments}".to_string(),
    };

    let cmd = ClaudeCodeCommand::from(&prompt);

    assert_eq!(cmd.name, Some("review".to_string()));
    assert_eq!(cmd.description, Some("Review code".to_string()));
    assert_eq!(cmd.allowed_tools, Some("Read, Bash".to_string()));
    assert_eq!(cmd.argument_hint, Some("[file path]".to_string()));
    assert_eq!(cmd.model, Some("sonnet".to_string()));
    assert_eq!(cmd.body, "Review $1 with $ARGUMENTS");
}

#[test]
fn from_copilot_to_codex() {
    let prompt = CopilotPrompt {
        name: Some("build".to_string()),
        description: Some("Build project".to_string()),
        tools: Some(vec!["terminal".to_string()]),
        hint: None,
        model: Some("o1".to_string()),
        agent: None,
        body: "Run cargo build".to_string(),
    };

    let codex = CodexPrompt::from(&prompt);

    assert_eq!(codex.name, Some("build".to_string()));
    assert_eq!(codex.description, Some("Build project".to_string()));
    assert_eq!(codex.body, "Run cargo build");
}

#[test]
fn from_codex_to_claude_code() {
    let codex = CodexPrompt {
        name: Some("test".to_string()),
        description: Some("Run tests".to_string()),
        body: "cargo test".to_string(),
    };

    let cmd = ClaudeCodeCommand::from(&codex);

    assert_eq!(cmd.name, Some("test".to_string()));
    assert_eq!(cmd.description, Some("Run tests".to_string()));
    assert_eq!(cmd.allowed_tools, None);
    assert_eq!(cmd.model, None);
    assert_eq!(cmd.body, "cargo test");
}

#[test]
fn from_codex_to_copilot() {
    let codex = CodexPrompt {
        name: Some("lint".to_string()),
        description: Some("Run linter".to_string()),
        body: "cargo clippy".to_string(),
    };

    let prompt = CopilotPrompt::from(&codex);

    assert_eq!(prompt.name, Some("lint".to_string()));
    assert_eq!(prompt.description, Some("Run linter".to_string()));
    assert_eq!(prompt.tools, None);
    assert_eq!(prompt.body, "cargo clippy");
}

// ============================================================================
// Into trait tests (auto-implemented from From)
// ============================================================================

#[test]
fn into_copilot_from_claude_code() {
    let cmd = ClaudeCodeCommand {
        name: Some("test".to_string()),
        description: None,
        allowed_tools: None,
        argument_hint: None,
        model: None,
        disable_model_invocation: None,
        user_invocable: None,
        body: "Test body".to_string(),
    };

    let copilot: CopilotPrompt = (&cmd).into();

    assert_eq!(copilot.name, Some("test".to_string()));
    assert_eq!(copilot.body, "Test body");
}

// ============================================================================
// Round-trip tests
// ============================================================================

#[test]
fn roundtrip_claude_code_to_copilot_to_claude_code() {
    let original = ClaudeCodeCommand {
        name: Some("commit".to_string()),
        description: Some("Create commit".to_string()),
        allowed_tools: Some("Bash".to_string()),
        argument_hint: Some("[message]".to_string()),
        model: Some("haiku".to_string()),
        disable_model_invocation: None, // Lost in conversion
        user_invocable: None,           // Lost in conversion
        body: "Commit $ARGUMENTS".to_string(),
    };

    let copilot = CopilotPrompt::from(&original);
    let restored = ClaudeCodeCommand::from(&copilot);

    // Restorable fields
    assert_eq!(restored.name, original.name);
    assert_eq!(restored.description, original.description);
    assert_eq!(restored.allowed_tools, original.allowed_tools);
    assert_eq!(restored.argument_hint, original.argument_hint);
    assert_eq!(restored.model, original.model);
    assert_eq!(restored.body, original.body);

    // Lost fields
    assert_eq!(restored.disable_model_invocation, None);
    assert_eq!(restored.user_invocable, None);
}

#[test]
fn roundtrip_claude_code_to_codex_to_claude_code() {
    let original = ClaudeCodeCommand {
        name: Some("build".to_string()),
        description: Some("Build project".to_string()),
        allowed_tools: Some("Bash".to_string()),     // Lost
        argument_hint: Some("[target]".to_string()), // Lost
        model: Some("sonnet".to_string()),           // Lost
        disable_model_invocation: None,
        user_invocable: None,
        body: "Build it".to_string(),
    };

    let codex = CodexPrompt::from(&original);
    let restored = ClaudeCodeCommand::from(&codex);

    // Restorable fields
    assert_eq!(restored.name, original.name);
    assert_eq!(restored.description, original.description);
    assert_eq!(restored.body, original.body);

    // Lost fields (Codex doesn't support these)
    assert_eq!(restored.allowed_tools, None);
    assert_eq!(restored.argument_hint, None);
    assert_eq!(restored.model, None);
}

#[test]
fn roundtrip_copilot_to_claude_code_to_copilot() {
    let original = CopilotPrompt {
        name: Some("review".to_string()),
        description: Some("Review code".to_string()),
        tools: Some(vec!["terminal".to_string()]),
        hint: Some("Enter file".to_string()),
        model: Some("GPT-4o".to_string()),
        agent: Some("reviewer".to_string()), // Lost
        body: "Review ${arguments}".to_string(),
    };

    let claude = ClaudeCodeCommand::from(&original);
    let restored = CopilotPrompt::from(&claude);

    // Restorable fields
    assert_eq!(restored.name, original.name);
    assert_eq!(restored.description, original.description);
    assert_eq!(restored.tools, original.tools);
    assert_eq!(restored.hint, original.hint);
    assert_eq!(restored.model, original.model);
    assert_eq!(restored.body, original.body);

    // Lost fields
    assert_eq!(restored.agent, None);
}

#[test]
fn roundtrip_tools_lose_granularity() {
    // Multiple Claude Code tools mapping to same Copilot tool
    let original = ClaudeCodeCommand {
        name: Some("test".to_string()),
        description: None,
        allowed_tools: Some("Read, Write, Edit".to_string()),
        argument_hint: None,
        model: None,
        disable_model_invocation: None,
        user_invocable: None,
        body: "Body".to_string(),
    };

    let copilot = CopilotPrompt::from(&original);

    // All three map to "codebase"
    assert_eq!(copilot.tools, Some(vec!["codebase".to_string()]));

    let restored = ClaudeCodeCommand::from(&copilot);

    // Only "Read" is restored (representative value)
    assert_eq!(restored.allowed_tools, Some("Read".to_string()));
}

#[test]
fn roundtrip_codex_to_copilot_to_codex() {
    let original = CodexPrompt {
        name: Some("deploy".to_string()),
        description: Some("Deploy app".to_string()),
        body: "Run deploy".to_string(),
    };

    let copilot = CopilotPrompt::from(&original);
    let restored = CodexPrompt::from(&copilot);

    assert_eq!(restored.name, original.name);
    assert_eq!(restored.description, original.description);
    // Body may have variable conversion artifacts but simple text is preserved
    assert_eq!(restored.body, original.body);
}
