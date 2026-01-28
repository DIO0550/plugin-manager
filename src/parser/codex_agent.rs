//! Codex Agent parser.
//!
//! Parses `.codex/agents/<name>.agent.md` files.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use super::convert;
use super::convert::TargetFormat;
use super::frontmatter::{parse_frontmatter, ParsedDocument};

/// Codex Agent frontmatter fields.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodexAgentFrontmatter {
    /// Agent description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Parsed Codex Agent.
#[derive(Debug, Clone)]
pub struct CodexAgent {
    /// Agent name (from filename, Codex doesn't have name in frontmatter).
    pub name: Option<String>,
    /// Agent description.
    pub description: Option<String>,
    /// Agent body.
    pub body: String,
}

impl CodexAgent {
    /// Parses a Codex Agent from content string.
    ///
    /// Note: Codex frontmatter doesn't have a name field, so name is always None
    /// when using parse(). Use load() to get the name from the filename.
    pub fn parse(content: &str) -> Result<Self> {
        let ParsedDocument { frontmatter, body } =
            parse_frontmatter::<CodexAgentFrontmatter>(content)?;

        let fm = frontmatter.unwrap_or_default();

        Ok(CodexAgent {
            name: None,
            description: fm.description,
            body,
        })
    }

    /// Loads and parses a Codex Agent from a file.
    ///
    /// The filename (without extension) is used as the agent name.
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let mut agent = Self::parse(&content)?;

        agent.name = extract_name_from_path(path);

        Ok(agent)
    }
}

impl TargetFormat for CodexAgent {
    fn to_markdown(&self) -> String {
        let mut fields: Vec<String> = Vec::new();

        // Codex doesn't include name in frontmatter
        if let Some(ref v) = self.description {
            fields.push(format!("description: {}", convert::escape_yaml_string(v)));
        }

        if fields.is_empty() {
            self.body.clone()
        } else {
            format!("---\n{}\n---\n\n{}", fields.join("\n"), self.body)
        }
    }
}

/// Extracts agent name from file path.
///
/// Removes `.agent.md` or `.md` extension from the filename.
fn extract_name_from_path(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|s| {
            s.strip_suffix(".agent.md")
                .or_else(|| s.strip_suffix(".md"))
                .unwrap_or(s)
                .to_string()
        })
        .filter(|s| !s.is_empty())
}
