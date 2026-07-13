//! Generic YAML frontmatter parser.
//!
//! Parses documents with YAML frontmatter delimited by `---` markers.
//! Also provides shared helpers for name extraction and frontmatter emission.

use crate::error::{PlmError, Result};
use serde::de::DeserializeOwned;
use std::path::Path;

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
///
/// # Arguments
///
/// * `content` - Document text whose optional YAML frontmatter should be parsed.
pub fn parse_frontmatter<T: DeserializeOwned + Default>(
    content: &str,
) -> Result<ParsedDocument<T>> {
    // Remove UTF-8 BOM if present
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);

    let lines: Vec<&str> = content.lines().collect();

    let first_line = lines.first().map(|s| s.trim()).unwrap_or("");
    if !first_line.starts_with("---") {
        // No frontmatter - entire content is body
        return Ok(ParsedDocument {
            frontmatter: None,
            body: content.to_string(),
        });
    }

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

    let yaml_content: String = lines[1..closing_index].join("\n");

    let frontmatter: T = if yaml_content.trim().is_empty() {
        // Empty frontmatter - use default
        T::default()
    } else {
        serde_yaml::from_str(&yaml_content).map_err(PlmError::Yaml)?
    };

    // Extract body (everything after closing ---)
    // Preserve exact content including leading newlines
    let body = if closing_index + 1 < lines.len() {
        let mut offset = 0;
        for (i, line) in lines.iter().enumerate() {
            if i <= closing_index {
                offset += line.len() + 1; // +1 for newline
            } else {
                break;
            }
        }
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

/// Extracts a component name from a file path by stripping known suffixes.
///
/// Suffixes are tried in the given order; callers should pass longer suffixes
/// first (e.g. `.agent.md` before `.md`) so that `foo.agent.md` becomes `foo`
/// rather than `foo.agent`.
///
/// # Arguments
///
/// * `path` - File path whose file name will be used as the stem source.
/// * `suffixes` - Candidate suffixes to strip, in preference order.
pub(crate) fn stem_without_suffixes(path: &Path, suffixes: &[&str]) -> Option<String> {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|s| {
            suffixes
                .iter()
                .find_map(|suffix| s.strip_suffix(suffix))
                .unwrap_or(s)
                .to_string()
        })
        .filter(|s| !s.is_empty())
}

/// Normalizes an optional frontmatter name: trim whitespace, empty becomes None.
///
/// Distinct from [`crate::marketplace::normalize_name`], which validates
/// marketplace identifiers.
///
/// # Arguments
///
/// * `name` - Optional raw name string from frontmatter.
pub(crate) fn normalize_optional_name(name: Option<String>) -> Option<String> {
    name.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// Builds a YAML frontmatter envelope around `body`.
///
/// When `fields` is empty, returns `body` unchanged (no `---` markers).
///
/// # Arguments
///
/// * `fields` - Pre-formatted `key: value` lines for the frontmatter block.
/// * `body` - Markdown body after the frontmatter.
pub(crate) fn emit_frontmatter(fields: &[String], body: &str) -> String {
    if fields.is_empty() {
        body.to_string()
    } else {
        format!("---\n{}\n---\n\n{}", fields.join("\n"), body)
    }
}

/// Formats a YAML flow sequence using single-quoted scalars.
///
/// Single quotes inside items are escaped by doubling (`'` → `''`).
///
/// # Arguments
///
/// * `items` - Array elements to serialize (e.g. Copilot `tools`).
pub(crate) fn yaml_single_quoted_array(items: &[String]) -> String {
    let arr = items
        .iter()
        .map(|t| format!("'{}'", t.replace('\'', "''")))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{}]", arr)
}
