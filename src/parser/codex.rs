//! Codex Prompt parser.
//!
//! Parses `~/.codex/prompts/<name>.md` files.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use super::frontmatter::{parse_frontmatter, ParsedDocument};

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
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let mut prompt = Self::parse(&content)?;

        // Always use filename as name for Codex prompts
        prompt.name = extract_name_from_path(path);

        Ok(prompt)
    }
}

/// Extracts prompt name from file path.
///
/// Removes the `.md` extension from the filename.
fn extract_name_from_path(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.strip_suffix(".md").unwrap_or(s).to_string())
        .filter(|s| !s.is_empty())
}
