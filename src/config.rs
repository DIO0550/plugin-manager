//! HTTP設定と認証プロバイダー

use crate::env::EnvVar;
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

        // 保険: 環境変数から CA を明示追加する。
        // SSL_CERT_FILE は rustls-native-certs でも読まれるが、CODEX_PROXY_CERT 単独設定の
        // ケースと、ロード失敗時の警告可視化のために明示追加する。
        let ssl_cert_path = EnvVar::get("SSL_CERT_FILE");
        let codex_cert_path = EnvVar::get("CODEX_PROXY_CERT");

        let mut added_paths: Vec<String> = Vec::new();

        if let Some(ref path) = ssl_cert_path {
            builder = Self::add_cert_from_path(builder, path);
            added_paths.push(path.clone());
        }

        if let Some(ref path) = codex_cert_path {
            if !added_paths.contains(path) {
                builder = Self::add_cert_from_path(builder, path);
            }
        }

        builder.build().unwrap_or_else(|e| {
            eprintln!(
                "[plm warn] failed to build HTTP client with custom settings: {}; falling back to defaults",
                e
            );
            Client::new()
        })
    }

    /// PEM ファイル（単一またはバンドル）から証明書を読み込んで ClientBuilder に追加する。
    /// 読み込み失敗・パース失敗は eprintln! で警告してそのまま builder を返す。
    fn add_cert_from_path(builder: reqwest::ClientBuilder, path: &str) -> reqwest::ClientBuilder {
        match std::fs::read(path) {
            Err(e) => {
                eprintln!(
                    "[plm warn] cannot read CA certificate file '{}': {}",
                    path, e
                );
                builder
            }
            Ok(pem_bytes) => match reqwest::Certificate::from_pem_bundle(&pem_bytes) {
                Err(e) => {
                    eprintln!("[plm warn] invalid PEM in '{}': {}", path, e);
                    builder
                }
                Ok(certs) => {
                    let mut builder = builder;
                    for cert in certs {
                        builder = builder.add_root_certificate(cert);
                    }
                    builder
                }
            },
        }
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
    ///
    /// # Arguments
    ///
    /// * `token` - GitHub personal access token or installation token.
    pub fn with_github_token(mut self, token: impl Into<String>) -> Self {
        self.github_token = Some(token.into());
        self
    }

    /// GitLabトークンを設定
    ///
    /// # Arguments
    ///
    /// * `token` - GitLab personal or project access token.
    pub fn with_gitlab_token(mut self, token: impl Into<String>) -> Self {
        self.gitlab_token = Some(token.into());
        self
    }

    /// Bitbucketトークンを設定
    ///
    /// # Arguments
    ///
    /// * `token` - Bitbucket app password or access token.
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
#[path = "config_test.rs"]
mod tests;
