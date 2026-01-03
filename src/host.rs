//! ホスト別クライアント
//!
//! GitHub, GitLab, Bitbucket 等のホスティングサービス用クライアント。

pub mod github;

pub use github::GitHubClient;

use crate::config::{AuthProvider, HttpConfig};
use crate::error::Result;
use crate::repo::Repo;
use std::future::Future;
use std::pin::Pin;

/// ホスト種別
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HostKind {
    GitHub,
    GitLab,
    Bitbucket,
}

impl HostKind {
    /// ホスト名を返す
    pub fn as_str(&self) -> &'static str {
        match self {
            HostKind::GitHub => "github",
            HostKind::GitLab => "gitlab",
            HostKind::Bitbucket => "bitbucket",
        }
    }
}

impl std::fmt::Display for HostKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// ホスト別クライアント trait
pub trait HostClient: Send + Sync {
    /// デフォルトブランチを取得
    fn get_default_branch<'a>(
        &'a self,
        repo: &'a Repo,
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>>;

    /// コミットSHAを取得
    fn get_commit_sha<'a>(
        &'a self,
        repo: &'a Repo,
        git_ref: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>>;

    /// リポジトリをzipアーカイブとしてダウンロード
    fn download_archive<'a>(
        &'a self,
        repo: &'a Repo,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'a>>;

    /// リポジトリをダウンロードし、コミットSHAも一緒に返す
    fn download_archive_with_sha<'a>(
        &'a self,
        repo: &'a Repo,
    ) -> Pin<Box<dyn Future<Output = Result<(Vec<u8>, String, String)>> + Send + 'a>>;

    /// リポジトリ内のファイルを取得
    fn fetch_file<'a>(
        &'a self,
        repo: &'a Repo,
        path: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>>;
}

/// ホストクライアントファクトリー
///
/// HTTP設定と認証プロバイダーを保持し、ホスト種別に応じたクライアントを生成する。
pub struct HostClientFactory {
    config: HttpConfig,
    auth: AuthProvider,
}

impl HostClientFactory {
    /// 新しいファクトリーを作成
    pub fn new(config: HttpConfig, auth: AuthProvider) -> Self {
        Self { config, auth }
    }

    /// デフォルト設定でファクトリーを作成
    pub fn with_defaults() -> Self {
        Self::new(HttpConfig::default(), AuthProvider::new())
    }

    /// ホスト種別に応じたクライアントを生成
    pub fn create(&self, host: HostKind) -> Box<dyn HostClient> {
        match host {
            HostKind::GitHub => Box::new(GitHubClient::new(&self.config, &self.auth)),
            HostKind::GitLab => {
                // TODO: GitLabClient実装後に置き換え
                panic!("GitLab is not yet supported")
            }
            HostKind::Bitbucket => {
                // TODO: BitbucketClient実装後に置き換え
                panic!("Bitbucket is not yet supported")
            }
        }
    }
}

impl Default for HostClientFactory {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_kind_as_str() {
        assert_eq!(HostKind::GitHub.as_str(), "github");
        assert_eq!(HostKind::GitLab.as_str(), "gitlab");
        assert_eq!(HostKind::Bitbucket.as_str(), "bitbucket");
    }

    #[test]
    fn test_host_kind_display() {
        assert_eq!(format!("{}", HostKind::GitHub), "github");
    }

    #[test]
    fn test_factory_creation() {
        let factory = HostClientFactory::with_defaults();
        // GitHubクライアントは生成できる
        let _client = factory.create(HostKind::GitHub);
    }

    // === 境界値テスト ===

    #[test]
    #[should_panic(expected = "GitLab is not yet supported")]
    fn test_factory_gitlab_panics() {
        let factory = HostClientFactory::with_defaults();
        let _client = factory.create(HostKind::GitLab);
    }

    #[test]
    #[should_panic(expected = "Bitbucket is not yet supported")]
    fn test_factory_bitbucket_panics() {
        let factory = HostClientFactory::with_defaults();
        let _client = factory.create(HostKind::Bitbucket);
    }
}
