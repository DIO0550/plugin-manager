use super::codex_agent::CodexAgent;
use super::convert::TargetFormat;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn parse_with_description() {
    let content = "---\ndescription: Expert code reviewer\n---\n\nYou are a code review expert.";
    let agent = CodexAgent::parse(content).unwrap();
    assert_eq!(agent.description.as_deref(), Some("Expert code reviewer"));
    assert_eq!(agent.body, "\nYou are a code review expert.");
    assert!(agent.name.is_none());
}

#[test]
fn parse_no_frontmatter() {
    let content = "You are a code review expert.";
    let agent = CodexAgent::parse(content).unwrap();
    assert!(agent.description.is_none());
    assert!(agent.name.is_none());
    assert_eq!(agent.body, "You are a code review expert.");
}

#[test]
fn parse_empty_frontmatter() {
    let content = "---\n---\n\nBody only.";
    let agent = CodexAgent::parse(content).unwrap();
    assert!(agent.description.is_none());
    assert_eq!(agent.body, "\nBody only.");
}

#[test]
fn parse_unclosed_frontmatter() {
    let content = "---\ndescription: test\nBody text here.";
    let agent = CodexAgent::parse(content).unwrap();
    // Treated as no frontmatter
    assert!(agent.description.is_none());
    assert_eq!(agent.body, content);
}

#[test]
fn parse_unknown_fields_ignored() {
    let content = "---\ndescription: test\nunknown_field: value\n---\n\nBody.";
    let agent = CodexAgent::parse(content).unwrap();
    assert_eq!(agent.description.as_deref(), Some("test"));
    assert_eq!(agent.body, "\nBody.");
}

#[test]
fn load_uses_filename_as_name() {
    let mut file = NamedTempFile::with_suffix(".agent.md").unwrap();
    write!(file, "---\ndescription: test\n---\n\nBody.").unwrap();
    let agent = CodexAgent::load(file.path()).unwrap();
    assert!(agent.name.is_some());
    assert_eq!(agent.description.as_deref(), Some("test"));
}

#[test]
fn load_strips_agent_md_extension() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("reviewer.agent.md");
    std::fs::write(&path, "Body only.").unwrap();
    let agent = CodexAgent::load(&path).unwrap();
    assert_eq!(agent.name.as_deref(), Some("reviewer"));
}

#[test]
fn load_strips_md_extension() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("reviewer.md");
    std::fs::write(&path, "Body only.").unwrap();
    let agent = CodexAgent::load(&path).unwrap();
    assert_eq!(agent.name.as_deref(), Some("reviewer"));
}

#[test]
fn to_markdown_with_description() {
    let agent = CodexAgent {
        name: Some("test".to_string()),
        description: Some("A test agent".to_string()),
        body: "Body content.".to_string(),
    };
    let md = agent.to_markdown();
    assert_eq!(md, "---\ndescription: A test agent\n---\n\nBody content.");
}

#[test]
fn to_markdown_body_only() {
    let agent = CodexAgent {
        name: None,
        description: None,
        body: "Body only.".to_string(),
    };
    assert_eq!(agent.to_markdown(), "Body only.");
}

#[test]
fn to_markdown_special_characters() {
    let agent = CodexAgent {
        name: None,
        description: Some("has: colon".to_string()),
        body: "Body.".to_string(),
    };
    let md = agent.to_markdown();
    assert!(md.contains("\"has: colon\""));
}
