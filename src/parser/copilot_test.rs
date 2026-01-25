//! Tests for Copilot Prompt parser.

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
