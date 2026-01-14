//! リモートバージョン取得
//!
//! GitHub APIを使用して最新のコミットSHAを取得する。
//! `plm update` と `plm list --outdated` の両方で使用される。

use crate::error::PlmError;
use crate::host::{HostClient, HostKind};
use crate::plugin::PluginMeta;
use crate::repo::Repo;
use serde::Serialize;

/// リモート（GitHub等）から取得したバージョン情報
#[derive(Debug, Clone, Serialize)]
pub struct RemoteVersion {
    /// コミットSHA
    pub sha: String,
    /// ブランチ/タグ名
    pub git_ref: String,
}

/// リモートバージョン取得の結果
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum VersionQueryResult {
    /// 取得成功
    Found(RemoteVersion),
    /// 取得失敗
    Failed {
        /// エラーメッセージ
        message: String,
    },
}

impl VersionQueryResult {
    /// 取得成功かどうか
    pub fn is_found(&self) -> bool {
        matches!(self, Self::Found(_))
    }

    /// 取得失敗かどうか
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }

    /// 成功時のRemoteVersionを取得
    pub fn as_found(&self) -> Option<&RemoteVersion> {
        match self {
            Self::Found(v) => Some(v),
            Self::Failed { .. } => None,
        }
    }

    /// 失敗時のエラーメッセージを取得
    pub fn error_message(&self) -> Option<&str> {
        match self {
            Self::Found(_) => None,
            Self::Failed { message } => Some(message),
        }
    }
}

/// ローカルとリモートのバージョンを比較し、更新が必要かどうかを判定
///
/// - ローカルSHAがNoneの場合: 更新が必要（インストール時にSHAが記録されていない）
/// - ローカルSHAとリモートSHAが異なる場合: 更新が必要
/// - ローカルSHAとリモートSHAが同じ場合: 更新不要
pub fn needs_update(local_sha: Option<&str>, remote_sha: &str) -> bool {
    match local_sha {
        Some(local) => local != remote_sha,
        None => true,
    }
}

/// PlmErrorからエラーメッセージを生成
fn error_message(error: &PlmError) -> String {
    match error {
        PlmError::RepoApi {
            status: 403,
            message,
            ..
        } => {
            let msg_lower = message.to_lowercase();
            if msg_lower.contains("rate limit") || msg_lower.contains("ratelimit") {
                "Rate limited".to_string()
            } else {
                "Access denied".to_string()
            }
        }
        PlmError::RepoApi { status: 404, .. } => "Repository or ref not found".to_string(),
        PlmError::Network(_) => "Network error".to_string(),
        _ => error.to_string(),
    }
}

/// 単一プラグインのリモートバージョンを取得
///
/// 1. `meta.git_ref` を取得（未記録時は `client.get_default_branch()` を使用）
/// 2. `client.get_commit_sha()` で最新 SHA を取得
pub async fn fetch_remote_version(
    meta: &PluginMeta,
    client: &dyn HostClient,
) -> VersionQueryResult {
    // リポジトリ情報がなければエラー
    let (owner, name) = match meta.get_source_repo() {
        Some(repo) => repo,
        None => {
            return VersionQueryResult::Failed {
                message: "No repository info".to_string(),
            }
        }
    };

    // git_ref を取得（未記録時はデフォルトブランチを取得）
    let repo_for_default = Repo::new(HostKind::GitHub, owner, name, None);
    let git_ref = match &meta.git_ref {
        Some(r) => r.clone(),
        None => match client.get_default_branch(&repo_for_default).await {
            Ok(branch) => branch,
            Err(e) => {
                return VersionQueryResult::Failed {
                    message: error_message(&e),
                }
            }
        },
    };

    // リポジトリ情報を構築
    let repo = Repo::new(HostKind::GitHub, owner, name, Some(git_ref.clone()));

    // 最新 SHA を取得
    match client.get_commit_sha(&repo, &git_ref).await {
        Ok(sha) => VersionQueryResult::Found(RemoteVersion { sha, git_ref }),
        Err(e) => VersionQueryResult::Failed {
            message: error_message(&e),
        },
    }
}

/// 複数プラグインのリモートバージョンを一括取得
///
/// 各プラグインに対して `fetch_remote_version()` を呼び出し、結果を集約する。
/// エラーが発生しても後続の処理を継続する。
pub async fn fetch_remote_versions(
    plugins: &[(String, PluginMeta)],
    client: &dyn HostClient,
) -> Vec<(String, VersionQueryResult)> {
    let mut results = Vec::with_capacity(plugins.len());

    for (name, meta) in plugins {
        let result = fetch_remote_version(meta, client).await;
        results.push((name.clone(), result));
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;
    use std::future::Future;
    use std::pin::Pin;

    /// モック用の結果型
    enum MockResult {
        Ok(String),
        Err { status: u16, message: String },
    }

    /// テスト用モッククライアント
    struct MockHostClient {
        default_branch: String,
        commit_sha_result: MockResult,
    }

    impl MockHostClient {
        fn success(sha: &str) -> Self {
            Self {
                default_branch: "main".to_string(),
                commit_sha_result: MockResult::Ok(sha.to_string()),
            }
        }

        fn with_error(status: u16, message: &str) -> Self {
            Self {
                default_branch: "main".to_string(),
                commit_sha_result: MockResult::Err {
                    status,
                    message: message.to_string(),
                },
            }
        }
    }

    impl HostClient for MockHostClient {
        fn get_default_branch<'a>(
            &'a self,
            _repo: &'a Repo,
        ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
            let branch = self.default_branch.clone();
            Box::pin(async move { Ok(branch) })
        }

        fn get_commit_sha<'a>(
            &'a self,
            _repo: &'a Repo,
            _git_ref: &'a str,
        ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
            let result = match &self.commit_sha_result {
                MockResult::Ok(sha) => Ok(sha.clone()),
                MockResult::Err { status, message } => Err(PlmError::RepoApi {
                    host: "github".to_string(),
                    status: *status,
                    message: message.clone(),
                }),
            };
            Box::pin(async move { result })
        }

        fn download_archive<'a>(
            &'a self,
            _repo: &'a Repo,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'a>> {
            Box::pin(async { Ok(vec![]) })
        }

        fn download_archive_with_sha<'a>(
            &'a self,
            _repo: &'a Repo,
        ) -> Pin<Box<dyn Future<Output = Result<(Vec<u8>, String, String)>> + Send + 'a>> {
            Box::pin(async { Ok((vec![], "main".to_string(), "abc123".to_string())) })
        }

        fn fetch_file<'a>(
            &'a self,
            _repo: &'a Repo,
            _path: &'a str,
        ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
            Box::pin(async { Ok(String::new()) })
        }
    }

    fn create_meta(
        source_repo: Option<&str>,
        git_ref: Option<&str>,
        commit_sha: Option<&str>,
    ) -> PluginMeta {
        PluginMeta {
            source_repo: source_repo.map(|s| s.to_string()),
            git_ref: git_ref.map(|s| s.to_string()),
            commit_sha: commit_sha.map(|s| s.to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_needs_update_different_sha() {
        assert!(needs_update(Some("old123"), "new456"));
    }

    #[test]
    fn test_needs_update_same_sha() {
        assert!(!needs_update(Some("same123"), "same123"));
    }

    #[test]
    fn test_needs_update_local_none() {
        assert!(needs_update(None, "new456"));
    }

    #[tokio::test]
    async fn test_fetch_remote_version_success() {
        let meta = create_meta(Some("owner/repo"), Some("main"), Some("old123"));
        let client = MockHostClient::success("new456");

        let result = fetch_remote_version(&meta, &client).await;

        assert!(result.is_found());
        let remote = result.as_found().unwrap();
        assert_eq!(remote.sha, "new456");
        assert_eq!(remote.git_ref, "main");
    }

    #[tokio::test]
    async fn test_fetch_remote_version_no_repo_info() {
        let meta = create_meta(None, Some("main"), Some("abc123"));
        let client = MockHostClient::success("def456");

        let result = fetch_remote_version(&meta, &client).await;

        assert!(result.is_failed());
        assert_eq!(result.error_message(), Some("No repository info"));
    }

    #[tokio::test]
    async fn test_fetch_remote_version_rate_limited() {
        let meta = create_meta(Some("owner/repo"), Some("main"), Some("abc123"));
        let client = MockHostClient::with_error(403, "API rate limit exceeded");

        let result = fetch_remote_version(&meta, &client).await;

        assert!(result.is_failed());
        assert_eq!(result.error_message(), Some("Rate limited"));
    }

    #[tokio::test]
    async fn test_fetch_remote_version_not_found() {
        let meta = create_meta(Some("owner/repo"), Some("main"), Some("abc123"));
        let client = MockHostClient::with_error(404, "Not Found");

        let result = fetch_remote_version(&meta, &client).await;

        assert!(result.is_failed());
        assert_eq!(result.error_message(), Some("Repository or ref not found"));
    }

    #[tokio::test]
    async fn test_fetch_remote_versions_batch() {
        let plugins = vec![
            (
                "plugin1".to_string(),
                create_meta(Some("owner/repo1"), Some("main"), Some("old1")),
            ),
            (
                "plugin2".to_string(),
                create_meta(Some("owner/repo2"), Some("main"), Some("old2")),
            ),
        ];

        let client = MockHostClient::success("new123");

        let results = fetch_remote_versions(&plugins, &client).await;

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "plugin1");
        assert_eq!(results[1].0, "plugin2");
        assert!(results[0].1.is_found());
        assert!(results[1].1.is_found());
    }
}
