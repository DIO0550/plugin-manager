use super::*;

// =========================================================================
// is_retriable_error tests
// =========================================================================

#[test]
fn test_is_retriable_error_429() {
    let error = PlmError::RepoApi {
        host: "github.com".to_string(),
        status: 429,
        message: "Too Many Requests".to_string(),
    };
    assert!(is_retriable_error(&error));
}

#[test]
fn test_is_retriable_error_403_rate_limit() {
    let error = PlmError::RepoApi {
        host: "github.com".to_string(),
        status: 403,
        message: "API rate limit exceeded".to_string(),
    };
    assert!(is_retriable_error(&error));
}

#[test]
fn test_is_retriable_error_403_rate_limit_case_insensitive() {
    let error = PlmError::RepoApi {
        host: "github.com".to_string(),
        status: 403,
        message: "RATE LIMIT exceeded".to_string(),
    };
    assert!(is_retriable_error(&error));
}

#[test]
fn test_is_retriable_error_403_not_rate_limit() {
    let error = PlmError::RepoApi {
        host: "github.com".to_string(),
        status: 403,
        message: "Forbidden".to_string(),
    };
    assert!(!is_retriable_error(&error));
}

#[test]
fn test_is_retriable_error_500() {
    let error = PlmError::RepoApi {
        host: "github.com".to_string(),
        status: 500,
        message: "Internal Server Error".to_string(),
    };
    // 500 is retryable via PlmError::is_retryable()
    assert!(is_retriable_error(&error));
}

#[test]
fn test_is_retriable_error_502() {
    let error = PlmError::RepoApi {
        host: "github.com".to_string(),
        status: 502,
        message: "Bad Gateway".to_string(),
    };
    assert!(is_retriable_error(&error));
}

#[test]
fn test_is_retriable_error_503() {
    let error = PlmError::RepoApi {
        host: "github.com".to_string(),
        status: 503,
        message: "Service Unavailable".to_string(),
    };
    assert!(is_retriable_error(&error));
}

#[test]
fn test_is_retriable_error_404() {
    let error = PlmError::RepoApi {
        host: "github.com".to_string(),
        status: 404,
        message: "Not Found".to_string(),
    };
    assert!(!is_retriable_error(&error));
}

#[test]
fn test_is_retriable_error_401() {
    let error = PlmError::RepoApi {
        host: "github.com".to_string(),
        status: 401,
        message: "Unauthorized".to_string(),
    };
    assert!(!is_retriable_error(&error));
}

// =========================================================================
// with_retry tests
// =========================================================================

#[tokio::test]
async fn test_with_retry_success_first_try() {
    let mut call_count = 0;
    let result = with_retry(
        || {
            call_count += 1;
            async { Ok::<_, PlmError>(42) }
        },
        3,
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    assert_eq!(call_count, 1);
}

#[tokio::test]
async fn test_with_retry_success_after_retries() {
    let mut call_count = 0;
    let result = with_retry(
        || {
            call_count += 1;
            async move {
                if call_count < 3 {
                    Err(PlmError::RepoApi {
                        host: "test".to_string(),
                        status: 500,
                        message: "error".to_string(),
                    })
                } else {
                    Ok(42)
                }
            }
        },
        3,
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    assert_eq!(call_count, 3);
}

#[tokio::test]
async fn test_with_retry_fails_after_max_retries() {
    let mut call_count = 0;
    let result: Result<i32> = with_retry(
        || {
            call_count += 1;
            async {
                Err(PlmError::RepoApi {
                    host: "test".to_string(),
                    status: 500,
                    message: "always fails".to_string(),
                })
            }
        },
        2,
    )
    .await;

    assert!(result.is_err());
    // 初回 + 2回リトライ = 3回
    assert_eq!(call_count, 3);
}

#[tokio::test]
async fn test_with_retry_non_retriable_error_fails_immediately() {
    let mut call_count = 0;
    let result: Result<i32> = with_retry(
        || {
            call_count += 1;
            async {
                Err(PlmError::RepoApi {
                    host: "test".to_string(),
                    status: 404,
                    message: "not found".to_string(),
                })
            }
        },
        3,
    )
    .await;

    assert!(result.is_err());
    // 404 はリトライ不可なので1回で終了
    assert_eq!(call_count, 1);
}
