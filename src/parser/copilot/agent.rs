//! Copilot Agent parser.
//!
//! Parses `.github/agents/<name>.agent.md` files.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use super::super::convert;
use super::super::convert::TargetFormat;
use super::super::frontmatter::{
    emit_frontmatter, normalize_optional_name, parse_frontmatter, stem_without_suffixes,
    yaml_single_quoted_array, ParsedDocument,
};

/// Copilot Agent handoff entry.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CopilotAgentHandoff {
    /// Target agent name.
    #[serde(default)]
    pub agent: Option<String>,
    /// UI label for the handoff.
    #[serde(default)]
    pub label: Option<String>,
    /// Prompt to send when handing off.
    #[serde(default)]
    pub prompt: Option<String>,
    /// Whether to send the prompt immediately.
    #[serde(default)]
    pub send: Option<bool>,
}

/// Copilot Agent frontmatter fields.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CopilotAgentFrontmatter {
    /// Agent name (defaults to filename if not specified).
    #[serde(default)]
    pub name: Option<String>,

    /// Agent description.
    #[serde(default)]
    pub description: Option<String>,

    /// Available tools (array format).
    #[serde(default)]
    pub tools: Option<Vec<String>>,

    /// Model to use.
    #[serde(default)]
    pub model: Option<String>,

    /// Target environment (vscode, github-copilot).
    #[serde(default)]
    pub target: Option<String>,

    /// Workflow handoffs.
    #[serde(default)]
    pub handoffs: Option<Vec<CopilotAgentHandoff>>,
}

/// Parsed Copilot Agent.
#[derive(Debug, Clone)]
pub struct CopilotAgent {
    /// Agent name.
    pub name: Option<String>,
    /// Agent description.
    pub description: Option<String>,
    /// Available tools.
    pub tools: Option<Vec<String>>,
    /// Model to use.
    pub model: Option<String>,
    /// Target environment.
    pub target: Option<String>,
    /// Workflow handoffs.
    pub handoffs: Option<Vec<CopilotAgentHandoff>>,
    /// Agent body.
    pub body: String,
}

impl CopilotAgent {
    /// Parses a Copilot Agent from content string.
    ///
    /// The name field is taken directly from frontmatter (no filename fallback).
    ///
    /// # Arguments
    ///
    /// * `content` - Raw markdown content including optional YAML frontmatter.
    pub fn parse(content: &str) -> Result<Self> {
        let ParsedDocument { frontmatter, body } =
            parse_frontmatter::<CopilotAgentFrontmatter>(content)?;

        let fm = frontmatter.unwrap_or_default();

        Ok(CopilotAgent {
            name: normalize_optional_name(fm.name),
            description: fm.description,
            tools: fm.tools,
            model: fm.model,
            target: fm.target,
            handoffs: fm.handoffs,
            body,
        })
    }

    /// Loads and parses a Copilot Agent from a file.
    ///
    /// If the frontmatter doesn't specify a name, the filename is used as fallback.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `.github/agents/<name>.agent.md` file to load.
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let mut agent = Self::parse(&content)?;

        if agent.name.is_none() {
            agent.name = stem_without_suffixes(path, &[".agent.md", ".md"]);
        }

        Ok(agent)
    }
}

impl TargetFormat for CopilotAgent {
    fn to_markdown(&self) -> String {
        let mut fields: Vec<String> = Vec::new();

        if let Some(ref v) = self.name {
            fields.push(format!("name: {}", convert::escape_yaml_string(v)));
        }
        if let Some(ref v) = self.description {
            fields.push(format!("description: {}", convert::escape_yaml_string(v)));
        }
        if let Some(ref tools) = self.tools {
            if !tools.is_empty() {
                // YAML array format: tools: ['codebase', 'terminal']
                fields.push(format!("tools: {}", yaml_single_quoted_array(tools)));
            }
        }
        if let Some(ref v) = self.model {
            fields.push(format!("model: {}", v));
        }
        if let Some(ref v) = self.target {
            fields.push(format!("target: {}", v));
        }
        if let Some(ref handoffs) = self.handoffs {
            if !handoffs.is_empty() {
                fields.push("handoffs:".to_string());
                for h in handoffs {
                    let mut h_fields: Vec<String> = Vec::new();
                    if let Some(ref a) = h.agent {
                        h_fields.push(format!("agent: {}", convert::escape_yaml_string(a)));
                    }
                    if let Some(ref l) = h.label {
                        h_fields.push(format!("label: {}", convert::escape_yaml_string(l)));
                    }
                    if let Some(ref p) = h.prompt {
                        h_fields.push(format!("prompt: {}", convert::escape_yaml_string(p)));
                    }
                    if let Some(s) = h.send {
                        h_fields.push(format!("send: {}", s));
                    }
                    if !h_fields.is_empty() {
                        fields.push(format!("  - {}", h_fields.join("\n    ")));
                    }
                }
            }
        }

        emit_frontmatter(&fields, &self.body)
    }
}
