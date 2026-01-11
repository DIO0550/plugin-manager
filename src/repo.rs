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
#[path = "repo_test.rs"]
mod tests;

#[cfg(test)]
#[path = "repo_proptests.rs"]
mod proptests;
