use super::repo::GitRepo;
use super::token::Token;
use crate::error::{PlmError, Result};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;

/// GitHub APIクライアント
pub struct GitHubClient {
    client: Client,
}

impl GitHubClient {
    /// 新しいGitHubクライアントを作成
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// 認証トークンを取得
    fn auth_token(&self) -> Option<Token> {
        Token::from_env()
    }

    /// リポジトリのデフォルトブランチを取得
    pub async fn get_default_branch(&self, repo: &GitRepo) -> Result<String> {
        let url = repo.github_repo_url();

        let mut req = self.client.get(&url).header("User-Agent", "plm-cli");

        if let Some(token) = self.auth_token() {
            req = req.header("Authorization", token.to_bearer());
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
    pub async fn download_archive(&self, repo: &GitRepo) -> Result<Vec<u8>> {
        let git_ref = if repo.git_ref.is_some() {
            repo.ref_or_default().to_string()
        } else {
            self.get_default_branch(repo).await?
        };

        // GitHub API経由でzipballを取得（プライベートリポジトリ対応）
        let url = repo.github_zipball_url(&git_ref);

        self.download_with_progress(&url).await
    }

    /// タグからzipアーカイブをダウンロード
    /// GitHub API経由でダウンロードするため、プライベートリポジトリにも対応
    pub async fn download_archive_by_tag(&self, repo: &GitRepo, tag: &str) -> Result<Vec<u8>> {
        // GitHub API経由でzipballを取得（プライベートリポジトリ対応）
        let url = repo.github_zipball_url(tag);

        self.download_with_progress(&url).await
    }

    /// ブランチまたはrefの最新コミットSHAを取得
    pub async fn get_commit_sha(&self, repo: &GitRepo, git_ref: &str) -> Result<String> {
        let url = repo.github_commit_url(git_ref);

        let mut req = self
            .client
            .get(&url)
            .header("User-Agent", "plm-cli")
            .header("Accept", "application/vnd.github.sha");

        if let Some(token) = self.auth_token() {
            req = req.header("Authorization", token.to_bearer());
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
    pub async fn download_archive_with_sha(&self, repo: &GitRepo) -> Result<(Vec<u8>, String, String)> {
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
            req = req.header("Authorization", token.to_bearer());
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
    pub async fn fetch_file(&self, repo: &GitRepo, path: &str) -> Result<String> {
        let git_ref = if repo.git_ref.is_some() {
            repo.ref_or_default().to_string()
        } else {
            self.get_default_branch(repo).await?
        };

        // GitHub API経由でファイルコンテンツを取得（プライベートリポジトリ対応）
        let url = repo.github_contents_url(path, &git_ref);

        let mut req = self.client
            .get(&url)
            .header("User-Agent", "plm-cli")
            .header("Accept", "application/vnd.github.raw+json");

        if let Some(token) = self.auth_token() {
            req = req.header("Authorization", token.to_bearer());
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
