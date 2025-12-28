use thiserror::Error;

/// PLM統一エラー型
#[derive(Debug, Error)]
pub enum PlmError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("GitHub API error: {message} (status: {status})")]
    GitHubApi { status: u16, message: String },

    #[error("Invalid repository format: {0}. Expected 'owner/repo' or 'owner/repo@ref'")]
    InvalidRepoFormat(String),

    #[error("Plugin not found: {0}")]
    PluginNotFound(String),

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
}

pub type Result<T> = std::result::Result<T, PlmError>;

impl PlmError {
    /// リトライ可能なエラーかどうか
    pub fn is_retryable(&self) -> bool {
        match self {
            PlmError::Network(_) => true,
            PlmError::GitHubApi { status, .. } => {
                // 5xx エラーはリトライ可能
                *status >= 500 && *status < 600
            }
            _ => false,
        }
    }
}
