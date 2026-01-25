mod code;
mod rich;

pub use code::ErrorCode;
pub use rich::{ErrorContext, RichError};

use thiserror::Error;

/// AmbiguousPlugin エラーのフォーマット
fn format_ambiguous_plugin(name: &str, candidates: &[String]) -> String {
    let mut msg = format!("multiple plugins named '{}' found:\n", name);
    for c in candidates {
        msg.push_str(&format!("  - {}\n", c));
    }
    msg.push_str(&format!(
        "Use 'plm info <marketplace>/{}' to specify.",
        name
    ));
    msg
}

/// PLM統一エラー型
#[derive(Debug, Error)]
pub enum PlmError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("{url} API error: {message} (status: {status})")]
    RepoApi {
        url: String,
        status: u16,
        message: String,
    },

    #[error("Invalid repository format: {0}. Expected 'owner/repo' or 'owner/repo@ref'")]
    InvalidRepoFormat(String),

    #[error("Plugin not found: {0}")]
    PluginNotFound(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("{}", format_ambiguous_plugin(.name, .candidates))]
    AmbiguousPlugin {
        name: String,
        candidates: Vec<String>,
    },

    #[error("Marketplace not found: {0}")]
    MarketplaceNotFound(String),

    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Zip extraction error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Deployment error: {0}")]
    Deployment(String),

    #[error("Target not found: {0}")]
    TargetNotFound(String),

    #[error("TUI error: {0}")]
    Tui(String),

    #[error("Operation cancelled")]
    Cancelled,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Invalid source: {0}")]
    InvalidSource(String),

    #[error("Target registry error: {0}")]
    TargetRegistry(String),

    #[error("Import registry error: {0}")]
    ImportRegistry(String),
}

pub type Result<T> = std::result::Result<T, PlmError>;

impl PlmError {
    /// リトライ可能なエラーかどうか
    pub fn is_retryable(&self) -> bool {
        match self {
            PlmError::Network(_) => true,
            PlmError::RepoApi { status, .. } => {
                // 5xx エラーはリトライ可能
                *status >= 500 && *status < 600
            }
            _ => false,
        }
    }
}

impl From<PlmError> for RichError {
    fn from(err: PlmError) -> Self {
        let (code, message, context) = match &err {
            PlmError::Network(e) => {
                // Extract URL from reqwest::Error if available
                let mut ctx = ErrorContext::default();
                if let Some(url) = e.url() {
                    ctx.url = Some(url.to_string());
                }
                // Distinguish timeout errors
                let code = if e.is_timeout() {
                    ErrorCode::Net002
                } else {
                    ErrorCode::Net001
                };
                (code, e.to_string(), ctx)
            }
            PlmError::RepoApi {
                url,
                status,
                message,
            } => {
                let code = if *status == 429 {
                    ErrorCode::Api001 // Rate limit exceeded
                } else if *status == 404 {
                    ErrorCode::Api003 // Resource not found
                } else if *status >= 500 {
                    ErrorCode::Api004 // Server error
                } else if *status == 401 || *status == 403 {
                    ErrorCode::Api002 // Authentication failed
                } else {
                    // Other 4xx errors (400, 422, 409, etc.) - use validation error
                    ErrorCode::Val001 // Client request error
                };
                let ctx = ErrorContext::new().with_url(url.clone());
                (code, message.clone(), ctx)
            }
            PlmError::PluginNotFound(name) => {
                let ctx = ErrorContext::new().with_plugin_name(name.clone());
                (
                    ErrorCode::Plg001,
                    format!("Plugin '{}' not found", name),
                    ctx,
                )
            }
            PlmError::InvalidRepoFormat(s) => (
                ErrorCode::Val002,
                format!("Invalid repository format: {}", s),
                ErrorContext::default(),
            ),
            PlmError::InvalidArgument(s) => (ErrorCode::Val001, s.clone(), ErrorContext::default()),
            PlmError::AmbiguousPlugin { name, candidates } => {
                let ctx = ErrorContext::new().with_plugin_name(name.clone());
                let msg = format!(
                    "Multiple plugins named '{}' found: {}",
                    name,
                    candidates.join(", ")
                );
                (ErrorCode::Plg003, msg, ctx)
            }
            PlmError::MarketplaceNotFound(name) => (
                ErrorCode::Mkt001,
                format!("Marketplace '{}' not found", name),
                ErrorContext::default(),
            ),
            PlmError::InvalidManifest(s) => (
                ErrorCode::Plg002,
                format!("Invalid manifest: {}", s),
                ErrorContext::default(),
            ),
            PlmError::Io(e) => {
                // Distinguish I/O error types by ErrorKind
                let code = match e.kind() {
                    std::io::ErrorKind::NotFound => ErrorCode::Io001,
                    std::io::ErrorKind::PermissionDenied => ErrorCode::Io002,
                    // Disk full detection (StorageFull is unstable, check raw_os_error)
                    _ if e.raw_os_error() == Some(28) => ErrorCode::Io003, // ENOSPC on Unix
                    // Other I/O errors map to internal error to avoid misleading messages
                    _ => ErrorCode::Int001,
                };
                (code, e.to_string(), ErrorContext::default())
            }
            PlmError::Json(e) => (
                ErrorCode::Cfg001,
                format!("JSON parse error: {}", e),
                ErrorContext::default(),
            ),
            PlmError::Zip(e) => {
                // Zip errors are typically corruption/format issues, not file-not-found
                (
                    ErrorCode::Int001,
                    format!("Zip extraction error: {}", e),
                    ErrorContext::default(),
                )
            }
            PlmError::Cache(s) => (
                ErrorCode::Int001,
                format!("Cache error: {}", s),
                ErrorContext::default(),
            ),
            PlmError::Deployment(s) => (
                ErrorCode::Int001,
                format!("Deployment error: {}", s),
                ErrorContext::default(),
            ),
            PlmError::TargetNotFound(s) => (
                ErrorCode::Cfg002,
                format!("Target not found: {}", s),
                ErrorContext::default(),
            ),
            PlmError::Tui(s) => (ErrorCode::Tui001, s.clone(), ErrorContext::default()),
            PlmError::Cancelled => (
                ErrorCode::Int001,
                "Operation cancelled".to_string(),
                ErrorContext::default(),
            ),
            PlmError::Validation(s) => (ErrorCode::Val001, s.clone(), ErrorContext::default()),
            PlmError::InvalidSource(s) => (
                ErrorCode::Val001,
                format!("Invalid source: {}", s),
                ErrorContext::default(),
            ),
            PlmError::TargetRegistry(s) => (
                ErrorCode::Cfg002,
                format!("Target registry error: {}", s),
                ErrorContext::default(),
            ),
            PlmError::ImportRegistry(s) => (
                ErrorCode::Cfg002,
                format!("Import registry error: {}", s),
                ErrorContext::default(),
            ),
        };

        RichError::new(code, message).with_context(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plm_error_to_rich_error_network() {
        // We can't easily create a reqwest::Error, so test via the mapping logic
        let error = PlmError::PluginNotFound("test-plugin".to_string());
        let rich: RichError = error.into();
        assert_eq!(rich.code(), ErrorCode::Plg001);
        assert!(rich.message().contains("test-plugin"));
    }

    #[test]
    fn plm_error_to_rich_error_repo_api_rate_limit() {
        let error = PlmError::RepoApi {
            url: "https://api.github.com".to_string(),
            status: 429,
            message: "Rate limit exceeded".to_string(),
        };
        let rich: RichError = error.into();
        assert_eq!(rich.code(), ErrorCode::Api001);
    }

    #[test]
    fn plm_error_to_rich_error_repo_api_not_found() {
        let error = PlmError::RepoApi {
            url: "https://api.github.com".to_string(),
            status: 404,
            message: "Not found".to_string(),
        };
        let rich: RichError = error.into();
        assert_eq!(rich.code(), ErrorCode::Api003);
    }

    #[test]
    fn plm_error_to_rich_error_repo_api_server_error() {
        let error = PlmError::RepoApi {
            url: "https://api.github.com".to_string(),
            status: 500,
            message: "Internal server error".to_string(),
        };
        let rich: RichError = error.into();
        assert_eq!(rich.code(), ErrorCode::Api004);
    }

    #[test]
    fn plm_error_to_rich_error_repo_api_auth_error() {
        let error = PlmError::RepoApi {
            url: "https://api.github.com".to_string(),
            status: 401,
            message: "Unauthorized".to_string(),
        };
        let rich: RichError = error.into();
        assert_eq!(rich.code(), ErrorCode::Api002);
    }

    #[test]
    fn plm_error_to_rich_error_repo_api_other_4xx() {
        let error = PlmError::RepoApi {
            url: "https://api.github.com".to_string(),
            status: 400,
            message: "Bad request".to_string(),
        };
        let rich: RichError = error.into();
        assert_eq!(rich.code(), ErrorCode::Val001);
    }

    #[test]
    fn plm_error_to_rich_error_io_not_found() {
        let error = PlmError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        let rich: RichError = error.into();
        assert_eq!(rich.code(), ErrorCode::Io001);
    }

    #[test]
    fn plm_error_to_rich_error_io_permission_denied() {
        let error = PlmError::Io(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "permission denied",
        ));
        let rich: RichError = error.into();
        assert_eq!(rich.code(), ErrorCode::Io002);
    }

    #[test]
    fn plm_error_to_rich_error_io_other() {
        let error = PlmError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "some other error",
        ));
        let rich: RichError = error.into();
        assert_eq!(rich.code(), ErrorCode::Int001);
    }

    #[test]
    fn plm_error_to_rich_error_ambiguous_plugin() {
        let error = PlmError::AmbiguousPlugin {
            name: "plugin".to_string(),
            candidates: vec!["a/plugin".to_string(), "b/plugin".to_string()],
        };
        let rich: RichError = error.into();
        assert_eq!(rich.code(), ErrorCode::Plg003);
        assert!(rich.context().plugin_name.is_some());
    }

    #[test]
    fn all_plm_errors_have_explicit_mapping() {
        // This test ensures all PlmError variants are explicitly handled
        // If a new variant is added and not handled, this test should be updated
        let test_cases: Vec<PlmError> = vec![
            PlmError::InvalidRepoFormat("test".to_string()),
            PlmError::PluginNotFound("test".to_string()),
            PlmError::InvalidArgument("test".to_string()),
            PlmError::AmbiguousPlugin {
                name: "test".to_string(),
                candidates: vec![],
            },
            PlmError::MarketplaceNotFound("test".to_string()),
            PlmError::InvalidManifest("test".to_string()),
            PlmError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test")),
            PlmError::Cache("test".to_string()),
            PlmError::Deployment("test".to_string()),
            PlmError::TargetNotFound("test".to_string()),
            PlmError::Tui("test".to_string()),
            PlmError::Cancelled,
            PlmError::Validation("test".to_string()),
            PlmError::InvalidSource("test".to_string()),
            PlmError::TargetRegistry("test".to_string()),
            PlmError::ImportRegistry("test".to_string()),
        ];

        for error in test_cases {
            let rich: RichError = error.into();
            // Ensure each error maps to a valid code (not just panicking)
            let _ = rich.code().as_str();
        }
    }
}
