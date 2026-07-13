//! Tests for frontmatter parser.

use super::frontmatter::{
    emit_frontmatter, normalize_optional_name, parse_frontmatter, stem_without_suffixes,
    yaml_single_quoted_array,
};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
struct TestFrontmatter {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

#[test]
fn parse_basic_frontmatter() {
    let content = r#"---
name: test
description: A test description
---

This is the body."#;

    let result = parse_frontmatter::<TestFrontmatter>(content).unwrap();

    assert!(result.frontmatter.is_some());
    let fm = result.frontmatter.unwrap();
    assert_eq!(fm.name, Some("test".to_string()));
    assert_eq!(fm.description, Some("A test description".to_string()));
    assert_eq!(result.body, "\nThis is the body.");
}

#[test]
fn parse_no_frontmatter() {
    let content = "This is just body text without frontmatter.";

    let result = parse_frontmatter::<TestFrontmatter>(content).unwrap();

    assert!(result.frontmatter.is_none());
    assert_eq!(result.body, content);
}

#[test]
fn parse_empty_frontmatter() {
    let content = r#"---
---

Body after empty frontmatter."#;

    let result = parse_frontmatter::<TestFrontmatter>(content).unwrap();

    assert!(result.frontmatter.is_some());
    let fm = result.frontmatter.unwrap();
    assert_eq!(fm, TestFrontmatter::default());
    assert_eq!(result.body, "\nBody after empty frontmatter.");
}

#[test]
fn parse_frontmatter_no_closing() {
    let content = r#"---
name: test
This has no closing marker"#;

    let result = parse_frontmatter::<TestFrontmatter>(content).unwrap();

    // No closing --- means no valid frontmatter
    assert!(result.frontmatter.is_none());
    assert_eq!(result.body, content);
}

#[test]
fn parse_with_bom() {
    let content = "\u{feff}---
name: with-bom
---

Body text.";

    let result = parse_frontmatter::<TestFrontmatter>(content).unwrap();

    assert!(result.frontmatter.is_some());
    let fm = result.frontmatter.unwrap();
    assert_eq!(fm.name, Some("with-bom".to_string()));
}

#[test]
fn parse_frontmatter_with_trailing_whitespace() {
    let content = "---   \nname: test\n---   \n\nBody.";

    let result = parse_frontmatter::<TestFrontmatter>(content).unwrap();

    assert!(result.frontmatter.is_some());
    assert_eq!(result.frontmatter.unwrap().name, Some("test".to_string()));
}

#[test]
fn parse_body_preserves_leading_newlines() {
    let content = r#"---
name: test
---


Multiple leading newlines in body."#;

    let result = parse_frontmatter::<TestFrontmatter>(content).unwrap();

    assert!(result.body.starts_with("\n\n"));
}

#[test]
fn parse_frontmatter_partial_fields() {
    let content = r#"---
name: only-name
---

Body."#;

    let result = parse_frontmatter::<TestFrontmatter>(content).unwrap();

    assert!(result.frontmatter.is_some());
    let fm = result.frontmatter.unwrap();
    assert_eq!(fm.name, Some("only-name".to_string()));
    assert_eq!(fm.description, None);
}

#[test]
fn parse_empty_content() {
    let content = "";

    let result = parse_frontmatter::<TestFrontmatter>(content).unwrap();

    assert!(result.frontmatter.is_none());
    assert_eq!(result.body, "");
}

#[test]
fn parse_only_opening_marker() {
    let content = "---";

    let result = parse_frontmatter::<TestFrontmatter>(content).unwrap();

    // Single --- with no closing is treated as no frontmatter
    assert!(result.frontmatter.is_none());
    assert_eq!(result.body, "---");
}

#[test]
fn parse_invalid_yaml_returns_error() {
    let content = r#"---
name: [invalid yaml
---

Body."#;

    let result = parse_frontmatter::<TestFrontmatter>(content);
    assert!(result.is_err());
}

#[test]
fn parse_frontmatter_with_body_containing_dashes() {
    let content = r#"---
name: test
---

Body with --- dashes in text."#;

    let result = parse_frontmatter::<TestFrontmatter>(content).unwrap();

    assert!(result.frontmatter.is_some());
    assert!(result.body.contains("--- dashes"));
}

#[test]
fn stem_without_suffixes_prefers_longer_agent_suffix() {
    let path = Path::new("/tmp/code-reviewer.agent.md");
    assert_eq!(
        stem_without_suffixes(path, &[".agent.md", ".md"]),
        Some("code-reviewer".to_string())
    );
}

#[test]
fn stem_without_suffixes_prefers_longer_prompt_suffix() {
    let path = Path::new("/tmp/commit.prompt.md");
    assert_eq!(
        stem_without_suffixes(path, &[".prompt.md", ".md"]),
        Some("commit".to_string())
    );
}

#[test]
fn stem_without_suffixes_falls_back_to_md() {
    let path = Path::new("/tmp/helper.md");
    assert_eq!(
        stem_without_suffixes(path, &[".agent.md", ".md"]),
        Some("helper".to_string())
    );
}

#[test]
fn stem_without_suffixes_rejects_empty_stem() {
    let path = Path::new("/tmp/.md");
    assert_eq!(stem_without_suffixes(path, &[".md"]), None);
}

#[test]
fn normalize_optional_name_trims_and_filters_empty() {
    assert_eq!(
        normalize_optional_name(Some("  name  ".to_string())),
        Some("name".to_string())
    );
    assert_eq!(normalize_optional_name(Some("".to_string())), None);
    assert_eq!(normalize_optional_name(Some("   ".to_string())), None);
    assert_eq!(normalize_optional_name(None), None);
}

#[test]
fn emit_frontmatter_returns_body_when_fields_empty() {
    assert_eq!(emit_frontmatter(&[], "Body only."), "Body only.");
}

#[test]
fn emit_frontmatter_wraps_fields() {
    let fields = vec!["name: test".to_string(), "model: opus".to_string()];
    assert_eq!(
        emit_frontmatter(&fields, "Body"),
        "---\nname: test\nmodel: opus\n---\n\nBody"
    );
}

#[test]
fn yaml_single_quoted_array_escapes_quotes() {
    let items = vec!["tool'with'quotes".to_string(), "codebase".to_string()];
    assert_eq!(
        yaml_single_quoted_array(&items),
        "['tool''with''quotes', 'codebase']"
    );
}

#[test]
fn yaml_single_quoted_array_empty() {
    assert_eq!(yaml_single_quoted_array(&[]), "[]");
}
