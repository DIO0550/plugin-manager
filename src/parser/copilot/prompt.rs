//! Copilot Prompt parser.
//!
//! Parses `.github/prompts/<name>.prompt.md` files.

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
    ///
    /// # Arguments
    ///
    /// * `content` - Raw markdown content including optional YAML frontmatter.
    pub fn parse(content: &str) -> Result<Self> {
        let ParsedDocument { frontmatter, body } =
            parse_frontmatter::<CopilotPromptFrontmatter>(content)?;

        let fm = frontmatter.unwrap_or_default();

        Ok(CopilotPrompt {
            name: normalize_optional_name(fm.name),
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
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `.github/prompts/<name>.prompt.md` file to load.
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let mut prompt = Self::parse(&content)?;

        if prompt.name.is_none() {
            prompt.name = stem_without_suffixes(path, &[".prompt.md", ".md"]);
        }

        Ok(prompt)
    }
}

impl TargetFormat for CopilotPrompt {
    fn to_markdown(&self) -> String {
        let mut fields: Vec<String> = Vec::new();

        if let Some(ref v) = self.name {
            fields.push(format!("name: {}", convert::escape_yaml_string(v)));
        }
        if let Some(ref v) = self.description {
            fields.push(format!("description: {}", convert::escape_yaml_string(v)));
        }
        if let Some(ref v) = self.tools {
            // YAML array format: tools: ['codebase', 'terminal']
            // Empty vec still emits `tools: []` (intentional; differs from CopilotAgent).
            fields.push(format!("tools: {}", yaml_single_quoted_array(v)));
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

        emit_frontmatter(&fields, &self.body)
    }
}
