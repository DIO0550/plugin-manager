//! HTTP設定と認証プロバイダー

use reqwest::Client;
use std::time::Duration;

/// HTTP設定
#[derive(Debug, Clone)]
pub struct HttpConfig {
    /// タイムアウト（秒）
    pub timeout: Option<Duration>,
    /// User-Agent
    pub user_agent: String,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            timeout: Some(Duration::from_secs(30)),
            user_agent: "plm-cli".to_string(),
        }
    }
}

impl HttpConfig {
    /// reqwest::Client を構築
    pub fn build_client(&self) -> Client {
        let mut builder = Client::builder().user_agent(&self.user_agent);

        if let Some(timeout) = self.timeout {
            builder = builder.timeout(timeout);
        }

        builder.build().unwrap_or_else(|_| Client::new())
    }
}

/// 認証プロバイダー
///
/// トークンの取得元を管理する。
/// 優先順位: 明示的なトークン > 環境変数 > CLI
#[derive(Debug, Clone, Default)]
pub struct AuthProvider {
    /// 明示的に設定されたトークン（ホスト別）
    github_token: Option<String>,
    gitlab_token: Option<String>,
    bitbucket_token: Option<String>,
}

impl AuthProvider {
    /// 新しいAuthProviderを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// GitHubトークンを設定
    pub fn with_github_token(mut self, token: impl Into<String>) -> Self {
        self.github_token = Some(token.into());
        self
    }

    /// GitLabトークンを設定
    pub fn with_gitlab_token(mut self, token: impl Into<String>) -> Self {
        self.gitlab_token = Some(token.into());
        self
    }

    /// Bitbucketトークンを設定
    pub fn with_bitbucket_token(mut self, token: impl Into<String>) -> Self {
        self.bitbucket_token = Some(token.into());
        self
    }

    /// GitHubトークンを取得
    pub fn github_token(&self) -> Option<&str> {
        self.github_token.as_deref()
    }

    /// GitLabトークンを取得
    pub fn gitlab_token(&self) -> Option<&str> {
        self.gitlab_token.as_deref()
    }

    /// Bitbucketトークンを取得
    pub fn bitbucket_token(&self) -> Option<&str> {
        self.bitbucket_token.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_config_default() {
        let config = HttpConfig::default();
        assert_eq!(config.user_agent, "plm-cli");
        assert!(config.timeout.is_some());
    }

    #[test]
    fn test_auth_provider_builder() {
        let auth = AuthProvider::new()
            .with_github_token("gh_token")
            .with_gitlab_token("gl_token");

        assert_eq!(auth.github_token(), Some("gh_token"));
        assert_eq!(auth.gitlab_token(), Some("gl_token"));
        assert_eq!(auth.bitbucket_token(), None);
    }
}
