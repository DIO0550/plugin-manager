use super::claude_code_agent::ClaudeCodeAgent;
use super::convert::TargetType;
use super::copilot_agent::CopilotAgent;
use std::io::Write;
use tempfile::NamedTempFile;

// ==================== Parse Tests ====================

#[test]
fn parse_full_frontmatter() {
    let content = r#"---
name: code-reviewer
description: Expert code review specialist
tools: Read, Grep, Glob, Bash
model: opus
---

You are a code review expert."#;
    let agent = ClaudeCodeAgent::parse(content).unwrap();
    assert_eq!(agent.name.as_deref(), Some("code-reviewer"));
    assert_eq!(
        agent.description.as_deref(),
        Some("Expert code review specialist")
    );
    assert_eq!(agent.tools.as_deref(), Some("Read, Grep, Glob, Bash"));
    assert_eq!(agent.model.as_deref(), Some("opus"));
    assert_eq!(agent.body, "\nYou are a code review expert.");
}

#[test]
fn parse_minimal_frontmatter() {
    let content = "---\nname: simple\n---\n\nBody.";
    let agent = ClaudeCodeAgent::parse(content).unwrap();
    assert_eq!(agent.name.as_deref(), Some("simple"));
    assert!(agent.description.is_none());
    assert!(agent.tools.is_none());
    assert!(agent.model.is_none());
}

#[test]
fn parse_no_frontmatter() {
    let content = "Body only.";
    let agent = ClaudeCodeAgent::parse(content).unwrap();
    assert!(agent.name.is_none());
    assert!(agent.description.is_none());
    assert_eq!(agent.body, "Body only.");
}

#[test]
fn parse_empty_frontmatter() {
    let content = "---\n---\n\nBody.";
    let agent = ClaudeCodeAgent::parse(content).unwrap();
    assert!(agent.name.is_none());
    assert_eq!(agent.body, "\nBody.");
}

#[test]
fn parse_unclosed_frontmatter() {
    let content = "---\nname: test\nBody text.";
    let agent = ClaudeCodeAgent::parse(content).unwrap();
    // Treated as no frontmatter
    assert!(agent.name.is_none());
    assert_eq!(agent.body, content);
}

#[test]
fn parse_yaml_error() {
    let content = "---\nname: [invalid yaml\n---\n\nBody.";
    let result = ClaudeCodeAgent::parse(content);
    assert!(result.is_err());
}

#[test]
fn parse_unknown_fields_ignored() {
    let content = "---\nname: test\nunknown_field: value\n---\n\nBody.";
    let agent = ClaudeCodeAgent::parse(content).unwrap();
    assert_eq!(agent.name.as_deref(), Some("test"));
}

#[test]
fn parse_utf8_bom() {
    let content = "\u{feff}---\nname: test\n---\n\nBody.";
    let agent = ClaudeCodeAgent::parse(content).unwrap();
    assert_eq!(agent.name.as_deref(), Some("test"));
}

// ==================== Name Normalization Tests ====================

#[test]
fn parse_name_empty_string() {
    let content = "---\nname: \"\"\n---\n\nBody.";
    let agent = ClaudeCodeAgent::parse(content).unwrap();
    assert!(agent.name.is_none());
}

#[test]
fn parse_name_whitespace_only() {
    let content = "---\nname: \"  \"\n---\n\nBody.";
    let agent = ClaudeCodeAgent::parse(content).unwrap();
    assert!(agent.name.is_none());
}

#[test]
fn parse_name_trimmed() {
    let content = "---\nname: \"  trimmed  \"\n---\n\nBody.";
    let agent = ClaudeCodeAgent::parse(content).unwrap();
    assert_eq!(agent.name.as_deref(), Some("trimmed"));
}

// ==================== File Load Tests ====================

#[test]
fn load_uses_filename_fallback() {
    let mut file = NamedTempFile::with_suffix(".md").unwrap();
    write!(file, "---\ndescription: test\n---\n\nBody.").unwrap();
    let agent = ClaudeCodeAgent::load(file.path()).unwrap();
    assert!(agent.name.is_some());
}

#[test]
fn load_keeps_frontmatter_name() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("file.md");
    std::fs::write(&path, "---\nname: explicit-name\n---\n\nBody.").unwrap();
    let agent = ClaudeCodeAgent::load(&path).unwrap();
    assert_eq!(agent.name.as_deref(), Some("explicit-name"));
}

#[test]
fn load_strips_md_extension() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("reviewer.md");
    std::fs::write(&path, "Body.").unwrap();
    let agent = ClaudeCodeAgent::load(&path).unwrap();
    assert_eq!(agent.name.as_deref(), Some("reviewer"));
}

// ==================== Serialization Tests ====================

#[test]
fn to_markdown_full() {
    let agent = ClaudeCodeAgent {
        name: Some("test".to_string()),
        description: Some("A test agent".to_string()),
        tools: Some("Read, Write".to_string()),
        model: Some("opus".to_string()),
        body: "Body.".to_string(),
    };
    let md = agent.to_markdown();
    assert!(md.contains("name: test"));
    assert!(md.contains("description: A test agent"));
    assert!(md.contains("tools: Read, Write"));
    assert!(md.contains("model: opus"));
    assert!(md.ends_with("Body."));
}

#[test]
fn to_markdown_body_only() {
    let agent = ClaudeCodeAgent {
        name: None,
        description: None,
        tools: None,
        model: None,
        body: "Body only.".to_string(),
    };
    assert_eq!(agent.to_markdown(), "Body only.");
}

#[test]
fn to_markdown_special_characters() {
    let agent = ClaudeCodeAgent {
        name: Some("has: colon".to_string()),
        description: Some("has \"quotes\"".to_string()),
        tools: None,
        model: None,
        body: "Body.".to_string(),
    };
    let md = agent.to_markdown();
    assert!(md.contains("\"has: colon\""));
    assert!(md.contains("\\\""));
}

// ==================== Conversion Tests: Copilot ====================

#[test]
fn to_copilot_basic() {
    let agent = ClaudeCodeAgent {
        name: Some("test".to_string()),
        description: Some("A test agent".to_string()),
        tools: Some("Read, Grep, Bash".to_string()),
        model: Some("opus".to_string()),
        body: "Body.".to_string(),
    };
    let target = agent.to_format(TargetType::Copilot).unwrap();
    let md = target.to_markdown();

    assert!(md.contains("name: test"));
    assert!(md.contains("description: A test agent"));
    // Tools should be converted
    assert!(md.contains("codebase"));
    assert!(md.contains("search/codebase"));
    assert!(md.contains("terminal"));
    // Model should be converted
    assert!(md.contains("o1")); // opus -> o1
                                // Target should be set
    assert!(md.contains("target: vscode"));
}

#[test]
fn to_copilot_tool_deduplication() {
    let agent = ClaudeCodeAgent {
        name: Some("test".to_string()),
        description: None,
        tools: Some("Read, Write, Edit".to_string()), // All map to "codebase"
        model: None,
        body: "Body.".to_string(),
    };
    let target = agent.to_format(TargetType::Copilot).unwrap();
    let md = target.to_markdown();

    // Should only have one "codebase" after deduplication
    let count = md.matches("codebase").count();
    assert_eq!(count, 1);
}

#[test]
fn to_copilot_model_conversion() {
    let test_cases = vec![
        ("haiku", "GPT-4o-mini"),
        ("sonnet", "GPT-4o"),
        ("opus", "o1"),
        ("HAIKU", "GPT-4o-mini"), // Case insensitive
    ];

    for (input, expected) in test_cases {
        let agent = ClaudeCodeAgent {
            name: None,
            description: None,
            tools: None,
            model: Some(input.to_string()),
            body: "Body.".to_string(),
        };
        let target = agent.to_format(TargetType::Copilot).unwrap();
        let md = target.to_markdown();
        assert!(
            md.contains(&format!("model: {}", expected)),
            "Expected {} for input {}",
            expected,
            input
        );
    }
}

#[test]
fn to_copilot_unknown_model() {
    let agent = ClaudeCodeAgent {
        name: None,
        description: None,
        tools: None,
        model: Some("claude-sonnet-4-5-20250929".to_string()),
        body: "Body.".to_string(),
    };
    let target = agent.to_format(TargetType::Copilot).unwrap();
    let md = target.to_markdown();
    // Unknown model should be passed through
    assert!(md.contains("model: claude-sonnet-4-5-20250929"));
}

#[test]
fn to_copilot_no_tools() {
    let agent = ClaudeCodeAgent {
        name: Some("test".to_string()),
        description: None,
        tools: None,
        model: None,
        body: "Body.".to_string(),
    };
    let target = agent.to_format(TargetType::Copilot).unwrap();
    let md = target.to_markdown();
    assert!(!md.contains("tools:"));
}

#[test]
fn to_copilot_empty_tools() {
    let agent = ClaudeCodeAgent {
        name: Some("test".to_string()),
        description: None,
        tools: Some("".to_string()),
        model: None,
        body: "Body.".to_string(),
    };
    let target = agent.to_format(TargetType::Copilot).unwrap();
    let md = target.to_markdown();
    // Empty tools should not produce tools field
    assert!(!md.contains("tools:"));
}

#[test]
fn to_copilot_whitespace_only_tools() {
    let agent = ClaudeCodeAgent {
        name: Some("test".to_string()),
        description: None,
        tools: Some("  ".to_string()),
        model: None,
        body: "Body.".to_string(),
    };
    let target = agent.to_format(TargetType::Copilot).unwrap();
    let md = target.to_markdown();
    assert!(!md.contains("tools:"));
}

#[test]
fn to_copilot_trailing_comma_tools() {
    let agent = ClaudeCodeAgent {
        name: None,
        description: None,
        tools: Some("Read, Write,".to_string()),
        model: None,
        body: "Body.".to_string(),
    };
    let target = agent.to_format(TargetType::Copilot).unwrap();
    let md = target.to_markdown();
    // Should parse correctly, empty element ignored
    assert!(md.contains("codebase"));
}

#[test]
fn to_copilot_body_not_converted() {
    let agent = ClaudeCodeAgent {
        name: None,
        description: None,
        tools: None,
        model: None,
        body: "Body with $ARGUMENTS.".to_string(),
    };
    let target = agent.to_format(TargetType::Copilot).unwrap();
    let md = target.to_markdown();
    // Agent body should NOT have variables converted (unlike Command)
    assert!(md.contains("$ARGUMENTS"));
}

#[test]
fn to_copilot_no_handoffs() {
    let agent = ClaudeCodeAgent {
        name: Some("test".to_string()),
        description: None,
        tools: None,
        model: None,
        body: "Body.".to_string(),
    };
    let target = agent.to_format(TargetType::Copilot).unwrap();
    let md = target.to_markdown();
    // Claude Code Agent doesn't have handoffs, Copilot shouldn't either
    assert!(!md.contains("handoffs:"));
}

// ==================== Conversion Tests: Codex ====================

#[test]
fn to_codex_basic() {
    let agent = ClaudeCodeAgent {
        name: Some("test".to_string()),
        description: Some("A test agent".to_string()),
        tools: Some("Read, Grep".to_string()),
        model: Some("opus".to_string()),
        body: "Body.".to_string(),
    };
    let target = agent.to_format(TargetType::Codex).unwrap();
    let md = target.to_markdown();

    assert!(md.contains("description: A test agent"));
    assert!(md.ends_with("Body."));
    // Codex should NOT have tools or model
    assert!(!md.contains("tools:"));
    assert!(!md.contains("model:"));
    // Codex should NOT have name in frontmatter
    assert!(!md.contains("name:"));
}

#[test]
fn to_codex_no_description() {
    let agent = ClaudeCodeAgent {
        name: Some("test".to_string()),
        description: None,
        tools: None,
        model: None,
        body: "Body only.".to_string(),
    };
    let target = agent.to_format(TargetType::Codex).unwrap();
    let md = target.to_markdown();
    // No frontmatter when only body
    assert_eq!(md, "Body only.");
}

#[test]
fn to_codex_preserves_body() {
    let agent = ClaudeCodeAgent {
        name: None,
        description: None,
        tools: None,
        model: None,
        body: "Body with $ARGUMENTS.".to_string(),
    };
    let target = agent.to_format(TargetType::Codex).unwrap();
    let md = target.to_markdown();
    // Body should be unchanged
    assert!(md.contains("$ARGUMENTS"));
}
