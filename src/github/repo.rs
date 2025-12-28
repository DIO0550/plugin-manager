use crate::error::{PlmError, Result};

/// Gitリポジトリ参照（GitHub/GitLab/Bitbucket等で共通利用可能）
#[derive(Debug, Clone)]
pub struct GitRepo {
    pub owner: String,
    pub repo: String,
    pub git_ref: Option<String>,
}

impl GitRepo {
    /// 新しいGitRepoを作成
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            repo: repo.into(),
            git_ref: None,
        }
    }

    /// refを指定してGitRepoを作成
    pub fn with_ref(owner: impl Into<String>, repo: impl Into<String>, git_ref: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            repo: repo.into(),
            git_ref: Some(git_ref.into()),
        }
    }

    /// "owner/repo" または "owner/repo@ref" 形式をパース
    pub fn parse(input: &str) -> Result<Self> {
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
    }

    #[test]
    fn test_parse_with_ref() {
        let repo = GitRepo::parse("owner/repo@v1.0.0").unwrap();
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.repo, "repo");
        assert_eq!(repo.git_ref, Some("v1.0.0".to_string()));
    }

    #[test]
    fn test_parse_with_branch() {
        let repo = GitRepo::parse("owner/repo@main").unwrap();
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.repo, "repo");
        assert_eq!(repo.git_ref, Some("main".to_string()));
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
    }

    #[test]
    fn test_ref_or_default() {
        let repo = GitRepo::new("owner", "repo");
        assert_eq!(repo.ref_or_default(), "HEAD");

        let repo_with_ref = GitRepo::with_ref("owner", "repo", "main");
        assert_eq!(repo_with_ref.ref_or_default(), "main");
    }
}
