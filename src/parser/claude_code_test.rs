//! Tests for Claude Code Command parser.

use super::claude_code::ClaudeCodeCommand;
use super::convert::TargetType;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn parse_full_command() {
    let content = r#"---
name: commit-helper
description: Create a git commit with conventional message
allowed-tools: Bash(git add:*), Bash(git commit:*)
argument-hint: "[message]"
model: haiku
disable-model-invocation: false
user-invocable: true
---

Commit the staged changes with the message: $ARGUMENTS"#;

    let cmd = ClaudeCodeCommand::parse(content).unwrap();

    assert_eq!(cmd.name, Some("commit-helper".to_string()));
    assert_eq!(
        cmd.description,
        Some("Create a git commit with conventional message".to_string())
    );
    assert_eq!(
        cmd.allowed_tools,
        Some("Bash(git add:*), Bash(git commit:*)".to_string())
    );
    assert_eq!(cmd.argument_hint, Some("[message]".to_string()));
    assert_eq!(cmd.model, Some("haiku".to_string()));
    assert_eq!(cmd.disable_model_invocation, Some(false));
    assert_eq!(cmd.user_invocable, Some(true));
    assert!(cmd.body.contains("$ARGUMENTS"));
}

#[test]
fn parse_minimal_command() {
    let content = r#"---
description: A minimal command
---

Do something."#;

    let cmd = ClaudeCodeCommand::parse(content).unwrap();

    assert_eq!(cmd.name, None);
    assert_eq!(cmd.description, Some("A minimal command".to_string()));
    assert_eq!(cmd.allowed_tools, None);
    assert_eq!(cmd.model, None);
}

#[test]
fn parse_command_no_frontmatter() {
    let content = "Just a prompt without any frontmatter.";

    let cmd = ClaudeCodeCommand::parse(content).unwrap();

    assert_eq!(cmd.name, None);
    assert_eq!(cmd.description, None);
    assert_eq!(cmd.body, content);
}

#[test]
fn parse_command_empty_name_becomes_none() {
    let content = r#"---
name: ""
description: Empty name test
---

Body."#;

    let cmd = ClaudeCodeCommand::parse(content).unwrap();

    assert_eq!(cmd.name, None);
    assert_eq!(cmd.description, Some("Empty name test".to_string()));
}

#[test]
fn parse_command_whitespace_only_name_becomes_none() {
    let content = r#"---
name: "   "
description: Whitespace name test
---

Body."#;

    let cmd = ClaudeCodeCommand::parse(content).unwrap();

    assert_eq!(cmd.name, None);
    assert_eq!(cmd.description, Some("Whitespace name test".to_string()));
}

#[test]
fn parse_command_name_trimmed() {
    let content = r#"---
name: "  test  "
description: Trimmed name test
---

Body."#;

    let cmd = ClaudeCodeCommand::parse(content).unwrap();

    assert_eq!(cmd.name, Some("test".to_string()));
}

#[test]
fn parse_command_preserves_body_newlines() {
    let content = r#"---
description: test
---


Multiple leading newlines."#;

    let cmd = ClaudeCodeCommand::parse(content).unwrap();

    assert!(cmd.body.starts_with("\n\n"));
}

#[test]
fn load_command_uses_filename_fallback() {
    let content = r#"---
description: No name in frontmatter
---

Body."#;

    let mut file = NamedTempFile::with_suffix(".md").unwrap();
    file.write_all(content.as_bytes()).unwrap();

    let cmd = ClaudeCodeCommand::load(file.path()).unwrap();

    // Should use filename (without .md) as name
    assert!(cmd.name.is_some());
}

#[test]
fn load_command_prefers_frontmatter_name() {
    let content = r#"---
name: frontmatter-name
description: Has name in frontmatter
---

Body."#;

    let mut file = NamedTempFile::with_suffix(".md").unwrap();
    file.write_all(content.as_bytes()).unwrap();

    let cmd = ClaudeCodeCommand::load(file.path()).unwrap();

    assert_eq!(cmd.name, Some("frontmatter-name".to_string()));
}

#[test]
fn parse_command_with_bom() {
    let content = "\u{feff}---
name: bom-test
---

Body.";

    let cmd = ClaudeCodeCommand::parse(content).unwrap();

    assert_eq!(cmd.name, Some("bom-test".to_string()));
}

#[test]
fn parse_command_unknown_fields_ignored() {
    let content = r#"---
name: test
unknown-field: should be ignored
another-unknown: also ignored
---

Body."#;

    let cmd = ClaudeCodeCommand::parse(content).unwrap();

    assert_eq!(cmd.name, Some("test".to_string()));
}

// ============================================================================
// to_markdown tests
// ============================================================================

#[test]
fn to_markdown_full_command() {
    let cmd = ClaudeCodeCommand {
        name: Some("commit".to_string()),
        description: Some("Create a commit".to_string()),
        allowed_tools: Some("Bash(git:*)".to_string()),
        argument_hint: Some("[message]".to_string()),
        model: Some("haiku".to_string()),
        disable_model_invocation: Some(false),
        user_invocable: Some(true),
        body: "Commit body".to_string(),
    };

    let md = cmd.to_markdown();

    assert!(md.contains("---\n"));
    assert!(md.contains("name: commit"));
    assert!(md.contains("description: Create a commit")); // No quotes needed
    assert!(md.contains("allowed-tools: \"Bash(git:*)\"")); // Has colon
    assert!(md.contains("argument-hint: [message]")); // Brackets don't need quotes
    assert!(md.contains("model: haiku"));
    assert!(md.contains("disable-model-invocation: false"));
    assert!(md.contains("user-invocable: true"));
    assert!(md.ends_with("Commit body"));
}

#[test]
fn to_markdown_minimal_command() {
    let cmd = ClaudeCodeCommand {
        name: None,
        description: None,
        allowed_tools: None,
        argument_hint: None,
        model: None,
        disable_model_invocation: None,
        user_invocable: None,
        body: "Just body".to_string(),
    };

    let md = cmd.to_markdown();

    // No frontmatter, just body
    assert_eq!(md, "Just body");
}

#[test]
fn to_markdown_escapes_special_chars() {
    let cmd = ClaudeCodeCommand {
        name: Some("test".to_string()),
        description: Some("A: B".to_string()), // Contains colon
        allowed_tools: None,
        argument_hint: None,
        model: None,
        disable_model_invocation: None,
        user_invocable: None,
        body: "Body".to_string(),
    };

    let md = cmd.to_markdown();

    // Description with colon should be quoted
    assert!(md.contains("description: \"A: B\""));
}

// ============================================================================
// to_format tests
// ============================================================================

#[test]
fn to_format_copilot_converts_correctly() {
    let cmd = ClaudeCodeCommand {
        name: Some("commit".to_string()),
        description: Some("Create a commit".to_string()),
        allowed_tools: Some("Read, Write, Bash".to_string()),
        argument_hint: Some("[message]".to_string()),
        model: Some("haiku".to_string()),
        disable_model_invocation: Some(false),
        user_invocable: Some(true),
        body: "Commit with $ARGUMENTS".to_string(),
    };

    let target = cmd.to_format(TargetType::Copilot).unwrap();
    let md = target.to_markdown();

    // Copilot format features
    assert!(md.contains("name: commit"));
    assert!(md.contains("tools:"));
    assert!(md.contains("${arguments}"));
    assert!(md.contains("GPT-4o-mini"));
}

#[test]
fn to_format_codex_converts_correctly() {
    let cmd = ClaudeCodeCommand {
        name: Some("deploy".to_string()),
        description: Some("Deploy the app".to_string()),
        allowed_tools: Some("Bash".to_string()),
        argument_hint: Some("[env]".to_string()),
        model: Some("sonnet".to_string()),
        disable_model_invocation: None,
        user_invocable: None,
        body: "Deploy to $1".to_string(),
    };

    let target = cmd.to_format(TargetType::Codex).unwrap();
    let md = target.to_markdown();

    // Codex format features
    assert!(md.contains("description: Deploy the app"));
    // Codex doesn't include name in frontmatter
    assert!(!md.contains("name:"));
    // Codex doesn't support variables
    assert!(md.contains("Deploy to $1"));
}

#[test]
fn to_format_minimal_command() {
    let cmd = ClaudeCodeCommand {
        name: None,
        description: None,
        allowed_tools: None,
        argument_hint: None,
        model: None,
        disable_model_invocation: None,
        user_invocable: None,
        body: "Simple body".to_string(),
    };

    let copilot = cmd.to_format(TargetType::Copilot).unwrap();
    assert_eq!(copilot.to_markdown(), "Simple body");

    let codex = cmd.to_format(TargetType::Codex).unwrap();
    assert_eq!(codex.to_markdown(), "Simple body");
}

#[test]
fn to_format_copilot_deduplicates_tools() {
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

    // Read, Write, Edit all map to "codebase" and should be deduplicated
    assert!(md.contains("tools: ['codebase']"));
}
