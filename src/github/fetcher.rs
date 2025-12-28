use crate::error::{PlmError, Result};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::process::Command;

/// GitHubトークンを取得
/// 優先順位: 1. GITHUB_TOKEN環境変数, 2. gh CLI認証
fn get_github_token() -> Option<String> {
    // 1. 環境変数を優先（CI/CD用）
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        if !token.is_empty() {
            return Some(token);
        }
    }

    // 2. gh CLI から取得（ローカル開発用）
    if let Ok(output) = Command::new("gh").args(["auth", "token"]).output() {
        if output.status.success() {
            let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !token.is_empty() {
                return Some(token);
            }
        }
    }

    None
}

/// GitHubリポジトリ参照
#[derive(Debug, Clone)]
pub struct RepoRef {
    pub owner: String,
    pub repo: String,
    pub git_ref: Option<String>,
}

impl RepoRef {
    /// "owner/repo" または "owner/repo@ref" 形式をパース
    pub fn parse(input: &str) -> Result<Self> {
        let (repo_part, git_ref) = if let Some((repo, ref_part)) = input.split_once('@') {
            (repo, Some(ref_part.to_string()))
        } else {
            (input, None)
        };

        let parts: Vec<&str> = repo_part.split('/').collect();
        if parts.len() != 2 {
            return Err(PlmError::InvalidRepoFormat(input.to_string()));
        }

        let owner = parts[0].trim();
        let repo = parts[1].trim();

        if owner.is_empty() || repo.is_empty() {
            return Err(PlmError::InvalidRepoFormat(input.to_string()));
        }

        Ok(Self {
            owner: owner.to_string(),
            repo: repo.to_string(),
            git_ref,
        })
    }

    /// デフォルトブランチまたは指定されたrefを返す
    pub fn ref_or_default(&self) -> &str {
        self.git_ref.as_deref().unwrap_or("HEAD")
    }
}

/// GitHub APIクライアント
pub struct GitHubClient {
    client: Client,
    base_url: String,
}

impl GitHubClient {
    /// 新しいGitHubクライアントを作成
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://api.github.com".to_string(),
        }
    }

    /// 認証トークンを取得
    fn auth_token(&self) -> Option<String> {
        get_github_token()
    }

    /// リポジトリのデフォルトブランチを取得
    pub async fn get_default_branch(&self, repo: &RepoRef) -> Result<String> {
        let url = format!(
            "{}/repos/{}/{}",
            self.base_url, repo.owner, repo.repo
        );

        let mut req = self.client.get(&url).header("User-Agent", "plm-cli");

        if let Some(token) = self.auth_token() {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        let response = req.send().await?;
        let status = response.status().as_u16();

        if !response.status().is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(PlmError::GitHubApi { status, message });
        }

        let json: serde_json::Value = response.json().await?;
        let default_branch = json["default_branch"]
            .as_str()
            .unwrap_or("main")
            .to_string();

        Ok(default_branch)
    }

    /// リポジトリをzipアーカイブとしてダウンロード
    /// GitHub API経由でダウンロードするため、プライベートリポジトリにも対応
    pub async fn download_archive(&self, repo: &RepoRef) -> Result<Vec<u8>> {
        let git_ref = if repo.git_ref.is_some() {
            repo.ref_or_default().to_string()
        } else {
            self.get_default_branch(repo).await?
        };

        // GitHub API経由でzipballを取得（プライベートリポジトリ対応）
        let url = format!(
            "{}/repos/{}/{}/zipball/{}",
            self.base_url, repo.owner, repo.repo, git_ref
        );

        self.download_with_progress(&url).await
    }

    /// タグからzipアーカイブをダウンロード
    /// GitHub API経由でダウンロードするため、プライベートリポジトリにも対応
    pub async fn download_archive_by_tag(&self, repo: &RepoRef, tag: &str) -> Result<Vec<u8>> {
        // GitHub API経由でzipballを取得（プライベートリポジトリ対応）
        let url = format!(
            "{}/repos/{}/{}/zipball/{}",
            self.base_url, repo.owner, repo.repo, tag
        );

        self.download_with_progress(&url).await
    }

    /// ブランチまたはrefの最新コミットSHAを取得
    pub async fn get_commit_sha(&self, repo: &RepoRef, git_ref: &str) -> Result<String> {
        let url = format!(
            "{}/repos/{}/{}/commits/{}",
            self.base_url, repo.owner, repo.repo, git_ref
        );

        let mut req = self
            .client
            .get(&url)
            .header("User-Agent", "plm-cli")
            .header("Accept", "application/vnd.github.sha");

        if let Some(token) = self.auth_token() {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        let response = req.send().await?;
        let status = response.status().as_u16();

        if !response.status().is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(PlmError::GitHubApi { status, message });
        }

        Ok(response.text().await?.trim().to_string())
    }

    /// リポジトリをダウンロードし、コミットSHAも一緒に返す
    pub async fn download_archive_with_sha(&self, repo: &RepoRef) -> Result<(Vec<u8>, String, String)> {
        let git_ref = if repo.git_ref.is_some() {
            repo.ref_or_default().to_string()
        } else {
            self.get_default_branch(repo).await?
        };

        let sha = self.get_commit_sha(repo, &git_ref).await?;
        let archive = self.download_archive(repo).await?;

        Ok((archive, git_ref, sha))
    }

    /// プログレスバー付きでダウンロード
    async fn download_with_progress(&self, url: &str) -> Result<Vec<u8>> {
        let mut req = self.client.get(url).header("User-Agent", "plm-cli");

        if let Some(token) = self.auth_token() {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        let response = req.send().await?;
        let status = response.status().as_u16();

        if !response.status().is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(PlmError::GitHubApi { status, message });
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

    /// 単一ファイルを取得
    /// GitHub API経由で取得するため、プライベートリポジトリにも対応
    pub async fn fetch_file(&self, repo: &RepoRef, path: &str) -> Result<String> {
        let git_ref = if repo.git_ref.is_some() {
            repo.ref_or_default().to_string()
        } else {
            self.get_default_branch(repo).await?
        };

        // GitHub API経由でファイルコンテンツを取得（プライベートリポジトリ対応）
        let url = format!(
            "{}/repos/{}/{}/contents/{}?ref={}",
            self.base_url, repo.owner, repo.repo, path, git_ref
        );

        let mut req = self.client
            .get(&url)
            .header("User-Agent", "plm-cli")
            .header("Accept", "application/vnd.github.raw+json");

        if let Some(token) = self.auth_token() {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        let response = req.send().await?;
        let status = response.status().as_u16();

        if !response.status().is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(PlmError::GitHubApi { status, message });
        }

        Ok(response.text().await?)
    }
}

impl Default for GitHubClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_repo_ref_simple() {
        let repo = RepoRef::parse("owner/repo").unwrap();
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.repo, "repo");
        assert!(repo.git_ref.is_none());
    }

    #[test]
    fn test_parse_repo_ref_with_ref() {
        let repo = RepoRef::parse("owner/repo@v1.0.0").unwrap();
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.repo, "repo");
        assert_eq!(repo.git_ref, Some("v1.0.0".to_string()));
    }

    #[test]
    fn test_parse_repo_ref_with_branch() {
        let repo = RepoRef::parse("owner/repo@main").unwrap();
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.repo, "repo");
        assert_eq!(repo.git_ref, Some("main".to_string()));
    }

    #[test]
    fn test_parse_repo_ref_invalid() {
        assert!(RepoRef::parse("invalid").is_err());
        assert!(RepoRef::parse("").is_err());
        assert!(RepoRef::parse("/repo").is_err());
        assert!(RepoRef::parse("owner/").is_err());
    }
}
