//! Claude Code Agent parser.
//!
//! Parses `.claude/agents/<name>.md` files.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use super::codex_agent::CodexAgent;
use super::convert::{self, TargetFormat, TargetType};
use super::copilot_agent::CopilotAgent;
use super::frontmatter::{parse_frontmatter, ParsedDocument};

/// Claude Code Agent frontmatter fields.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ClaudeCodeAgentFrontmatter {
    /// Agent identifier (defaults to filename if not specified).
    #[serde(default)]
    pub name: Option<String>,

    /// Agent description.
    #[serde(default)]
    pub description: Option<String>,

    /// Available tools (comma-separated string).
    #[serde(default)]
    pub tools: Option<String>,

    /// Model to use (haiku, sonnet, opus).
    #[serde(default)]
    pub model: Option<String>,
}

/// Parsed Claude Code Agent.
#[derive(Debug, Clone)]
pub struct ClaudeCodeAgent {
    /// Agent name.
    pub name: Option<String>,
    /// Agent description.
    pub description: Option<String>,
    /// Available tools (comma-separated string).
    pub tools: Option<String>,
    /// Model to use.
    pub model: Option<String>,
    /// Agent body (system prompt).
    pub body: String,
}

impl ClaudeCodeAgent {
    /// Parses a Claude Code Agent from content string.
    ///
    /// The name field is taken directly from frontmatter (no filename fallback).
    pub fn parse(content: &str) -> Result<Self> {
        let ParsedDocument { frontmatter, body } =
            parse_frontmatter::<ClaudeCodeAgentFrontmatter>(content)?;

        let fm = frontmatter.unwrap_or_default();

        Ok(ClaudeCodeAgent {
            name: normalize_name(fm.name),
            description: fm.description,
            tools: fm.tools,
            model: fm.model,
            body,
        })
    }

    /// Loads and parses a Claude Code Agent from a file.
    ///
    /// If the frontmatter doesn't specify a name, the filename is used as fallback.
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let mut agent = Self::parse(&content)?;

        // Fallback to filename if name is not specified
        if agent.name.is_none() {
            agent.name = extract_name_from_path(path);
        }

        Ok(agent)
    }

    /// Serializes to Claude Code Markdown format.
    pub fn to_markdown(&self) -> String {
        let mut fields: Vec<String> = Vec::new();

        if let Some(ref v) = self.name {
            fields.push(format!("name: {}", convert::escape_yaml_string(v)));
        }
        if let Some(ref v) = self.description {
            fields.push(format!("description: {}", convert::escape_yaml_string(v)));
        }
        if let Some(ref v) = self.tools {
            fields.push(format!("tools: {}", convert::escape_yaml_string(v)));
        }
        if let Some(ref v) = self.model {
            fields.push(format!("model: {}", v));
        }

        if fields.is_empty() {
            self.body.clone()
        } else {
            format!("---\n{}\n---\n\n{}", fields.join("\n"), self.body)
        }
    }

    /// Converts to the specified target format.
    ///
    /// Returns a boxed trait object implementing `TargetFormat`.
    pub fn to_format(&self, target: TargetType) -> Result<Box<dyn TargetFormat>> {
        match target {
            TargetType::Copilot => Ok(Box::new(self.to_copilot())),
            TargetType::Codex => Ok(Box::new(self.to_codex())),
        }
    }

    /// Converts to Copilot Agent format (internal).
    fn to_copilot(&self) -> CopilotAgent {
        // Tool conversion: comma-separated string -> array -> convert -> deduplicate
        let tools = self
            .tools
            .as_ref()
            .map(|t| convert::tools_claude_to_copilot(&convert::parse_allowed_tools(t)))
            .filter(|t| !t.is_empty());

        CopilotAgent {
            name: self.name.clone(),
            description: self.description.clone(),
            tools,
            model: self
                .model
                .as_ref()
                .map(|m| convert::model_claude_to_copilot(m)),
            target: Some("vscode".to_string()),
            handoffs: None,
            body: self.body.clone(), // Agent body is NOT converted (no variable replacement)
        }
    }

    /// Converts to Codex Agent format (internal).
    fn to_codex(&self) -> CodexAgent {
        CodexAgent {
            name: self.name.clone(),
            description: self.description.clone(),
            body: self.body.clone(),
        }
    }
}

/// Normalizes name: empty or whitespace-only string becomes None.
fn normalize_name(name: Option<String>) -> Option<String> {
    name.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// Extracts agent name from file path.
///
/// Removes the `.md` extension from the filename.
fn extract_name_from_path(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.strip_suffix(".md").unwrap_or(s).to_string())
        .filter(|s| !s.is_empty())
}
