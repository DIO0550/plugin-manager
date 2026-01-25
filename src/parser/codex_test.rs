//! Tests for Codex Prompt parser.

use super::codex::CodexPrompt;
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
