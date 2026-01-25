//! Tests for Codex Prompt parser.

use super::claude_code::ClaudeCodeCommand;
use super::codex::CodexPrompt;
use super::copilot::CopilotPrompt;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn parse_basic() {
    let content = r#"---
description: Create a git commit with conventional message
---

Commit the staged changes with the provided message."#;

    let prompt = CodexPrompt::parse(content).unwrap();

    assert_eq!(prompt.name, None); // Codex doesn't have name field
    assert_eq!(
        prompt.description,
        Some("Create a git commit with conventional message".to_string())
    );
    assert!(prompt.body.contains("Commit the staged changes"));
}

#[test]
fn parse_no_frontmatter() {
    let content = "Just a prompt without frontmatter.";

    let prompt = CodexPrompt::parse(content).unwrap();

    assert_eq!(prompt.name, None);
    assert_eq!(prompt.description, None);
    assert_eq!(prompt.body, content);
}

#[test]
fn parse_empty_frontmatter() {
    let content = r#"---
---

Body after empty frontmatter."#;

    let prompt = CodexPrompt::parse(content).unwrap();

    assert_eq!(prompt.name, None);
    assert_eq!(prompt.description, None);
    assert!(prompt.body.contains("Body after empty frontmatter"));
}

#[test]
fn parse_with_bom() {
    let content = "\u{feff}---
description: bom-test
---

Body.";

    let prompt = CodexPrompt::parse(content).unwrap();

    assert_eq!(prompt.description, Some("bom-test".to_string()));
}

#[test]
fn parse_unknown_fields_ignored() {
    let content = r#"---
description: test
unknown-field: should be ignored
---

Body."#;

    let prompt = CodexPrompt::parse(content).unwrap();

    assert_eq!(prompt.description, Some("test".to_string()));
}

#[test]
fn parse_preserves_body_leading_newlines() {
    let content = r#"---
description: test
---


Multiple leading newlines."#;

    let prompt = CodexPrompt::parse(content).unwrap();

    assert!(prompt.body.starts_with("\n\n"));
}

#[test]
fn load_from_file() {
    let content = r#"---
description: Test loading from file
---

Body content."#;

    let mut file = NamedTempFile::with_suffix(".md").unwrap();
    file.write_all(content.as_bytes()).unwrap();

    let prompt = CodexPrompt::load(file.path()).unwrap();

    assert_eq!(
        prompt.description,
        Some("Test loading from file".to_string())
    );
    // Name should come from filename
    assert!(prompt.name.is_some());
}

#[test]
fn load_uses_filename_as_name() {
    let content = r#"---
description: Test
---

Body."#;

    let mut file = NamedTempFile::with_suffix(".md").unwrap();
    file.write_all(content.as_bytes()).unwrap();

    let prompt = CodexPrompt::load(file.path()).unwrap();

    // Name should be extracted from filename (without .md)
    if let Some(name) = &prompt.name {
        assert!(!name.ends_with(".md"));
    }
}

#[test]
fn extract_name_removes_md_suffix() {
    let content = r#"---
description: Test
---

Body."#;

    let mut file = NamedTempFile::with_suffix(".md").unwrap();
    file.write_all(content.as_bytes()).unwrap();

    let prompt = CodexPrompt::load(file.path()).unwrap();

    // Name should not contain ".md"
    if let Some(name) = &prompt.name {
        assert!(!name.ends_with(".md"));
    }
}

// ============================================================================
// Conversion tests (using From trait)
// ============================================================================

#[test]
fn from_trait_to_claude_code() {
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

#[test]
fn from_trait_to_copilot() {
    let codex = CodexPrompt {
        name: Some("test".to_string()),
        description: Some("Run tests".to_string()),
        body: "Execute tests".to_string(),
    };

    let prompt = CopilotPrompt::from(&codex);

    assert_eq!(prompt.name, Some("test".to_string()));
    assert_eq!(prompt.description, Some("Run tests".to_string()));
    assert_eq!(prompt.tools, None);
    assert_eq!(prompt.hint, None);
    assert_eq!(prompt.model, None);
    assert_eq!(prompt.body, "Execute tests");
}

#[test]
fn from_trait_from_claude_code() {
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
    // Codex doesn't support variables, so body is unchanged
    assert_eq!(codex.body, "Deploy to $1");
}

#[test]
fn from_trait_from_copilot() {
    let prompt = CopilotPrompt {
        name: Some("review".to_string()),
        description: Some("Review code".to_string()),
        tools: Some(vec!["codebase".to_string()]),
        hint: Some("Enter path".to_string()),
        model: Some("GPT-4o".to_string()),
        agent: None,
        body: "Review ${arguments}".to_string(),
    };

    let codex = CodexPrompt::from(&prompt);

    assert_eq!(codex.name, Some("review".to_string()));
    assert_eq!(codex.description, Some("Review code".to_string()));
    // Body is kept as-is (Codex doesn't support variables)
    assert_eq!(codex.body, "Review ${arguments}");
}

// ============================================================================
// to_markdown tests
// ============================================================================

#[test]
fn to_markdown_full() {
    let codex = CodexPrompt {
        name: Some("build".to_string()), // Name is not in frontmatter for Codex
        description: Some("Build project".to_string()),
        body: "Run build".to_string(),
    };

    let md = codex.to_markdown();

    assert!(md.contains("---\n"));
    assert!(md.contains("description: Build project"));
    // Codex doesn't include name in frontmatter
    assert!(!md.contains("name:"));
    assert!(md.ends_with("Run build"));
}

#[test]
fn to_markdown_minimal() {
    let codex = CodexPrompt {
        name: None,
        description: None,
        body: "Just body".to_string(),
    };

    let md = codex.to_markdown();

    assert_eq!(md, "Just body");
}

#[test]
fn to_markdown_escapes_special_chars() {
    let codex = CodexPrompt {
        name: None,
        description: Some("A: B".to_string()),
        body: "Body".to_string(),
    };

    let md = codex.to_markdown();

    assert!(md.contains("description: \"A: B\""));
}
