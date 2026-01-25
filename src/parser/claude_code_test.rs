//! Tests for Claude Code Command parser.

use super::claude_code::ClaudeCodeCommand;
use super::codex::CodexPrompt;
use super::copilot::CopilotPrompt;
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
// Conversion tests (using From trait)
// ============================================================================

#[test]
fn from_trait_to_copilot_full_command() {
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

    let copilot = CopilotPrompt::from(&cmd);

    assert_eq!(copilot.name, Some("commit".to_string()));
    assert_eq!(copilot.description, Some("Create a commit".to_string()));
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
fn from_trait_to_copilot_minimal_command() {
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

    let copilot = CopilotPrompt::from(&cmd);

    assert_eq!(copilot.name, None);
    assert_eq!(copilot.tools, None);
    assert_eq!(copilot.body, "Simple body");
}

#[test]
fn from_trait_to_copilot_tools_deduplication() {
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

    let copilot = CopilotPrompt::from(&cmd);

    // Read, Write, Edit all map to "codebase"
    assert_eq!(copilot.tools, Some(vec!["codebase".to_string()]));
}

#[test]
fn from_trait_to_codex() {
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

    let codex = CodexPrompt::from(&cmd);

    assert_eq!(codex.name, Some("deploy".to_string()));
    assert_eq!(codex.description, Some("Deploy the app".to_string()));
    // Codex doesn't support variables, so body is unchanged
    assert_eq!(codex.body, "Deploy to $1");
}

#[test]
fn from_trait_from_copilot() {
    let copilot = CopilotPrompt {
        name: Some("review".to_string()),
        description: Some("Review code".to_string()),
        tools: Some(vec!["codebase".to_string(), "terminal".to_string()]),
        hint: Some("Enter file path".to_string()),
        model: Some("GPT-4o".to_string()),
        agent: Some("code-reviewer".to_string()), // This should be ignored
        body: "Review ${arg1} with ${arguments}".to_string(),
    };

    let cmd = ClaudeCodeCommand::from(&copilot);

    assert_eq!(cmd.name, Some("review".to_string()));
    assert_eq!(cmd.description, Some("Review code".to_string()));
    assert_eq!(cmd.allowed_tools, Some("Read, Bash".to_string()));
    assert_eq!(cmd.argument_hint, Some("[file path]".to_string()));
    assert_eq!(cmd.model, Some("sonnet".to_string()));
    assert_eq!(cmd.disable_model_invocation, None);
    assert_eq!(cmd.user_invocable, None);
    assert_eq!(cmd.body, "Review $1 with $ARGUMENTS");
}

#[test]
fn from_trait_from_copilot_hint_without_enter_prefix() {
    let copilot = CopilotPrompt {
        name: Some("test".to_string()),
        description: None,
        tools: None,
        hint: Some("file path".to_string()), // No "Enter " prefix
        model: None,
        agent: None,
        body: "Body".to_string(),
    };

    let cmd = ClaudeCodeCommand::from(&copilot);

    // Should wrap in brackets
    assert_eq!(cmd.argument_hint, Some("[file path]".to_string()));
}

#[test]
fn from_trait_from_codex() {
    let codex = CodexPrompt {
        name: Some("build".to_string()),
        description: Some("Build the project".to_string()),
        body: "Run cargo build".to_string(),
    };

    let cmd = ClaudeCodeCommand::from(&codex);

    assert_eq!(cmd.name, Some("build".to_string()));
    assert_eq!(cmd.description, Some("Build the project".to_string()));
    assert_eq!(cmd.allowed_tools, None);
    assert_eq!(cmd.argument_hint, None);
    assert_eq!(cmd.model, None);
    assert_eq!(cmd.body, "Run cargo build");
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
