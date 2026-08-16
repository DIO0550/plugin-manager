//! Claude Code-specific frontmatter normalization.

/// Claude Code frontmatter schema used during description normalization.
pub(super) struct FrontmatterSchema {
    /// Top-level YAML fields supported by the target component.
    top_level_fields: &'static [&'static str],
}

/// Agent frontmatter schema.
pub(super) const AGENT_FRONTMATTER_SCHEMA: FrontmatterSchema = FrontmatterSchema {
    top_level_fields: &["name", "description", "tools", "model"],
};

/// Command frontmatter schema.
pub(super) const COMMAND_FRONTMATTER_SCHEMA: FrontmatterSchema = FrontmatterSchema {
    top_level_fields: &[
        "name",
        "description",
        "allowed-tools",
        "argument-hint",
        "model",
        "disable-model-invocation",
        "user-invocable",
    ],
};

impl FrontmatterSchema {
    /// Normalizes a malformed plain-scalar description containing example blocks.
    ///
    /// Only a `description:` whose following line is `Examples:` or `<example>` is
    /// converted to a YAML literal block. Known metadata fields and unknown
    /// top-level keys terminate the description, while Claude example labels remain
    /// part of it.
    ///
    /// # Arguments
    ///
    /// * `yaml` - Claude Code frontmatter without `---` delimiter lines.
    ///
    /// # Returns
    ///
    /// Normalized YAML, or the original YAML unchanged when the targeted malformed
    /// description shape is not present.
    pub(super) fn normalize_description_examples(&self, yaml: &str) -> String {
        let lines: Vec<&str> = yaml.lines().collect();
        let Some(description_index) = lines.iter().position(|line| {
            line.strip_prefix("description:")
                .is_some_and(|value| Self::is_plain_scalar(value.trim()))
        }) else {
            return yaml.to_string();
        };

        let example_index = description_index + 1;
        if !lines
            .get(example_index)
            .is_some_and(|line| matches!(line.trim(), "Examples:" | "<example>"))
        {
            return yaml.to_string();
        }

        let end = lines[example_index..]
            .iter()
            .position(|line| self.is_metadata_boundary(line))
            .map(|offset| example_index + offset)
            .unwrap_or(lines.len());
        let first = lines[description_index]
            .strip_prefix("description:")
            .unwrap()
            .trim_start();
        let mut normalized: Vec<String> = Vec::with_capacity(lines.len() + 1);
        normalized.extend(
            lines[..description_index]
                .iter()
                .map(|line| (*line).to_string()),
        );
        normalized.push("description: |-".to_string());
        normalized.push(format!("  {first}"));
        for line in &lines[example_index..end] {
            normalized.push(format!("  {line}"));
        }
        normalized.extend(lines[end..].iter().map(|line| (*line).to_string()));
        normalized.join("\n")
    }

    /// Determines whether a description value uses a non-empty YAML plain scalar.
    ///
    /// # Arguments
    ///
    /// * `value` - Text after the `description:` key.
    ///
    /// # Returns
    ///
    /// `true` when the value is non-empty and does not start with YAML quoting,
    /// collection, or block-scalar syntax.
    fn is_plain_scalar(value: &str) -> bool {
        !value.is_empty()
            && !matches!(
                value.as_bytes()[0],
                b'\'' | b'"' | b'[' | b'{' | b'|' | b'>'
            )
    }

    /// Determines whether a line starts a top-level metadata field.
    ///
    /// # Arguments
    ///
    /// * `line` - Frontmatter line to inspect.
    ///
    /// # Returns
    ///
    /// `true` when the line is a known or unknown YAML-like top-level key, except
    /// for labels explicitly allowed inside Claude example descriptions.
    fn is_metadata_boundary(&self, line: &str) -> bool {
        if line.starts_with(char::is_whitespace) || line.is_empty() {
            return false;
        }
        let Some((key, _)) = line.split_once(':') else {
            return false;
        };
        if matches!(key, "Examples" | "Context" | "user" | "assistant") {
            return false;
        }
        self.top_level_fields.contains(&key)
            || key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    }
}
