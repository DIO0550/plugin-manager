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
