use super::convert::TargetFormat;
use super::copilot_agent::{CopilotAgent, CopilotAgentHandoff};
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn parse_full_frontmatter() {
    let content = r#"---
name: code-reviewer
description: Expert code review specialist
tools: ['codebase', 'terminal']
model: GPT-4o
target: vscode
handoffs:
  - agent: fixer
    label: "Fix issues"
    prompt: "Fix the issues found"
    send: true
---

You are a code review expert."#;
    let agent = CopilotAgent::parse(content).unwrap();
    assert_eq!(agent.name.as_deref(), Some("code-reviewer"));
    assert_eq!(
        agent.description.as_deref(),
        Some("Expert code review specialist")
    );
    assert_eq!(
        agent.tools.as_ref().unwrap(),
        &vec!["codebase".to_string(), "terminal".to_string()]
    );
    assert_eq!(agent.model.as_deref(), Some("GPT-4o"));
    assert_eq!(agent.target.as_deref(), Some("vscode"));
    assert!(agent.handoffs.is_some());
    let handoffs = agent.handoffs.unwrap();
    assert_eq!(handoffs.len(), 1);
    assert_eq!(handoffs[0].agent.as_deref(), Some("fixer"));
    assert_eq!(handoffs[0].label.as_deref(), Some("Fix issues"));
    assert_eq!(handoffs[0].prompt.as_deref(), Some("Fix the issues found"));
    assert_eq!(handoffs[0].send, Some(true));
}

#[test]
fn parse_minimal_frontmatter() {
    let content = "---\nname: simple\n---\n\nBody.";
    let agent = CopilotAgent::parse(content).unwrap();
    assert_eq!(agent.name.as_deref(), Some("simple"));
    assert!(agent.tools.is_none());
    assert!(agent.handoffs.is_none());
}

#[test]
fn parse_no_frontmatter() {
    let content = "Body only.";
    let agent = CopilotAgent::parse(content).unwrap();
    assert!(agent.name.is_none());
    assert_eq!(agent.body, "Body only.");
}

#[test]
fn parse_empty_frontmatter() {
    let content = "---\n---\n\nBody.";
    let agent = CopilotAgent::parse(content).unwrap();
    assert!(agent.name.is_none());
    assert_eq!(agent.body, "\nBody.");
}

#[test]
fn parse_name_empty_string() {
    let content = "---\nname: \"\"\n---\n\nBody.";
    let agent = CopilotAgent::parse(content).unwrap();
    assert!(agent.name.is_none());
}

#[test]
fn parse_name_whitespace_only() {
    let content = "---\nname: \"  \"\n---\n\nBody.";
    let agent = CopilotAgent::parse(content).unwrap();
    assert!(agent.name.is_none());
}

#[test]
fn load_uses_filename_fallback() {
    let mut file = NamedTempFile::with_suffix(".agent.md").unwrap();
    write!(file, "---\ndescription: test\n---\n\nBody.").unwrap();
    let agent = CopilotAgent::load(file.path()).unwrap();
    assert!(agent.name.is_some());
}

#[test]
fn load_strips_agent_md_extension() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("reviewer.agent.md");
    std::fs::write(&path, "Body.").unwrap();
    let agent = CopilotAgent::load(&path).unwrap();
    assert_eq!(agent.name.as_deref(), Some("reviewer"));
}

#[test]
fn load_strips_md_extension() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("reviewer.md");
    std::fs::write(&path, "Body.").unwrap();
    let agent = CopilotAgent::load(&path).unwrap();
    assert_eq!(agent.name.as_deref(), Some("reviewer"));
}

#[test]
fn load_keeps_frontmatter_name() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("file.agent.md");
    std::fs::write(&path, "---\nname: explicit-name\n---\n\nBody.").unwrap();
    let agent = CopilotAgent::load(&path).unwrap();
    assert_eq!(agent.name.as_deref(), Some("explicit-name"));
}

#[test]
fn to_markdown_full() {
    let agent = CopilotAgent {
        name: Some("test".to_string()),
        description: Some("A test agent".to_string()),
        tools: Some(vec!["codebase".to_string(), "terminal".to_string()]),
        model: Some("GPT-4o".to_string()),
        target: Some("vscode".to_string()),
        handoffs: Some(vec![CopilotAgentHandoff {
            agent: Some("fixer".to_string()),
            label: Some("Fix it".to_string()),
            prompt: Some("Fix issues".to_string()),
            send: Some(true),
        }]),
        body: "Body.".to_string(),
    };
    let md = agent.to_markdown();
    assert!(md.contains("name: test"));
    assert!(md.contains("description: A test agent"));
    assert!(md.contains("tools: ['codebase', 'terminal']"));
    assert!(md.contains("model: GPT-4o"));
    assert!(md.contains("target: vscode"));
    assert!(md.contains("handoffs:"));
    assert!(md.contains("agent: fixer"));
}

#[test]
fn to_markdown_body_only() {
    let agent = CopilotAgent {
        name: None,
        description: None,
        tools: None,
        model: None,
        target: None,
        handoffs: None,
        body: "Body only.".to_string(),
    };
    assert_eq!(agent.to_markdown(), "Body only.");
}

#[test]
fn to_markdown_empty_tools() {
    let agent = CopilotAgent {
        name: Some("test".to_string()),
        description: None,
        tools: Some(vec![]),
        model: None,
        target: None,
        handoffs: None,
        body: "Body.".to_string(),
    };
    let md = agent.to_markdown();
    // Empty tools should not be serialized
    assert!(!md.contains("tools:"));
}

#[test]
fn to_markdown_empty_handoffs() {
    let agent = CopilotAgent {
        name: Some("test".to_string()),
        description: None,
        tools: None,
        model: None,
        target: None,
        handoffs: Some(vec![]),
        body: "Body.".to_string(),
    };
    let md = agent.to_markdown();
    // Empty handoffs should not be serialized
    assert!(!md.contains("handoffs:"));
}

#[test]
fn to_markdown_special_characters_in_name() {
    let agent = CopilotAgent {
        name: Some("has: colon".to_string()),
        description: None,
        tools: None,
        model: None,
        target: None,
        handoffs: None,
        body: "Body.".to_string(),
    };
    let md = agent.to_markdown();
    assert!(md.contains("\"has: colon\""));
}

#[test]
fn to_markdown_single_quote_in_tools() {
    let agent = CopilotAgent {
        name: None,
        description: None,
        tools: Some(vec!["it's".to_string()]),
        model: None,
        target: None,
        handoffs: None,
        body: "Body.".to_string(),
    };
    let md = agent.to_markdown();
    // Single quote should be escaped in YAML
    assert!(md.contains("'it''s'"));
}

#[test]
fn to_markdown_handoff_special_characters() {
    let agent = CopilotAgent {
        name: None,
        description: None,
        tools: None,
        model: None,
        target: None,
        handoffs: Some(vec![CopilotAgentHandoff {
            agent: Some("test".to_string()),
            label: Some("label: with colon".to_string()),
            prompt: Some("prompt\nwith newline".to_string()),
            send: None,
        }]),
        body: "Body.".to_string(),
    };
    let md = agent.to_markdown();
    // Special characters should be escaped
    assert!(md.contains("\"label: with colon\""));
    assert!(md.contains("\\n"));
}
