use thiserror::Error;

/// AmbiguousPlugin エラーのフォーマット
fn format_ambiguous_plugin(name: &str, candidates: &[String]) -> String {
    let mut msg = format!("multiple plugins named '{}' found:\n", name);
    for c in candidates {
        msg.push_str(&format!("  - {}\n", c));
    }
    msg.push_str(&format!("Use 'plm info <marketplace>/{}' to specify.", name));
    msg
}

/// PLM統一エラー型
#[derive(Debug, Error)]
pub enum PlmError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("{host} API error: {message} (status: {status})")]
    RepoApi {
        host: String,
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
    AmbiguousPlugin { name: String, candidates: Vec<String> },

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
