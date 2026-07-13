//! Codex Prompt parser.
//!
//! Parses `~/.codex/prompts/<name>.md` files.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use super::super::convert;
use super::super::convert::TargetFormat;
use super::super::frontmatter::{
    emit_frontmatter, parse_frontmatter, stem_without_suffixes, ParsedDocument,
};

/// Codex Prompt frontmatter fields.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodexPromptFrontmatter {
    /// Prompt description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Parsed Codex Prompt.
#[derive(Debug, Clone)]
pub struct CodexPrompt {
    /// Prompt name (from filename, Codex doesn't have name in frontmatter).
    pub name: Option<String>,
    /// Prompt description.
    pub description: Option<String>,
    /// Prompt body.
    pub body: String,
}

impl CodexPrompt {
    /// Parses a Codex Prompt from content string.
    ///
    /// Note: Codex frontmatter doesn't have a name field, so name is always None
    /// when using parse(). Use load() to get the name from the filename.
    ///
    /// # Arguments
    ///
    /// * `content` - Raw markdown content including optional YAML frontmatter.
    pub fn parse(content: &str) -> Result<Self> {
        let ParsedDocument { frontmatter, body } =
            parse_frontmatter::<CodexPromptFrontmatter>(content)?;

        let fm = frontmatter.unwrap_or_default();

        Ok(CodexPrompt {
            name: None, // Codex doesn't have name field in frontmatter
            description: fm.description,
            body,
        })
    }

    /// Loads and parses a Codex Prompt from a file.
    ///
    /// The filename (without .md extension) is used as the prompt name.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `~/.codex/prompts/<name>.md` file to load.
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let mut prompt = Self::parse(&content)?;

        prompt.name = stem_without_suffixes(path, &[".md"]);

        Ok(prompt)
    }
}

impl TargetFormat for CodexPrompt {
    fn to_markdown(&self) -> String {
        let mut fields: Vec<String> = Vec::new();

        // Codex doesn't include name in frontmatter
        if let Some(ref v) = self.description {
            fields.push(format!("description: {}", convert::escape_yaml_string(v)));
        }

        emit_frontmatter(&fields, &self.body)
    }
}
