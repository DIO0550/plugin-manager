//! Claude Code-specific frontmatter normalization.

/// Claude Code frontmatter schema used during description normalization.
pub(super) struct FrontmatterSchema {
    /// Whether the YAML accepts `name`.
    name: bool,
    /// Whether the YAML accepts `description`.
    description: bool,
    /// Whether the YAML accepts `tools`.
    tools: bool,
    /// Whether the YAML accepts `allowed-tools`.
    allowed_tools: bool,
    /// Whether the YAML accepts `argument-hint`.
    argument_hint: bool,
    /// Whether the YAML accepts `model`.
    model: bool,
    /// Whether the YAML accepts `disable-model-invocation`.
    disable_model_invocation: bool,
    /// Whether the YAML accepts `user-invocable`.
    user_invocable: bool,
}

/// Agent frontmatter schema.
pub(super) const AGENT_FRONTMATTER_SCHEMA: FrontmatterSchema = FrontmatterSchema {
    name: true,
    description: true,
    tools: true,
    allowed_tools: false,
    argument_hint: false,
    model: true,
    disable_model_invocation: false,
    user_invocable: false,
};

/// Command frontmatter schema.
pub(super) const COMMAND_FRONTMATTER_SCHEMA: FrontmatterSchema = FrontmatterSchema {
    name: true,
    description: true,
    tools: false,
    allowed_tools: true,
    argument_hint: true,
    model: true,
    disable_model_invocation: true,
    user_invocable: true,
};

impl FrontmatterSchema {
    /// Normalizes Claude Code frontmatter so serde_yaml can parse official shapes.
    ///
    /// Runs description example repair first, then quotes unquoted `argument-hint`
    /// values so flow sequences such as `[message]` and adjacent pairs such as
    /// `[filename] [format]` deserialize as display strings.
    ///
    /// # Arguments
    ///
    /// * `yaml` - Claude Code frontmatter without `---` delimiter lines.
    pub(super) fn normalize(&self, yaml: &str) -> String {
        let yaml = self.normalize_description_examples(yaml);
        self.quote_unquoted_argument_hint(&yaml)
    }

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

    /// Quotes an unquoted `argument-hint` so official Claude Code examples parse.
    ///
    /// Official docs write `argument-hint: [message]` and
    /// `argument-hint: [filename] [format]`. YAML treats the former as a sequence
    /// and the latter as invalid or a sequence, so `Option<String>` fails. The
    /// remainder of the line is wrapped as a double-quoted scalar. Already quoted
    /// or block-scalar values are left unchanged.
    ///
    /// # Arguments
    ///
    /// * `yaml` - Frontmatter YAML, typically after description normalization.
    fn quote_unquoted_argument_hint(&self, yaml: &str) -> String {
        if !self.argument_hint {
            return yaml.to_string();
        }
        yaml.lines()
            .map(|line| self.quote_argument_hint_line(line))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Quotes one top-level `argument-hint:` line when the value is unquoted.
    ///
    /// # Arguments
    ///
    /// * `line` - A single frontmatter line.
    fn quote_argument_hint_line(&self, line: &str) -> String {
        if line.starts_with(char::is_whitespace) {
            return line.to_string();
        }
        let Some(rest) = line.strip_prefix("argument-hint:") else {
            return line.to_string();
        };
        let value = rest.trim();
        if value.is_empty() || Self::is_quoted_or_block_scalar(value) {
            return line.to_string();
        }
        format!("argument-hint: {}", Self::double_quote_yaml(value))
    }

    /// Returns whether a YAML value is already quoted or a block scalar indicator.
    ///
    /// # Arguments
    ///
    /// * `value` - Trimmed text after `argument-hint:`.
    fn is_quoted_or_block_scalar(value: &str) -> bool {
        matches!(value.as_bytes()[0], b'\'' | b'"' | b'|' | b'>')
    }

    /// Wraps `value` as a double-quoted YAML scalar.
    ///
    /// # Arguments
    ///
    /// * `value` - Raw argument-hint text to embed as a string.
    fn double_quote_yaml(value: &str) -> String {
        let escaped = value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n");
        format!("\"{}\"", escaped)
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
        self.is_supported_field(key)
            || key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    }

    /// Determines whether a key is a supported top-level YAML field.
    ///
    /// # Arguments
    ///
    /// * `key` - Top-level YAML key to inspect.
    ///
    /// # Returns
    ///
    /// `true` when the corresponding schema property is enabled.
    fn is_supported_field(&self, key: &str) -> bool {
        match key {
            "name" => self.name,
            "description" => self.description,
            "tools" => self.tools,
            "allowed-tools" => self.allowed_tools,
            "argument-hint" => self.argument_hint,
            "model" => self.model,
            "disable-model-invocation" => self.disable_model_invocation,
            "user-invocable" => self.user_invocable,
            _ => false,
        }
    }
}
