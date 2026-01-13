//! 共通HTTPヘルパー

use crate::error::{PlmError, Result};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::future::Future;
use std::time::Duration;

/// プログレスバー付きダウンロード
pub async fn download_with_progress(
    client: &Client,
    url: &str,
    auth_header: Option<(&str, String)>,
) -> Result<Vec<u8>> {
    download_with_progress_impl(client, url, auth_header, "unknown").await
}

/// ホスト名付きプログレスバーダウンロード
pub async fn download_with_progress_and_host(
    client: &Client,
    url: &str,
    auth_header: Option<(&str, String)>,
    host: &str,
) -> Result<Vec<u8>> {
    download_with_progress_impl(client, url, auth_header, host).await
}

/// プログレスバー付きダウンロード実装
async fn download_with_progress_impl(
    client: &Client,
    url: &str,
    auth_header: Option<(&str, String)>,
    host: &str,
) -> Result<Vec<u8>> {
    let mut req = client.get(url).header("User-Agent", "plm-cli");

    if let Some((name, value)) = auth_header {
        req = req.header(name, value);
    }

    let response = req.send().await?;
    let status = response.status().as_u16();

    if !response.status().is_success() {
        let message = response.text().await.unwrap_or_default();
        return Err(PlmError::RepoApi {
            host: host.to_string(),
            status,
            message,
        });
    }

    let total_size = response.content_length().unwrap_or(0);

    let pb = create_progress_bar(total_size);
    let bytes = response.bytes().await?;
    pb.finish_and_clear();

    Ok(bytes.to_vec())
}

/// サイズに応じたプログレスバーを作成
fn create_progress_bar(total_size: u64) -> ProgressBar {
    if total_size > 0 {
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb
    } else {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} Downloading...")
                .unwrap(),
        );
        pb
    }
}

// =========================================================================
// リトライ機能
// =========================================================================

/// リトライ可能なエラーかどうかを判定
///
/// PlmError::is_retryable の判定に加え、レート制限エラー（429, 403 rate limit）も対象とする。
pub fn is_retriable_error(e: &PlmError) -> bool {
    if e.is_retryable() {
        return true;
    }

    // レート制限エラーの追加判定
    match e {
        PlmError::RepoApi { status: 429, .. } => true,
        PlmError::RepoApi {
            status: 403,
            message,
            ..
        } => message.to_lowercase().contains("rate limit"),
        _ => false,
    }
}

/// リトライ付きで非同期処理を実行
///
/// 指数バックオフ（1s, 2s, 4s）で最大 `max_retries` 回再試行する。
/// 初回 + max_retries 回 = 最大 (max_retries + 1) 回試行。
pub async fn with_retry<F, Fut, T>(mut f: F, max_retries: u32) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut last_error: Option<PlmError> = None;

    for attempt in 0..=max_retries {
        // リトライ時は待機
        if attempt > 0 {
            let delay = Duration::from_secs(1 << (attempt - 1));
            warn_if_rate_limited(&last_error, delay.as_secs(), attempt, max_retries);
            tokio::time::sleep(delay).await;
        }

        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                let can_retry = is_retriable_error(&e) && attempt < max_retries;
                if can_retry {
                    last_error = Some(e);
                } else {
                    return Err(e);
                }
            }
        }
    }

    // ループ終了時は最後のエラーを返す（理論上到達しない）
    Err(last_error.expect("retry loop should have returned"))
}

/// レート制限時の警告表示
fn warn_if_rate_limited(last_error: &Option<PlmError>, delay_secs: u64, attempt: u32, max: u32) {
    if let Some(PlmError::RepoApi { status, .. }) = last_error {
        if *status == 403 || *status == 429 {
            eprintln!(
                "Warning: Rate limited. Waiting {}s before retry ({}/{})...",
                delay_secs, attempt, max
            );
        }
    }
}

#[cfg(test)]
#[path = "http_test.rs"]
mod tests;
