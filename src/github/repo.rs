use crate::error::{PlmError, Result};

/// Gitリポジトリ参照（GitHub/GitLab/Bitbucket等で共通利用可能）
#[derive(Debug, Clone)]
pub struct GitRepo {
    pub owner: String,
    pub repo: String,
    pub git_ref: Option<String>,
    /// パース前の生の入力文字列
    pub raw: String,
}

impl GitRepo {
    /// 新しいGitRepoを作成
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        let owner = owner.into();
        let repo = repo.into();
        let raw = format!("{}/{}", owner, repo);
        Self {
            owner,
            repo,
            git_ref: None,
            raw,
        }
    }

    /// refを指定してGitRepoを作成
    pub fn with_ref(
        owner: impl Into<String>,
        repo: impl Into<String>,
        git_ref: impl Into<String>,
    ) -> Self {
        let owner = owner.into();
        let repo = repo.into();
        let git_ref = git_ref.into();
        let raw = format!("{}/{}@{}", owner, repo, git_ref);
        Self {
            owner,
            repo,
            git_ref: Some(git_ref),
            raw,
        }
    }

    /// "owner/repo" または "owner/repo@ref" 形式をパース
    pub fn parse(input: &str) -> Result<Self> {
        let raw = input.to_string();

        let (repo_part, git_ref) = match input.split_once('@') {
            Some((repo, ref_part)) => (repo, Some(ref_part.to_string())),
            None => (input, None),
        };

        let (owner, repo) = repo_part
            .split_once('/')
            .ok_or_else(|| PlmError::InvalidRepoFormat(input.to_string()))?;

        let owner = owner.trim();
        let repo = repo.trim();

        if owner.is_empty() || repo.is_empty() {
            return Err(PlmError::InvalidRepoFormat(input.to_string()));
        }

        Ok(Self {
            owner: owner.to_string(),
            repo: repo.to_string(),
            git_ref,
            raw,
        })
    }

    /// デフォルトブランチまたは指定されたrefを返す
    pub fn ref_or_default(&self) -> &str {
        self.git_ref.as_deref().unwrap_or("HEAD")
    }

    /// リポジトリ名をフルパス形式で返す (owner/repo)
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.repo)
    }

    // ===== GitHub URL生成メソッド =====

    /// GitHub API base URL
    const GITHUB_API: &'static str = "https://api.github.com";

    /// GitHub リポジトリ情報API URL
    pub fn github_repo_url(&self) -> String {
        format!("{}/repos/{}/{}", Self::GITHUB_API, self.owner, self.repo)
    }

    /// GitHub zipball ダウンロードURL
    pub fn github_zipball_url(&self, git_ref: &str) -> String {
        format!(
            "{}/repos/{}/{}/zipball/{}",
            Self::GITHUB_API,
            self.owner,
            self.repo,
            git_ref
        )
    }

    /// GitHub コミットSHA取得URL
    pub fn github_commit_url(&self, git_ref: &str) -> String {
        format!(
            "{}/repos/{}/{}/commits/{}",
            Self::GITHUB_API,
            self.owner,
            self.repo,
            git_ref
        )
    }

    /// GitHub ファイルコンテンツ取得URL
    pub fn github_contents_url(&self, path: &str, git_ref: &str) -> String {
        format!(
            "{}/repos/{}/{}/contents/{}?ref={}",
            Self::GITHUB_API,
            self.owner,
            self.repo,
            path,
            git_ref
        )
    }

    /// GitHub リポジトリのWeb URL (ブラウザ用)
    pub fn github_web_url(&self) -> String {
        format!("https://github.com/{}/{}", self.owner, self.repo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let repo = GitRepo::parse("owner/repo").unwrap();
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.repo, "repo");
        assert!(repo.git_ref.is_none());
        assert_eq!(repo.raw, "owner/repo");
    }

    #[test]
    fn test_parse_with_ref() {
        let repo = GitRepo::parse("owner/repo@v1.0.0").unwrap();
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.repo, "repo");
        assert_eq!(repo.git_ref, Some("v1.0.0".to_string()));
        assert_eq!(repo.raw, "owner/repo@v1.0.0");
    }

    #[test]
    fn test_parse_with_branch() {
        let repo = GitRepo::parse("owner/repo@main").unwrap();
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.repo, "repo");
        assert_eq!(repo.git_ref, Some("main".to_string()));
        assert_eq!(repo.raw, "owner/repo@main");
    }

    #[test]
    fn test_parse_invalid() {
        assert!(GitRepo::parse("invalid").is_err());
        assert!(GitRepo::parse("").is_err());
        assert!(GitRepo::parse("/repo").is_err());
        assert!(GitRepo::parse("owner/").is_err());
    }

    #[test]
    fn test_full_name() {
        let repo = GitRepo::new("owner", "repo");
        assert_eq!(repo.full_name(), "owner/repo");
        assert_eq!(repo.raw, "owner/repo");
    }

    #[test]
    fn test_ref_or_default() {
        let repo = GitRepo::new("owner", "repo");
        assert_eq!(repo.ref_or_default(), "HEAD");

        let repo_with_ref = GitRepo::with_ref("owner", "repo", "main");
        assert_eq!(repo_with_ref.ref_or_default(), "main");
        assert_eq!(repo_with_ref.raw, "owner/repo@main");
    }

    #[test]
    fn test_github_urls() {
        let repo = GitRepo::new("anthropics", "claude-code");

        assert_eq!(
            repo.github_repo_url(),
            "https://api.github.com/repos/anthropics/claude-code"
        );
        assert_eq!(
            repo.github_zipball_url("main"),
            "https://api.github.com/repos/anthropics/claude-code/zipball/main"
        );
        assert_eq!(
            repo.github_commit_url("abc123"),
            "https://api.github.com/repos/anthropics/claude-code/commits/abc123"
        );
        assert_eq!(
            repo.github_contents_url("README.md", "main"),
            "https://api.github.com/repos/anthropics/claude-code/contents/README.md?ref=main"
        );
        assert_eq!(
            repo.github_web_url(),
            "https://github.com/anthropics/claude-code"
        );
    }
}
