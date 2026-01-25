//! Copilot Prompt parser.
//!
//! Parses `.github/prompts/<name>.prompt.md` files.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use super::claude_code::ClaudeCodeCommand;
use super::codex::CodexPrompt;
use super::convert;
use super::frontmatter::{parse_frontmatter, ParsedDocument};

/// Copilot Prompt frontmatter fields.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CopilotPromptFrontmatter {
    /// Prompt name (defaults to filename if not specified).
    #[serde(default)]
    pub name: Option<String>,

    /// Prompt description.
    #[serde(default)]
    pub description: Option<String>,

    /// Available tools (array format only).
    #[serde(default)]
    pub tools: Option<Vec<String>>,

    /// Input field hint.
    #[serde(default)]
    pub hint: Option<String>,

    /// Model to use (GPT-4o, GPT-4o-mini, o1).
    #[serde(default)]
    pub model: Option<String>,

    /// Referenced agent name.
    #[serde(default)]
    pub agent: Option<String>,
}

/// Parsed Copilot Prompt.
#[derive(Debug, Clone)]
pub struct CopilotPrompt {
    /// Prompt name.
    pub name: Option<String>,
    /// Prompt description.
    pub description: Option<String>,
    /// Available tools.
    pub tools: Option<Vec<String>>,
    /// Input hint.
    pub hint: Option<String>,
    /// Model to use.
    pub model: Option<String>,
    /// Referenced agent.
    pub agent: Option<String>,
    /// Prompt body.
    pub body: String,
}

impl CopilotPrompt {
    /// Parses a Copilot Prompt from content string.
    ///
    /// The name field is taken directly from frontmatter (no filename fallback).
    pub fn parse(content: &str) -> Result<Self> {
        let ParsedDocument { frontmatter, body } =
            parse_frontmatter::<CopilotPromptFrontmatter>(content)?;

        let fm = frontmatter.unwrap_or_default();

        Ok(CopilotPrompt {
            name: normalize_name(fm.name),
            description: fm.description,
            tools: fm.tools,
            hint: fm.hint,
            model: fm.model,
            agent: fm.agent,
            body,
        })
    }

    /// Loads and parses a Copilot Prompt from a file.
    ///
    /// If the frontmatter doesn't specify a name, the filename is used as fallback.
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let mut prompt = Self::parse(&content)?;

        // Fallback to filename if name is not specified
        if prompt.name.is_none() {
            prompt.name = extract_name_from_path(path);
        }

        Ok(prompt)
    }

    /// Serializes to Copilot Markdown format.
    pub fn to_markdown(&self) -> String {
        let mut fields: Vec<String> = Vec::new();

        if let Some(ref v) = self.name {
            fields.push(format!("name: {}", convert::escape_yaml_string(v)));
        }
        if let Some(ref v) = self.description {
            fields.push(format!("description: {}", convert::escape_yaml_string(v)));
        }
        if let Some(ref v) = self.tools {
            // YAML array format: tools: ['codebase', 'terminal']
            let arr = v
                .iter()
                .map(|t| format!("'{}'", t.replace('\'', "''")))
                .collect::<Vec<_>>()
                .join(", ");
            fields.push(format!("tools: [{}]", arr));
        }
        if let Some(ref v) = self.hint {
            fields.push(format!("hint: {}", convert::escape_yaml_string(v)));
        }
        if let Some(ref v) = self.model {
            fields.push(format!("model: {}", v));
        }
        if let Some(ref v) = self.agent {
            fields.push(format!("agent: {}", convert::escape_yaml_string(v)));
        }

        if fields.is_empty() {
            self.body.clone()
        } else {
            format!("---\n{}\n---\n\n{}", fields.join("\n"), self.body)
        }
    }
}

// ============================================================================
// From trait implementations
// ============================================================================

impl From<&ClaudeCodeCommand> for CopilotPrompt {
    fn from(cmd: &ClaudeCodeCommand) -> Self {
        // Tool conversion: comma-separated string -> array -> convert -> deduplicate
        let tools = cmd
            .allowed_tools
            .as_ref()
            .map(|t| convert::tools_claude_to_copilot(&convert::parse_allowed_tools(t)));

        // Hint conversion: [message] -> "Enter message"
        let hint = cmd.argument_hint.as_ref().map(|h| {
            let inner = h.trim_start_matches('[').trim_end_matches(']');
            format!("Enter {}", inner)
        });

        CopilotPrompt {
            name: cmd.name.clone(),
            description: cmd.description.clone(),
            tools,
            hint,
            model: cmd
                .model
                .as_ref()
                .map(|m| convert::model_claude_to_copilot(m)),
            agent: None,
            body: convert::body_claude_to_copilot(&cmd.body),
        }
    }
}

impl From<&CodexPrompt> for CopilotPrompt {
    fn from(prompt: &CodexPrompt) -> Self {
        // Convert via ClaudeCodeCommand
        let cmd = ClaudeCodeCommand::from(prompt);
        CopilotPrompt::from(&cmd)
    }
}

/// Normalizes name: empty or whitespace-only string becomes None.
fn normalize_name(name: Option<String>) -> Option<String> {
    name.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// Extracts prompt name from file path.
///
/// Removes `.prompt.md` or `.md` extension from the filename.
fn extract_name_from_path(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|s| {
            // Try .prompt.md first, then .md
            s.strip_suffix(".prompt.md")
                .or_else(|| s.strip_suffix(".md"))
                .unwrap_or(s)
                .to_string()
        })
        .filter(|s| !s.is_empty())
}
