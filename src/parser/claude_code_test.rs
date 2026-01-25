//! Tests for Claude Code Command parser.

use super::claude_code::ClaudeCodeCommand;
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
