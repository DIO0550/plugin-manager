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
                url: "https://api.github.com/test".to_string(),
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

// ========================================
// UpgradeState unit tests
// ========================================

#[test]
fn test_update_check_has_update_available() {
    let check = UpgradeState::Outdated {
        current_sha: Some("abc".to_string()),
        latest_sha: "def".to_string(),
    };
    assert!(check.has_update());
    assert!(!check.is_unknown());
}

#[test]
fn test_update_check_has_update_up_to_date() {
    let check = UpgradeState::Latest {
        current_sha: Some("same".to_string()),
        latest_sha: "same".to_string(),
    };
    assert!(!check.has_update());
    assert!(!check.is_unknown());
}

#[test]
fn test_update_check_has_update_failed() {
    let check = UpgradeState::Unknown {
        current_sha: None,
        error: "network error".to_string(),
    };
    assert!(!check.has_update());
    assert!(check.is_unknown());
}

#[test]
fn test_update_check_accessors_up_to_date() {
    let check = UpgradeState::Latest {
        current_sha: Some("sha1".to_string()),
        latest_sha: "sha1".to_string(),
    };
    assert_eq!(check.current_sha(), Some("sha1"));
    assert_eq!(check.latest_sha(), Some("sha1"));
    assert_eq!(check.error(), None);
}

#[test]
fn test_update_check_accessors_available() {
    let check = UpgradeState::Outdated {
        current_sha: None,
        latest_sha: "new".to_string(),
    };
    assert_eq!(check.current_sha(), None);
    assert_eq!(check.latest_sha(), Some("new"));
    assert_eq!(check.error(), None);
}

#[test]
fn test_update_check_accessors_failed() {
    let check = UpgradeState::Unknown {
        current_sha: Some("local".to_string()),
        error: "boom".to_string(),
    };
    assert_eq!(check.current_sha(), Some("local"));
    assert_eq!(check.latest_sha(), None);
    assert_eq!(check.error(), Some("boom"));
}

#[test]
fn test_update_check_from_query_found_same_sha() {
    let meta = create_meta(Some("owner/repo"), Some("main"), Some("same123"));
    let result = VersionQueryResult::Found(RemoteVersion {
        sha: "same123".to_string(),
        git_ref: "main".to_string(),
    });
    let check = UpgradeState::from_query(&meta, &result);
    assert!(matches!(check, UpgradeState::Latest { .. }));
    assert_eq!(check.current_sha(), Some("same123"));
    assert_eq!(check.latest_sha(), Some("same123"));
}

#[test]
fn test_update_check_from_query_found_different_sha() {
    let meta = create_meta(Some("owner/repo"), Some("main"), Some("old123"));
    let result = VersionQueryResult::Found(RemoteVersion {
        sha: "new456".to_string(),
        git_ref: "main".to_string(),
    });
    let check = UpgradeState::from_query(&meta, &result);
    assert!(matches!(check, UpgradeState::Outdated { .. }));
    assert!(check.has_update());
    assert_eq!(check.current_sha(), Some("old123"));
    assert_eq!(check.latest_sha(), Some("new456"));
}

#[test]
fn test_update_check_from_query_found_current_none_yields_available() {
    // current_sha が未記録の場合は Available バリアント（SHA 不一致扱い）
    let meta = create_meta(Some("owner/repo"), Some("main"), None);
    let result = VersionQueryResult::Found(RemoteVersion {
        sha: "new456".to_string(),
        git_ref: "main".to_string(),
    });
    let check = UpgradeState::from_query(&meta, &result);
    assert!(matches!(check, UpgradeState::Outdated { .. }));
    assert!(check.has_update());
    assert_eq!(check.current_sha(), None);
}

#[test]
fn test_update_check_from_query_failed() {
    let meta = create_meta(Some("owner/repo"), Some("main"), Some("local123"));
    let result = VersionQueryResult::Failed {
        message: "Rate limited".to_string(),
    };
    let check = UpgradeState::from_query(&meta, &result);
    assert!(matches!(check, UpgradeState::Unknown { .. }));
    assert!(check.is_unknown());
    assert_eq!(check.current_sha(), Some("local123"));
    assert_eq!(check.error(), Some("Rate limited"));
}

#[test]
fn test_update_check_serde_tag_available() {
    let check = UpgradeState::Outdated {
        current_sha: Some("abc".to_string()),
        latest_sha: "def".to_string(),
    };
    let value: serde_json::Value = serde_json::to_value(&check).unwrap();
    assert_eq!(value["status"], "outdated");
    assert_eq!(value["current_sha"], "abc");
    assert_eq!(value["latest_sha"], "def");
}

#[test]
fn test_update_check_serde_tag_latest() {
    let check = UpgradeState::Latest {
        current_sha: Some("same".to_string()),
        latest_sha: "same".to_string(),
    };
    let value: serde_json::Value = serde_json::to_value(&check).unwrap();
    assert_eq!(value["status"], "latest");
}

#[test]
fn test_update_check_serde_tag_unknown() {
    let check = UpgradeState::Unknown {
        current_sha: None,
        error: "boom".to_string(),
    };
    let value: serde_json::Value = serde_json::to_value(&check).unwrap();
    assert_eq!(value["status"], "unknown");
    assert_eq!(value["error"], "boom");
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
