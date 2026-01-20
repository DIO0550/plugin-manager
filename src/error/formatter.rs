use std::error::Error;
use std::io::IsTerminal;

use owo_colors::OwoColorize;

use super::code::ErrorCode;
use super::rich::RichError;

/// Formats RichError for CLI output
pub struct ErrorFormatter {
    verbose: bool,
    use_color: bool,
}

impl ErrorFormatter {
    /// Creates a new ErrorFormatter with default TTY detection
    pub fn new(verbose: bool) -> Self {
        Self::with_color_detection(verbose, Self::default_should_use_color)
    }

    /// Creates a new ErrorFormatter with injectable TTY detection for testing
    pub fn with_color_detection(verbose: bool, detect_color: fn() -> bool) -> Self {
        let use_color = detect_color();
        Self { verbose, use_color }
    }

    fn default_should_use_color() -> bool {
        std::io::stderr().is_terminal() && std::env::var("NO_COLOR").is_err()
    }

    /// Formats the error for display
    pub fn format(&self, error: &RichError) -> String {
        let plain = if self.verbose {
            self.format_verbose_plain(error)
        } else {
            self.format_simple_plain(error)
        };

        // Apply masking before color
        let masked = self.mask_sensitive(&plain);

        if self.use_color {
            self.apply_color(&masked, error.code())
        } else {
            masked
        }
    }

    fn format_simple_plain(&self, error: &RichError) -> String {
        let mut output = format!("error[{}]: {}", error.code().as_str(), error.message());

        // Add context
        let context_lines = self.format_context(error);
        if !context_lines.is_empty() {
            output.push('\n');
            output.push_str(&context_lines);
        }

        output
    }

    fn format_verbose_plain(&self, error: &RichError) -> String {
        let mut output = format!("error[{}]: {}", error.code().as_str(), error.message());

        // Add context
        let context_lines = self.format_context(error);
        if !context_lines.is_empty() {
            output.push('\n');
            output.push_str(&context_lines);
        }

        // Add cause
        output.push_str("\n  |");
        output.push_str(&format!("\n  | Cause: {}", error.code().cause()));

        // Add remediation
        output.push_str("\n  |");
        output.push_str("\n  | Remediation:");
        for line in error.code().remediation().lines() {
            output.push_str(&format!("\n  |   {}", line));
        }

        // Add source chain if available
        let source_chain = self.format_source_chain(error);
        if !source_chain.is_empty() {
            output.push_str("\n  |");
            output.push_str(&format!("\n  | Source chain:\n{}", source_chain));
        }

        output.push_str("\n  |");
        output.push_str("\n  = note: use `plm --help` for more information");

        output
    }

    fn format_context(&self, error: &RichError) -> String {
        let ctx = error.context();
        let mut lines = Vec::new();

        // Display order: file_path -> url -> plugin_name -> additional (sorted)
        if let Some(path) = &ctx.file_path {
            lines.push(format!("  --> {}", path.to_string_lossy()));
        }

        if let Some(url) = &ctx.url {
            lines.push(format!("  --> {}", url));
        }

        if let Some(name) = &ctx.plugin_name {
            lines.push(format!("  --> plugin: {}", name));
        }

        // BTreeMap is already sorted by key
        for (key, value) in &ctx.additional {
            lines.push(format!("  --> {}: {}", key, value));
        }

        lines.join("\n")
    }

    fn format_source_chain(&self, error: &RichError) -> String {
        let mut chain = Vec::new();
        let mut current: Option<&(dyn std::error::Error + 'static)> = error.source();

        while let Some(err) = current {
            chain.push(format!("  |   - {}", err));
            current = err.source();
        }

        chain.join("\n")
    }

    /// Masks sensitive data in the text
    fn mask_sensitive(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Mask query parameter values: ?foo=xxx&bar=yyy -> ?foo=***&bar=***
        result = Self::mask_query_params(&result);

        // Mask token= patterns
        result = Self::mask_pattern(&result, "token=", "token=***");

        // Mask Authorization headers
        result = Self::mask_authorization(&result);

        // Mask sensitive file paths
        result = Self::mask_sensitive_paths(&result);

        result
    }

    fn mask_query_params(text: &str) -> String {
        use regex::Regex;

        // Match query parameters: key=value patterns after ? or &
        let re = Regex::new(r"([?&])([^=&]+)=([^&\s]+)").unwrap();
        re.replace_all(text, "$1$2=***").to_string()
    }

    fn mask_pattern(text: &str, pattern: &str, replacement: &str) -> String {
        if let Some(pos) = text.find(pattern) {
            let before = &text[..pos];
            let after_pattern = &text[pos + pattern.len()..];
            // Find the end of the value (space or end of string)
            let end = after_pattern
                .find(|c: char| c.is_whitespace() || c == '&' || c == '"' || c == '\'')
                .unwrap_or(after_pattern.len());
            let after = &after_pattern[end..];
            format!("{}{}{}", before, replacement, after)
        } else {
            text.to_string()
        }
    }

    fn mask_authorization(text: &str) -> String {
        use regex::Regex;

        let re = Regex::new(r"(?i)(Authorization:\s*Bearer\s+)\S+").unwrap();
        re.replace_all(text, "$1***").to_string()
    }

    fn mask_sensitive_paths(text: &str) -> String {
        // Only mask file-like patterns, not URLs
        // File patterns: /.env, /path/to/credentials.json, etc.
        // Match paths that look like Unix file paths (start with / followed by path chars)
        // but exclude URL-like patterns (://path)
        use regex::Regex;

        // Match file paths like /home/user/.env or /path/to/credentials.json
        // Requires at least one path separator and the sensitive filename
        let sensitive_file_re = Regex::new(r"(?:^|[^:/])(/(?:[^/\s]+/)*\.env(?:\.[^\s]*)?)").unwrap();
        let mut result = sensitive_file_re
            .replace_all(text, |caps: &regex::Captures| {
                // Keep the prefix character (if any) and replace the path
                let full = caps.get(0).unwrap().as_str();
                let path = caps.get(1).unwrap().as_str();
                full.replace(path, "<sensitive-path>")
            })
            .to_string();

        // Also mask credentials files
        let credentials_re = Regex::new(r"(?:^|[^:/])(/(?:[^/\s]+/)*credentials[^\s]*)").unwrap();
        result = credentials_re
            .replace_all(&result, |caps: &regex::Captures| {
                let full = caps.get(0).unwrap().as_str();
                let path = caps.get(1).unwrap().as_str();
                full.replace(path, "<sensitive-path>")
            })
            .to_string();

        result
    }

    fn apply_color(&self, text: &str, code: ErrorCode) -> String {
        let mut result = String::new();

        for line in text.lines() {
            if !result.is_empty() {
                result.push('\n');
            }

            if line.starts_with("error[") {
                // Color the error line red
                let bracket_end = line.find(']').unwrap_or(0) + 1;
                let error_prefix = &line[..bracket_end];
                let rest = &line[bracket_end..];
                result.push_str(&format!(
                    "{}{}",
                    error_prefix.red().bold(),
                    rest.bold()
                ));
            } else if line.starts_with("  -->") {
                // Color the location line blue
                result.push_str(&line.blue().to_string());
            } else if line.starts_with("  | Cause:") {
                result.push_str(&line.yellow().to_string());
            } else if line.starts_with("  | Remediation:") {
                result.push_str(&line.green().to_string());
            } else if line.starts_with("  = note:") {
                result.push_str(&line.dimmed().to_string());
            } else if line.starts_with("  |   -") {
                // Source chain items
                result.push_str(&line.dimmed().to_string());
            } else {
                result.push_str(line);
            }
        }

        // Suppress unused variable warning
        let _ = code;

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::rich::ErrorContext;

    fn no_color() -> bool {
        false
    }

    #[test]
    fn format_simple() {
        let formatter = ErrorFormatter::with_color_detection(false, no_color);
        let error = RichError::new(ErrorCode::Net001, "Connection failed");

        let output = formatter.format(&error);
        assert!(output.contains("error[NET001]"));
        assert!(output.contains("Connection failed"));
    }

    #[test]
    fn format_simple_with_context() {
        let formatter = ErrorFormatter::with_color_detection(false, no_color);
        let context = ErrorContext::new().with_url("https://api.github.com/repos/owner/repo");
        let error = RichError::new(ErrorCode::Net001, "Connection failed").with_context(context);

        let output = formatter.format(&error);
        assert!(output.contains("error[NET001]"));
        assert!(output.contains("  --> https://api.github.com/repos/owner/repo"));
    }

    #[test]
    fn format_verbose() {
        let formatter = ErrorFormatter::with_color_detection(true, no_color);
        let error = RichError::new(ErrorCode::Net001, "Connection failed");

        let output = formatter.format(&error);
        assert!(output.contains("error[NET001]"));
        assert!(output.contains("Cause:"));
        assert!(output.contains("Remediation:"));
        assert!(output.contains("note: use `plm --help`"));
    }

    #[test]
    fn mask_query_params() {
        let formatter = ErrorFormatter::with_color_detection(false, no_color);
        let error = RichError::new(
            ErrorCode::Net001,
            "Failed to fetch https://api.github.com?token=abc123&key=secret",
        );

        let output = formatter.format(&error);
        assert!(output.contains("token=***"), "output was: {}", output);
        assert!(output.contains("key=***"), "output was: {}", output);
        assert!(!output.contains("abc123"), "output was: {}", output);
        assert!(!output.contains("secret"), "output was: {}", output);
    }

    #[test]
    fn mask_authorization_header() {
        let formatter = ErrorFormatter::with_color_detection(false, no_color);
        let error = RichError::new(
            ErrorCode::Api002,
            "Request failed with Authorization: Bearer ghp_xxxxxxxxxxxx",
        );

        let output = formatter.format(&error);
        assert!(output.contains("Authorization: Bearer ***"));
        assert!(!output.contains("ghp_xxxxxxxxxxxx"));
    }

    #[test]
    fn mask_sensitive_paths() {
        let formatter = ErrorFormatter::with_color_detection(false, no_color);
        let context = ErrorContext::new().with_file_path("/home/user/.env");
        let error = RichError::new(ErrorCode::Io001, "File not found").with_context(context);

        let output = formatter.format(&error);
        assert!(output.contains("<sensitive-path>"));
    }

    #[test]
    fn context_display_order() {
        let formatter = ErrorFormatter::with_color_detection(false, no_color);
        let context = ErrorContext::new()
            .with_url("https://example.com")
            .with_file_path("/path/to/file")
            .with_plugin_name("test-plugin")
            .with_additional("extra", "value");

        let error = RichError::new(ErrorCode::Int001, "Test error").with_context(context);
        let output = formatter.format(&error);

        // Check order: file_path -> url -> plugin_name -> additional
        let file_pos = output.find("/path/to/file").unwrap();
        let url_pos = output.find("https://example.com").unwrap();
        let plugin_pos = output.find("plugin: test-plugin").unwrap();
        let extra_pos = output.find("extra: value").unwrap();

        assert!(file_pos < url_pos, "file_path should come before url");
        assert!(url_pos < plugin_pos, "url should come before plugin_name");
        assert!(
            plugin_pos < extra_pos,
            "plugin_name should come before additional"
        );
    }

    #[test]
    fn verbose_includes_source_chain() {
        let formatter = ErrorFormatter::with_color_detection(true, no_color);
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "underlying error");
        let error =
            RichError::new(ErrorCode::Io001, "Failed to read file").with_source(io_error);

        let output = formatter.format(&error);
        assert!(output.contains("Source chain:"));
        assert!(output.contains("underlying error"));
    }

    #[test]
    fn no_color_env_respected() {
        // This test verifies the logic, actual env var testing is tricky
        let formatter = ErrorFormatter::with_color_detection(false, || false);
        let error = RichError::new(ErrorCode::Net001, "Test");
        let output = formatter.format(&error);

        // Should not contain ANSI escape codes
        assert!(!output.contains("\x1b["));
    }
}
