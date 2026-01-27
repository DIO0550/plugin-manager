//! Claude Code Command parser.
//!
//! Parses `.claude/commands/<name>.md` files.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use super::codex::CodexPrompt;
use super::convert::{self, TargetFormat, TargetType};
use super::copilot::CopilotPrompt;
use super::frontmatter::{parse_frontmatter, ParsedDocument};

/// Claude Code Command frontmatter fields.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ClaudeCodeCommandFrontmatter {
    /// Command identifier (defaults to filename if not specified).
    #[serde(default)]
    pub name: Option<String>,

    /// Command description.
    #[serde(default)]
    pub description: Option<String>,

    /// Allowed tools (comma-separated string, e.g., "Bash(git:*), Read, Write").
    #[serde(default)]
    pub allowed_tools: Option<String>,

    /// Argument hint (e.g., "[message]").
    #[serde(default)]
    pub argument_hint: Option<String>,

    /// Model to use (haiku, sonnet, opus).
    #[serde(default)]
    pub model: Option<String>,

    /// Disable automatic invocation by model.
    #[serde(default)]
    pub disable_model_invocation: Option<bool>,

    /// Whether user can invoke this command.
    #[serde(default)]
    pub user_invocable: Option<bool>,
}

/// Parsed Claude Code Command.
#[derive(Debug, Clone)]
pub struct ClaudeCodeCommand {
    /// Command name.
    pub name: Option<String>,
    /// Command description.
    pub description: Option<String>,
    /// Allowed tools (comma-separated string).
    pub allowed_tools: Option<String>,
    /// Argument hint.
    pub argument_hint: Option<String>,
    /// Model to use.
    pub model: Option<String>,
    /// Disable model invocation flag.
    pub disable_model_invocation: Option<bool>,
    /// User invocable flag.
    pub user_invocable: Option<bool>,
    /// Command body (prompt template).
    pub body: String,
}

impl ClaudeCodeCommand {
    /// Parses a Claude Code Command from content string.
    ///
    /// The name field is taken directly from frontmatter (no filename fallback).
    pub fn parse(content: &str) -> Result<Self> {
        let ParsedDocument { frontmatter, body } =
            parse_frontmatter::<ClaudeCodeCommandFrontmatter>(content)?;

        let fm = frontmatter.unwrap_or_default();

        Ok(ClaudeCodeCommand {
            name: normalize_name(fm.name),
            description: fm.description,
            allowed_tools: fm.allowed_tools,
            argument_hint: fm.argument_hint,
            model: fm.model,
            disable_model_invocation: fm.disable_model_invocation,
            user_invocable: fm.user_invocable,
            body,
        })
    }

    /// Loads and parses a Claude Code Command from a file.
    ///
    /// If the frontmatter doesn't specify a name, the filename is used as fallback.
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let mut command = Self::parse(&content)?;

        // Fallback to filename if name is not specified
        if command.name.is_none() {
            command.name = extract_name_from_path(path);
        }

        Ok(command)
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
        if let Some(ref v) = self.allowed_tools {
            fields.push(format!("allowed-tools: {}", convert::escape_yaml_string(v)));
        }
        if let Some(ref v) = self.argument_hint {
            fields.push(format!("argument-hint: {}", convert::escape_yaml_string(v)));
        }
        if let Some(ref v) = self.model {
            fields.push(format!("model: {}", v));
        }
        if let Some(v) = self.disable_model_invocation {
            fields.push(format!("disable-model-invocation: {}", v));
        }
        if let Some(v) = self.user_invocable {
            fields.push(format!("user-invocable: {}", v));
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

    /// Converts to Copilot format (internal).
    fn to_copilot(&self) -> CopilotPrompt {
        // Tool conversion: comma-separated string -> array -> convert -> deduplicate
        let tools = self
            .allowed_tools
            .as_ref()
            .map(|t| convert::tools_claude_to_copilot(&convert::parse_allowed_tools(t)));

        // Hint conversion: [message] -> "Enter message"
        let hint = self.argument_hint.as_ref().map(|h| {
            let inner = h.trim_start_matches('[').trim_end_matches(']');
            format!("Enter {}", inner)
        });

        CopilotPrompt {
            name: self.name.clone(),
            description: self.description.clone(),
            tools,
            hint,
            model: self
                .model
                .as_ref()
                .map(|m| convert::model_claude_to_copilot(m)),
            agent: None,
            body: convert::body_claude_to_copilot(&self.body),
        }
    }

    /// Converts to Codex format (internal).
    fn to_codex(&self) -> CodexPrompt {
        CodexPrompt {
            name: self.name.clone(),
            description: self.description.clone(),
            body: self.body.clone(), // Codex doesn't support variables
        }
    }
}

/// Normalizes name: empty or whitespace-only string becomes None.
fn normalize_name(name: Option<String>) -> Option<String> {
    name.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// Extracts command name from file path.
///
/// Removes the `.md` extension from the filename.
fn extract_name_from_path(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.strip_suffix(".md").unwrap_or(s).to_string())
        .filter(|s| !s.is_empty())
}
