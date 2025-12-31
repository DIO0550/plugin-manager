//! リポジトリ情報
//!
//! URLから適切なリポジトリ情報を生成する共通パイプライン。
//!
//! ## 使い方
//!
//! ```ignore
//! use plm::repo::{self, Repo};
//! use plm::host::HostClientFactory;
//!
//! let repo = repo::from_url("owner/repo")?;
//! let factory = HostClientFactory::with_defaults();
//! let client = factory.create(repo.host());
//! let archive = client.download_archive(&repo).await?;
//! ```
//!
//! ## 対応フォーマット
//!
//! - `owner/repo` - 短縮記法（GitHub デフォルト）
//! - `owner/repo@ref` - ref指定
//! - `https://github.com/owner/repo` - HTTP URL
//! - `ssh://git@github.com/owner/repo` - SSH URL
//! - `git@github.com:owner/repo` - SCP形式

use crate::error::{PlmError, Result};
use crate::host::{self, HostKind};

/// リポジトリ情報
///
/// ホスト種別とリポジトリの基本情報を保持する。
/// HTTP/API操作は `HostClient` を使用する。
#[derive(Debug, Clone)]
pub struct Repo {
    host: HostKind,
    owner: String,
    name: String,
    git_ref: Option<String>,
}

impl Repo {
    /// 新しいRepoを作成
    pub fn new(
        host: HostKind,
        owner: impl Into<String>,
        name: impl Into<String>,
        git_ref: Option<String>,
    ) -> Self {
        Self {
            host,
            owner: owner.into(),
            name: name.into(),
            git_ref,
        }
    }

    /// ホスト種別
    pub fn host(&self) -> HostKind {
        self.host
    }

    /// オーナー名
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// リポジトリ名
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Git ref（ブランチ、タグ、コミットSHA）
    pub fn git_ref(&self) -> Option<&str> {
        self.git_ref.as_deref()
    }

    /// デフォルトのgit ref（指定がなければ "HEAD"）
    pub fn ref_or_default(&self) -> &str {
        self.git_ref().unwrap_or("HEAD")
    }

    /// フルパス形式 (owner/repo)
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }
}

/// ソースロケータの種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceLocatorKind {
    /// HTTP/HTTPS URL (https://github.com/owner/repo)
    HttpUrl,
    /// SSH URL (ssh://git@github.com/owner/repo)
    SshUrl,
    /// SCP形式 (git@github.com:owner/repo)
    Scp,
    /// 短縮記法 (owner/repo)
    Shorthand,
}

/// URLからリポジトリを生成
///
/// 共通パイプライン:
/// 1. 入力形式を判定 (HTTP URL / SSH URL / SCP / 短縮記法)
/// 2. ホストとパスを抽出
/// 3. `@ref` を分離
/// 4. ホスト別パース: `host::<name>::parse_repo_path()` に委譲
/// 5. 正規化: `Repo { host, owner, name, git_ref }` を生成
pub fn from_url(input: &str) -> Result<Repo> {
    let (host, path, git_ref) = parse_input(input)?;

    // ホスト別パース
    let (owner, name) = match host {
        HostKind::GitHub => host::github::parse_repo_path(&path)?,
        HostKind::GitLab => {
            return Err(PlmError::Validation("GitLab is not yet supported".into()))
        }
        HostKind::Bitbucket => {
            return Err(PlmError::Validation("Bitbucket is not yet supported".into()))
        }
    };

    Ok(Repo::new(host, owner, name, git_ref))
}

/// 入力形式を判定
fn detect_source_locator_kind(input: &str) -> Result<SourceLocatorKind> {
    if let Some((scheme, _rest)) = input.split_once("://") {
        return match scheme {
            "http" | "https" => Ok(SourceLocatorKind::HttpUrl),
            "ssh" => Ok(SourceLocatorKind::SshUrl),
            _ => Err(PlmError::InvalidRepoFormat(format!(
                "Unsupported scheme: {}",
                scheme
            ))),
        };
    }

    if input.starts_with("git@") && input.contains(':') {
        return Ok(SourceLocatorKind::Scp);
    }

    Ok(SourceLocatorKind::Shorthand)
}

/// 入力をパースしてホスト、パス、git refを抽出
fn parse_input(input: &str) -> Result<(HostKind, String, Option<String>)> {
    let input = input.trim();
    if input.is_empty() {
        return Err(PlmError::InvalidRepoFormat(input.to_string()));
    }

    let (host_hint, raw_path) = match detect_source_locator_kind(input)? {
        SourceLocatorKind::HttpUrl => parse_http_url(input)?,
        SourceLocatorKind::SshUrl => parse_ssh_url(input)?,
        SourceLocatorKind::Scp => parse_scp_url(input)?,
        SourceLocatorKind::Shorthand => (None, input.to_string()),
    };

    let (path, git_ref) = split_ref(&raw_path)?;
    let host = host_hint.unwrap_or(HostKind::GitHub);

    Ok((host, path, git_ref))
}

/// パスから `@ref` を分離
fn split_ref(path: &str) -> Result<(String, Option<String>)> {
    // .git サフィックスを除去
    let path = path.strip_suffix(".git").unwrap_or(path);

    if let Some((left, right)) = path.split_once('@') {
        if right.is_empty() {
            return Err(PlmError::InvalidRepoFormat(format!(
                "Empty ref after @: {}",
                path
            )));
        }
        Ok((left.to_string(), Some(right.to_string())))
    } else {
        Ok((path.to_string(), None))
    }
}

/// HTTP/HTTPS URLをパース
fn parse_http_url(input: &str) -> Result<(Option<HostKind>, String)> {
    let rest = input
        .strip_prefix("https://")
        .or_else(|| input.strip_prefix("http://"))
        .unwrap_or(input);

    let (host, path) = rest
        .split_once('/')
        .ok_or_else(|| PlmError::InvalidRepoFormat(input.to_string()))?;

    let host_kind = host_kind_from_host(host)
        .ok_or_else(|| PlmError::InvalidRepoFormat(format!("Unknown host: {}", host)))?;

    Ok((Some(host_kind), path.trim_start_matches('/').to_string()))
}

/// SSH URLをパース (ssh://git@github.com/owner/repo)
fn parse_ssh_url(input: &str) -> Result<(Option<HostKind>, String)> {
    let rest = input.strip_prefix("ssh://").unwrap_or(input);

    let (host_part, path) = rest
        .split_once('/')
        .ok_or_else(|| PlmError::InvalidRepoFormat(input.to_string()))?;

    // user@host から host を抽出
    let host = host_part
        .rsplit_once('@')
        .map(|(_, h)| h)
        .unwrap_or(host_part);

    let host_kind = host_kind_from_host(host)
        .ok_or_else(|| PlmError::InvalidRepoFormat(format!("Unknown host: {}", host)))?;

    Ok((Some(host_kind), path.trim_start_matches('/').to_string()))
}

/// SCP形式URLをパース (git@github.com:owner/repo)
fn parse_scp_url(input: &str) -> Result<(Option<HostKind>, String)> {
    let rest = input.strip_prefix("git@").unwrap_or(input);

    let (host, path) = rest
        .split_once(':')
        .ok_or_else(|| PlmError::InvalidRepoFormat(input.to_string()))?;

    let host_kind = host_kind_from_host(host)
        .ok_or_else(|| PlmError::InvalidRepoFormat(format!("Unknown host: {}", host)))?;

    Ok((Some(host_kind), path.to_string()))
}

/// ホスト名からHostKindを取得
fn host_kind_from_host(host: &str) -> Option<HostKind> {
    // ポート番号を除去 (host:port -> host)
    let host = host.split(':').next().unwrap_or(host);

    match host.to_ascii_lowercase().as_str() {
        "github.com" | "www.github.com" => Some(HostKind::GitHub),
        "gitlab.com" | "www.gitlab.com" => Some(HostKind::GitLab),
        "bitbucket.org" | "www.bitbucket.org" => Some(HostKind::Bitbucket),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === 短縮記法 ===

    #[test]
    fn test_from_url_simple() {
        let repo = from_url("owner/repo").unwrap();
        assert_eq!(repo.owner(), "owner");
        assert_eq!(repo.name(), "repo");
        assert_eq!(repo.host(), HostKind::GitHub);
        assert!(repo.git_ref().is_none());
    }

    #[test]
    fn test_from_url_with_ref() {
        let repo = from_url("owner/repo@v1.0.0").unwrap();
        assert_eq!(repo.owner(), "owner");
        assert_eq!(repo.name(), "repo");
        assert_eq!(repo.git_ref(), Some("v1.0.0"));
    }

    #[test]
    fn test_from_url_full_name() {
        let repo = from_url("owner/repo").unwrap();
        assert_eq!(repo.full_name(), "owner/repo");
    }

    #[test]
    fn test_from_url_ref_or_default() {
        let repo = from_url("owner/repo").unwrap();
        assert_eq!(repo.ref_or_default(), "HEAD");

        let repo_with_ref = from_url("owner/repo@main").unwrap();
        assert_eq!(repo_with_ref.ref_or_default(), "main");
    }

    // === HTTP URL ===

    #[test]
    fn test_from_url_https_github() {
        let repo = from_url("https://github.com/DIO0550/cc-plugin").unwrap();
        assert_eq!(repo.owner(), "DIO0550");
        assert_eq!(repo.name(), "cc-plugin");
        assert_eq!(repo.host(), HostKind::GitHub);
    }

    #[test]
    fn test_from_url_https_with_git_suffix() {
        let repo = from_url("https://github.com/owner/repo.git").unwrap();
        assert_eq!(repo.owner(), "owner");
        assert_eq!(repo.name(), "repo");
    }

    #[test]
    fn test_from_url_https_with_ref() {
        let repo = from_url("https://github.com/owner/repo@v1.0.0").unwrap();
        assert_eq!(repo.owner(), "owner");
        assert_eq!(repo.name(), "repo");
        assert_eq!(repo.git_ref(), Some("v1.0.0"));
    }

    // === SCP形式 ===

    #[test]
    fn test_from_url_scp() {
        let repo = from_url("git@github.com:owner/repo").unwrap();
        assert_eq!(repo.owner(), "owner");
        assert_eq!(repo.name(), "repo");
        assert_eq!(repo.host(), HostKind::GitHub);
    }

    #[test]
    fn test_from_url_scp_with_ref() {
        let repo = from_url("git@github.com:owner/repo@v1.0.0").unwrap();
        assert_eq!(repo.owner(), "owner");
        assert_eq!(repo.name(), "repo");
        assert_eq!(repo.git_ref(), Some("v1.0.0"));
    }

    #[test]
    fn test_from_url_scp_with_git_suffix() {
        let repo = from_url("git@github.com:owner/repo.git").unwrap();
        assert_eq!(repo.owner(), "owner");
        assert_eq!(repo.name(), "repo");
    }

    // === SSH URL ===

    #[test]
    fn test_from_url_ssh() {
        let repo = from_url("ssh://git@github.com/owner/repo").unwrap();
        assert_eq!(repo.owner(), "owner");
        assert_eq!(repo.name(), "repo");
        assert_eq!(repo.host(), HostKind::GitHub);
    }

    #[test]
    fn test_from_url_ssh_with_ref() {
        let repo = from_url("ssh://git@github.com/owner/repo@v1.0.0").unwrap();
        assert_eq!(repo.owner(), "owner");
        assert_eq!(repo.name(), "repo");
        assert_eq!(repo.git_ref(), Some("v1.0.0"));
    }

    // === エラーケース ===

    #[test]
    fn test_from_url_invalid_empty() {
        assert!(from_url("").is_err());
        assert!(from_url("   ").is_err());
    }

    #[test]
    fn test_from_url_unknown_host() {
        let result = from_url("https://unknown.example.com/owner/repo");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Unknown host"));
    }

    #[test]
    fn test_from_url_unsupported_scheme() {
        let result = from_url("ftp://github.com/owner/repo");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Unsupported scheme"));
    }

    #[test]
    fn test_from_url_empty_ref() {
        let result = from_url("owner/repo@");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Empty ref"));
    }

    // === SourceLocatorKind判定 ===

    #[test]
    fn test_detect_source_locator_kind() {
        assert_eq!(
            detect_source_locator_kind("https://github.com/owner/repo").unwrap(),
            SourceLocatorKind::HttpUrl
        );
        assert_eq!(
            detect_source_locator_kind("http://github.com/owner/repo").unwrap(),
            SourceLocatorKind::HttpUrl
        );
        assert_eq!(
            detect_source_locator_kind("ssh://git@github.com/owner/repo").unwrap(),
            SourceLocatorKind::SshUrl
        );
        assert_eq!(
            detect_source_locator_kind("git@github.com:owner/repo").unwrap(),
            SourceLocatorKind::Scp
        );
        assert_eq!(
            detect_source_locator_kind("owner/repo").unwrap(),
            SourceLocatorKind::Shorthand
        );
    }

    // === host_kind_from_host ===

    #[test]
    fn test_host_kind_from_host() {
        assert_eq!(host_kind_from_host("github.com"), Some(HostKind::GitHub));
        assert_eq!(
            host_kind_from_host("www.github.com"),
            Some(HostKind::GitHub)
        );
        assert_eq!(host_kind_from_host("gitlab.com"), Some(HostKind::GitLab));
        assert_eq!(
            host_kind_from_host("bitbucket.org"),
            Some(HostKind::Bitbucket)
        );
        assert_eq!(host_kind_from_host("unknown.com"), None);
    }
}
