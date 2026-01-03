//! GitHub クライアント

use crate::config::{AuthProvider, HttpConfig};
use crate::env::EnvVar;
use crate::error::{PlmError, Result};
use crate::host::HostClient;
use crate::http;
use crate::repo::Repo;
use reqwest::Client;
use std::future::Future;
use std::pin::Pin;
use std::process::Command;

const API_BASE: &str = "https://api.github.com";

/// GitHub クライアント
pub struct GitHubClient {
    http: Client,
    auth: AuthProvider,
}

impl GitHubClient {
    /// 新しいGitHubClientを作成
    pub fn new(config: &HttpConfig, auth: &AuthProvider) -> Self {
        Self {
            http: config.build_client(),
            auth: auth.clone(),
        }
    }

    /// 認証トークンを取得
    ///
    /// 優先順位:
    /// 1. AuthProviderに設定されたトークン
    /// 2. 環境変数 GITHUB_TOKEN
    /// 3. gh CLI から取得
    fn get_token(&self) -> Option<String> {
        // 1. AuthProviderから取得
        if let Some(token) = self.auth.github_token() {
            return Some(token.to_string());
        }

        // 2. 環境変数から取得
        if let Some(token) = EnvVar::get("GITHUB_TOKEN") {
            return Some(token);
        }

        // 3. gh CLIから取得
        self.get_token_from_cli()
    }

    /// gh CLIからトークンを取得
    fn get_token_from_cli(&self) -> Option<String> {
        Command::new("gh")
            .args(["auth", "token"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .filter(|s| !s.is_empty())
    }

    /// 認証ヘッダーを生成
    fn auth_header(&self) -> Option<(&'static str, String)> {
        self.get_token()
            .map(|t| ("Authorization", format!("Bearer {}", t)))
    }

    /// リポジトリAPI URL
    fn repo_api_url(&self, repo: &Repo) -> String {
        format!("{}/repos/{}/{}", API_BASE, repo.owner(), repo.name())
    }

    /// Zipball URL
    fn zipball_url(&self, repo: &Repo, git_ref: &str) -> String {
        format!(
            "{}/repos/{}/{}/zipball/{}",
            API_BASE,
            repo.owner(),
            repo.name(),
            git_ref
        )
    }

    /// コミットURL
    fn commit_url(&self, repo: &Repo, git_ref: &str) -> String {
        format!(
            "{}/repos/{}/{}/commits/{}",
            API_BASE,
            repo.owner(),
            repo.name(),
            git_ref
        )
    }

    /// コンテンツURL
    fn contents_url(&self, repo: &Repo, path: &str, git_ref: &str) -> String {
        format!(
            "{}/repos/{}/{}/contents/{}?ref={}",
            API_BASE,
            repo.owner(),
            repo.name(),
            path,
            git_ref
        )
    }
}

impl HostClient for GitHubClient {
    fn get_default_branch<'a>(
        &'a self,
        repo: &'a Repo,
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
        Box::pin(async move {
            let url = self.repo_api_url(repo);

            let mut req = self.http.get(&url).header("User-Agent", "plm-cli");

            if let Some((name, value)) = self.auth_header() {
                req = req.header(name, value);
            }

            let response = req.send().await?;
            let status = response.status().as_u16();

            if !response.status().is_success() {
                let message = response.text().await.unwrap_or_default();
                return Err(PlmError::RepoApi {
                    host: "github".to_string(),
                    status,
                    message,
                });
            }

            let json: serde_json::Value = response.json().await?;
            let default_branch = json["default_branch"]
                .as_str()
                .unwrap_or("main")
                .to_string();

            Ok(default_branch)
        })
    }

    fn get_commit_sha<'a>(
        &'a self,
        repo: &'a Repo,
        git_ref: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
        Box::pin(async move {
            let url = self.commit_url(repo, git_ref);

            let mut req = self
                .http
                .get(&url)
                .header("User-Agent", "plm-cli")
                .header("Accept", "application/vnd.github.sha");

            if let Some((name, value)) = self.auth_header() {
                req = req.header(name, value);
            }

            let response = req.send().await?;
            let status = response.status().as_u16();

            if !response.status().is_success() {
                let message = response.text().await.unwrap_or_default();
                return Err(PlmError::RepoApi {
                    host: "github".to_string(),
                    status,
                    message,
                });
            }

            Ok(response.text().await?.trim().to_string())
        })
    }

    fn download_archive<'a>(
        &'a self,
        repo: &'a Repo,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'a>> {
        Box::pin(async move {
            let git_ref = if repo.git_ref().is_some() {
                repo.ref_or_default().to_string()
            } else {
                self.get_default_branch(repo).await?
            };

            let url = self.zipball_url(repo, &git_ref);
            http::download_with_progress(&self.http, &url, self.auth_header()).await
        })
    }

    fn download_archive_with_sha<'a>(
        &'a self,
        repo: &'a Repo,
    ) -> Pin<Box<dyn Future<Output = Result<(Vec<u8>, String, String)>> + Send + 'a>> {
        Box::pin(async move {
            let git_ref = if repo.git_ref().is_some() {
                repo.ref_or_default().to_string()
            } else {
                self.get_default_branch(repo).await?
            };

            let sha = self.get_commit_sha(repo, &git_ref).await?;
            let archive = self.download_archive(repo).await?;

            Ok((archive, git_ref, sha))
        })
    }

    fn fetch_file<'a>(
        &'a self,
        repo: &'a Repo,
        path: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
        Box::pin(async move {
            let git_ref = repo.ref_or_default();
            let url = self.contents_url(repo, path, git_ref);

            let mut req = self
                .http
                .get(&url)
                .header("User-Agent", "plm-cli")
                .header("Accept", "application/vnd.github.raw");

            if let Some((name, value)) = self.auth_header() {
                req = req.header(name, value);
            }

            let response = req.send().await?;
            let status = response.status().as_u16();

            if !response.status().is_success() {
                let message = response.text().await.unwrap_or_default();
                return Err(PlmError::RepoApi {
                    host: "github".to_string(),
                    status,
                    message,
                });
            }

            Ok(response.text().await?)
        })
    }
}

/// GitHub用のリポジトリパスパーサ
///
/// 対応フォーマット:
/// - `owner/repo`
/// - `https://github.com/owner/repo`
/// - `github.com/owner/repo`
pub fn parse_repo_path(input: &str) -> Result<(String, String)> {
    let input = input.trim();

    // GitHub URLのプレフィックスを削除
    let without_prefix = input
        .strip_prefix("https://github.com/")
        .or_else(|| input.strip_prefix("http://github.com/"))
        .or_else(|| input.strip_prefix("github.com/"))
        .unwrap_or(input);

    // .git サフィックスを削除
    let without_suffix = without_prefix
        .strip_suffix(".git")
        .unwrap_or(without_prefix);

    // /tree/branch や /blob/branch などのパスを削除
    let parts: Vec<&str> = without_suffix.split('/').collect();
    if parts.len() >= 2 {
        let owner = parts[0].trim();
        let name = parts[1].trim();

        if owner.is_empty() || name.is_empty() {
            return Err(PlmError::InvalidRepoFormat(input.to_string()));
        }

        Ok((owner.to_string(), name.to_string()))
    } else {
        Err(PlmError::InvalidRepoFormat(input.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_repo_path_simple() {
        let (owner, name) = parse_repo_path("owner/repo").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(name, "repo");
    }

    #[test]
    fn test_parse_repo_path_full_url() {
        let (owner, name) = parse_repo_path("https://github.com/owner/repo").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(name, "repo");
    }

    #[test]
    fn test_parse_repo_path_with_git_suffix() {
        let (owner, name) = parse_repo_path("https://github.com/owner/repo.git").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(name, "repo");
    }

    #[test]
    fn test_parse_repo_path_invalid() {
        assert!(parse_repo_path("invalid").is_err());
        assert!(parse_repo_path("").is_err());
    }

    // === 境界値テスト ===

    #[test]
    fn test_parse_repo_path_github_com_prefix() {
        // github.com/ プレフィックス（https://なし）
        let (owner, name) = parse_repo_path("github.com/owner/repo").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(name, "repo");
    }

    #[test]
    fn test_parse_repo_path_http_url() {
        // http:// (httpsではなく)
        let (owner, name) = parse_repo_path("http://github.com/owner/repo").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(name, "repo");
    }

    #[test]
    fn test_parse_repo_path_with_extra_path() {
        // 余剰パス（/tree/main など）
        let (owner, name) = parse_repo_path("https://github.com/owner/repo/tree/main").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(name, "repo");
    }

    #[test]
    fn test_parse_repo_path_with_blob_path() {
        // /blob/main/file.rs など
        let (owner, name) =
            parse_repo_path("https://github.com/owner/repo/blob/main/src/main.rs").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(name, "repo");
    }

    #[test]
    fn test_parse_repo_path_trailing_slash() {
        // 末尾スラッシュ
        let (owner, name) = parse_repo_path("owner/repo/").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(name, "repo");
    }

    #[test]
    fn test_parse_repo_path_empty_owner() {
        // 空オーナー
        let result = parse_repo_path("/repo");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_repo_path_empty_repo() {
        // 空リポジトリ
        let result = parse_repo_path("owner/");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_repo_path_both_empty() {
        // 両方空
        let result = parse_repo_path("/");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_repo_path_whitespace_owner() {
        // 空白のみのオーナー
        let result = parse_repo_path("  /repo");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_repo_path_whitespace_repo() {
        // 空白のみのリポジトリ
        let result = parse_repo_path("owner/  ");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_repo_path_with_leading_whitespace() {
        // 先頭の空白
        let (owner, name) = parse_repo_path("  owner/repo").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(name, "repo");
    }

    #[test]
    fn test_parse_repo_path_with_trailing_whitespace() {
        // 末尾の空白
        let (owner, name) = parse_repo_path("owner/repo  ").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(name, "repo");
    }

    #[test]
    fn test_parse_repo_path_only_owner() {
        // オーナーのみ（スラッシュなし）
        let result = parse_repo_path("owner");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_repo_path_git_suffix_simple() {
        // .git サフィックス（シンプルパス）
        let (owner, name) = parse_repo_path("owner/repo.git").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(name, "repo");
    }

    #[test]
    fn test_parse_repo_path_multiple_git_suffix() {
        // 複数の .git（エッジケース）
        let (owner, name) = parse_repo_path("owner/repo.git.git").unwrap();
        assert_eq!(owner, "owner");
        // 最初の .git のみ除去
        assert_eq!(name, "repo.git");
    }
}
