//! Tests for frontmatter parser.

use super::frontmatter::parse_frontmatter;
use serde::Deserialize;

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
