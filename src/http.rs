//! 共通HTTPヘルパー

use crate::error::{PlmError, Result};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;

/// プログレスバー付きダウンロード
pub async fn download_with_progress(
    client: &Client,
    url: &str,
    auth_header: Option<(&str, String)>,
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
            host: "unknown".to_string(),
            status,
            message,
        });
    }

    let total_size = response.content_length().unwrap_or(0);

    let pb = if total_size > 0 {
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );
        Some(pb)
    } else {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} Downloading...")
                .unwrap(),
        );
        Some(pb)
    };

    let bytes = response.bytes().await?;

    if let Some(pb) = pb {
        pb.finish_and_clear();
    }

    Ok(bytes.to_vec())
}

/// ホスト名付きプログレスバーダウンロード
pub async fn download_with_progress_and_host(
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

    let pb = if total_size > 0 {
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );
        Some(pb)
    } else {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} Downloading...")
                .unwrap(),
        );
        Some(pb)
    };

    let bytes = response.bytes().await?;

    if let Some(pb) = pb {
        pb.finish_and_clear();
    }

    Ok(bytes.to_vec())
}
