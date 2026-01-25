//! Generic YAML frontmatter parser.
//!
//! Parses documents with YAML frontmatter delimited by `---` markers.

use crate::error::{PlmError, Result};
use serde::de::DeserializeOwned;

/// Parsed document with separated frontmatter and body.
#[derive(Debug, Clone)]
pub struct ParsedDocument<T> {
    /// Parsed frontmatter (None if no frontmatter present).
    pub frontmatter: Option<T>,
    /// Body text (everything after the frontmatter).
    pub body: String,
}

/// Parses YAML frontmatter from a document.
///
/// # Format
///
/// Documents may optionally start with YAML frontmatter delimited by `---`:
///
/// ```text
/// ---
/// key: value
/// ---
///
/// Body text here...
/// ```
///
/// # Rules
///
/// - UTF-8 BOM (`\u{feff}`) at the start is removed
/// - First line must start with `---` (after trimming)
/// - Closing `---` must be on its own line
/// - Body includes everything after the closing `---` (leading newlines preserved)
/// - If no frontmatter is present, the entire content is treated as body
/// - Empty frontmatter (`---\n---`) uses `T::default()`
///
/// # Type Parameters
///
/// - `T`: The type to deserialize the frontmatter into. Must implement
///   `DeserializeOwned` and `Default` (for empty frontmatter handling).
pub fn parse_frontmatter<T: DeserializeOwned + Default>(
    content: &str,
) -> Result<ParsedDocument<T>> {
    // Remove UTF-8 BOM if present
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);

    let lines: Vec<&str> = content.lines().collect();

    // Check if first line starts with ---
    let first_line = lines.first().map(|s| s.trim()).unwrap_or("");
    if !first_line.starts_with("---") {
        // No frontmatter - entire content is body
        return Ok(ParsedDocument {
            frontmatter: None,
            body: content.to_string(),
        });
    }

    // Find closing ---
    let closing_index = lines
        .iter()
        .enumerate()
        .skip(1) // Skip the opening ---
        .find(|(_, line)| line.trim().starts_with("---"))
        .map(|(i, _)| i);

    let Some(closing_index) = closing_index else {
        // No closing --- found - treat entire content as body
        return Ok(ParsedDocument {
            frontmatter: None,
            body: content.to_string(),
        });
    };

    // Extract frontmatter YAML (between opening and closing ---)
    let yaml_content: String = lines[1..closing_index].join("\n");

    // Parse frontmatter
    let frontmatter: T = if yaml_content.trim().is_empty() {
        // Empty frontmatter - use default
        T::default()
    } else {
        serde_yaml::from_str(&yaml_content).map_err(PlmError::Yaml)?
    };

    // Extract body (everything after closing ---)
    // Preserve exact content including leading newlines
    let body = if closing_index + 1 < lines.len() {
        // Calculate byte offset of the body start
        let mut offset = 0;
        for (i, line) in lines.iter().enumerate() {
            if i <= closing_index {
                offset += line.len() + 1; // +1 for newline
            } else {
                break;
            }
        }
        // Handle case where content doesn't end with newline
        if offset <= content.len() {
            content[offset..].to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    Ok(ParsedDocument {
        frontmatter: Some(frontmatter),
        body,
    })
}
