use std::collections::BTreeMap;
use std::path::PathBuf;

use super::code::ErrorCode;

/// Structured context for error display
#[derive(Debug, Clone, Default)]
pub struct ErrorContext {
    pub file_path: Option<PathBuf>,
    pub url: Option<String>,
    pub plugin_name: Option<String>,
    /// Additional key-value pairs for context (BTreeMap for deterministic ordering)
    pub additional: BTreeMap<String, String>,
}

impl ErrorContext {
    /// Creates a new empty context
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the file path
    pub fn with_file_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    /// Sets the URL
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Sets the plugin name
    pub fn with_plugin_name(mut self, name: impl Into<String>) -> Self {
        self.plugin_name = Some(name.into());
        self
    }

    /// Adds an additional key-value pair
    pub fn with_additional(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.additional.insert(key.into(), value.into());
        self
    }

    /// Returns true if the context has any data
    pub fn is_empty(&self) -> bool {
        self.file_path.is_none()
            && self.url.is_none()
            && self.plugin_name.is_none()
            && self.additional.is_empty()
    }
}

/// Rich error with code, message, and context
pub struct RichError {
    code: ErrorCode,
    message: String,
    context: ErrorContext,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl std::fmt::Debug for RichError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RichError")
            .field("code", &self.code)
            .field("message", &self.message)
            .field("context", &self.context)
            .field("source", &self.source.as_ref().map(|e| e.to_string()))
            .finish()
    }
}

impl RichError {
    /// Creates a new RichError with the given code and message
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            context: ErrorContext::default(),
            source: None,
        }
    }

    /// Sets the error context
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = context;
        self
    }

    /// Sets the source error
    pub fn with_source<E>(mut self, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        self.source = Some(Box::new(source));
        self
    }

    /// Returns the error code
    pub fn code(&self) -> ErrorCode {
        self.code
    }

    /// Returns the error message
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the error context
    pub fn context(&self) -> &ErrorContext {
        &self.context
    }
}

impl std::fmt::Display for RichError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error[{}]: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for RichError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn rich_error_creation() {
        let error = RichError::new(ErrorCode::Net001, "Connection failed");
        assert_eq!(error.code(), ErrorCode::Net001);
        assert_eq!(error.message(), "Connection failed");
        assert!(error.context().is_empty());
    }

    #[test]
    fn rich_error_with_context() {
        let context = ErrorContext::new()
            .with_url("https://example.com")
            .with_plugin_name("my-plugin");

        let error = RichError::new(ErrorCode::Plg001, "Plugin not found").with_context(context);

        assert!(!error.context().is_empty());
        assert_eq!(error.context().url.as_deref(), Some("https://example.com"));
        assert_eq!(error.context().plugin_name.as_deref(), Some("my-plugin"));
    }

    #[test]
    fn rich_error_display() {
        let error = RichError::new(ErrorCode::Net001, "Connection failed");
        let display = format!("{}", error);
        assert_eq!(display, "error[NET001]: Connection failed");
    }

    #[test]
    fn error_context_builder() {
        let context = ErrorContext::new()
            .with_file_path("/path/to/file")
            .with_url("https://api.github.com")
            .with_plugin_name("test-plugin")
            .with_additional("key1", "value1")
            .with_additional("key2", "value2");

        assert_eq!(context.file_path, Some(PathBuf::from("/path/to/file")));
        assert_eq!(context.url, Some("https://api.github.com".to_string()));
        assert_eq!(context.plugin_name, Some("test-plugin".to_string()));
        assert_eq!(context.additional.len(), 2);
        assert_eq!(context.additional.get("key1"), Some(&"value1".to_string()));
    }

    #[test]
    fn error_context_is_empty() {
        let empty = ErrorContext::new();
        assert!(empty.is_empty());

        let with_url = ErrorContext::new().with_url("https://example.com");
        assert!(!with_url.is_empty());
    }

    #[test]
    fn rich_error_with_source() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = RichError::new(ErrorCode::Io001, "Failed to read file").with_source(io_error);

        assert!(error.source().is_some());
    }
}
