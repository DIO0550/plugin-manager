//! Tests for Copilot Prompt parser.

use super::claude_code::ClaudeCodeCommand;
use super::codex::CodexPrompt;
use super::copilot::CopilotPrompt;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn parse_full_frontmatter() {
    let content = r#"---
name: commit-helper
description: Create a git commit with conventional message
tools: ['githubRepo', 'codebase']
hint: "Enter commit message"
model: GPT-4o
agent: coding-agent
---

Create a commit with the message: ${message}"#;

    let prompt = CopilotPrompt::parse(content).unwrap();

    assert_eq!(prompt.name, Some("commit-helper".to_string()));
    assert_eq!(
        prompt.description,
        Some("Create a git commit with conventional message".to_string())
    );
    assert_eq!(
        prompt.tools,
        Some(vec!["githubRepo".to_string(), "codebase".to_string()])
    );
    assert_eq!(prompt.hint, Some("Enter commit message".to_string()));
    assert_eq!(prompt.model, Some("GPT-4o".to_string()));
    assert_eq!(prompt.agent, Some("coding-agent".to_string()));
    assert!(prompt.body.contains("${message}"));
}

#[test]
fn parse_no_frontmatter() {
    let content = "Just a prompt without frontmatter.";

    let prompt = CopilotPrompt::parse(content).unwrap();

    assert_eq!(prompt.name, None);
    assert_eq!(prompt.description, None);
    assert_eq!(prompt.body, content);
}

#[test]
fn parse_empty_name_becomes_none() {
    let content = r#"---
name: ""
description: Empty name test
---

Body."#;

    let prompt = CopilotPrompt::parse(content).unwrap();

    assert_eq!(prompt.name, None);
}

#[test]
fn parse_whitespace_only_name_becomes_none() {
    let content = r#"---
name: "   "
description: Whitespace name test
---

Body."#;

    let prompt = CopilotPrompt::parse(content).unwrap();

    assert_eq!(prompt.name, None);
}

#[test]
fn parse_name_trimmed() {
    let content = r#"---
name: "  test  "
description: Trimmed name test
---

Body."#;

    let prompt = CopilotPrompt::parse(content).unwrap();

    assert_eq!(prompt.name, Some("test".to_string()));
}

#[test]
fn parse_with_bom() {
    let content = "\u{feff}---
name: bom-test
---

Body.";

    let prompt = CopilotPrompt::parse(content).unwrap();

    assert_eq!(prompt.name, Some("bom-test".to_string()));
}

#[test]
fn parse_empty_frontmatter() {
    let content = r#"---
---

Body after empty frontmatter."#;

    let prompt = CopilotPrompt::parse(content).unwrap();

    assert_eq!(prompt.name, None);
    assert_eq!(prompt.description, None);
    assert!(prompt.body.contains("Body after empty frontmatter"));
}

#[test]
fn parse_unknown_fields_ignored() {
    let content = r#"---
name: test
unknown-field: should be ignored
---

Body."#;

    let prompt = CopilotPrompt::parse(content).unwrap();

    assert_eq!(prompt.name, Some("test".to_string()));
}

#[test]
fn parse_tools_array() {
    let content = r#"---
tools:
  - githubRepo
  - codebase
  - terminal
---

Body."#;

    let prompt = CopilotPrompt::parse(content).unwrap();

    assert_eq!(
        prompt.tools,
        Some(vec![
            "githubRepo".to_string(),
            "codebase".to_string(),
            "terminal".to_string()
        ])
    );
}

#[test]
fn parse_preserves_body_leading_newlines() {
    let content = r#"---
name: test
---


Multiple leading newlines."#;

    let prompt = CopilotPrompt::parse(content).unwrap();

    assert!(prompt.body.starts_with("\n\n"));
}

#[test]
fn load_from_file() {
    let content = r#"---
name: file-test
description: Test loading from file
---

Body content."#;

    let mut file = NamedTempFile::with_suffix(".prompt.md").unwrap();
    file.write_all(content.as_bytes()).unwrap();

    let prompt = CopilotPrompt::load(file.path()).unwrap();

    assert_eq!(prompt.name, Some("file-test".to_string()));
}

#[test]
fn load_uses_filename_fallback() {
    let content = r#"---
description: No name in frontmatter
---

Body."#;

    let mut file = NamedTempFile::with_suffix(".prompt.md").unwrap();
    file.write_all(content.as_bytes()).unwrap();

    let prompt = CopilotPrompt::load(file.path()).unwrap();

    // Should use filename (without .prompt.md) as name
    assert!(prompt.name.is_some());
}

#[test]
fn load_prefers_frontmatter_name() {
    let content = r#"---
name: frontmatter-name
---

Body."#;

    let mut file = NamedTempFile::with_suffix(".prompt.md").unwrap();
    file.write_all(content.as_bytes()).unwrap();

    let prompt = CopilotPrompt::load(file.path()).unwrap();

    assert_eq!(prompt.name, Some("frontmatter-name".to_string()));
}

#[test]
fn extract_name_removes_prompt_md_suffix() {
    let content = r#"---
description: No name
---

Body."#;

    // Create a temp file, then manually test path extraction
    let mut file = NamedTempFile::with_suffix(".prompt.md").unwrap();
    file.write_all(content.as_bytes()).unwrap();

    let prompt = CopilotPrompt::load(file.path()).unwrap();

    // Name should not contain ".prompt.md"
    if let Some(name) = &prompt.name {
        assert!(!name.ends_with(".prompt.md"));
        assert!(!name.ends_with(".prompt"));
    }
}

// ============================================================================
// Conversion tests (using From trait)
// ============================================================================

#[test]
fn from_trait_to_claude_code() {
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
fn from_trait_to_codex() {
    let prompt = CopilotPrompt {
        name: Some("build".to_string()),
        description: Some("Build project".to_string()),
        tools: Some(vec!["terminal".to_string()]),
        hint: None,
        model: Some("o1".to_string()),
        agent: None,
        body: "Run cargo build with ${arguments}".to_string(),
    };

    let codex = CodexPrompt::from(&prompt);

    assert_eq!(codex.name, Some("build".to_string()));
    assert_eq!(codex.description, Some("Build project".to_string()));
    // Codex doesn't support variables, so body is unchanged
    assert_eq!(codex.body, "Run cargo build with ${arguments}");
}

#[test]
fn from_trait_from_claude_code() {
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

    let prompt = CopilotPrompt::from(&cmd);

    assert_eq!(prompt.name, Some("commit".to_string()));
    assert_eq!(prompt.description, Some("Create a commit".to_string()));
    assert_eq!(
        prompt.tools,
        Some(vec!["codebase".to_string(), "terminal".to_string()])
    );
    assert_eq!(prompt.hint, Some("Enter message".to_string()));
    assert_eq!(prompt.model, Some("GPT-4o-mini".to_string()));
    assert_eq!(prompt.agent, None);
    assert_eq!(prompt.body, "Commit with ${arguments}");
}

#[test]
fn from_trait_from_codex() {
    let codex = CodexPrompt {
        name: Some("deploy".to_string()),
        description: Some("Deploy app".to_string()),
        body: "Run deployment".to_string(),
    };

    let prompt = CopilotPrompt::from(&codex);

    assert_eq!(prompt.name, Some("deploy".to_string()));
    assert_eq!(prompt.description, Some("Deploy app".to_string()));
    // Converted via ClaudeCodeCommand then to Copilot
    assert_eq!(prompt.body, "Run deployment");
}

// ============================================================================
// to_markdown tests
// ============================================================================

#[test]
fn to_markdown_full() {
    let prompt = CopilotPrompt {
        name: Some("review".to_string()),
        description: Some("Code review".to_string()),
        tools: Some(vec!["codebase".to_string(), "terminal".to_string()]),
        hint: Some("Enter file".to_string()),
        model: Some("GPT-4o".to_string()),
        agent: Some("reviewer".to_string()),
        body: "Review body".to_string(),
    };

    let md = prompt.to_markdown();

    assert!(md.contains("---\n"));
    assert!(md.contains("name: review"));
    assert!(md.contains("description: Code review"));
    assert!(md.contains("tools: ['codebase', 'terminal']"));
    assert!(md.contains("hint: Enter file"));
    assert!(md.contains("model: GPT-4o"));
    assert!(md.contains("agent: reviewer"));
    assert!(md.ends_with("Review body"));
}

#[test]
fn to_markdown_minimal() {
    let prompt = CopilotPrompt {
        name: None,
        description: None,
        tools: None,
        hint: None,
        model: None,
        agent: None,
        body: "Just body".to_string(),
    };

    let md = prompt.to_markdown();

    assert_eq!(md, "Just body");
}

#[test]
fn to_markdown_escapes_quotes_in_tools() {
    let prompt = CopilotPrompt {
        name: Some("test".to_string()),
        description: None,
        tools: Some(vec!["tool'with'quotes".to_string()]),
        hint: None,
        model: None,
        agent: None,
        body: "Body".to_string(),
    };

    let md = prompt.to_markdown();

    // Single quotes in tool names should be escaped
    assert!(md.contains("['tool''with''quotes']"));
}
