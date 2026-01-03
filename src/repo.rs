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

    // === 境界値テスト ===

    #[test]
    fn test_from_url_trailing_slash() {
        // 末尾スラッシュは余剰パスとして処理される
        let repo = from_url("owner/repo/").unwrap();
        assert_eq!(repo.owner(), "owner");
        assert_eq!(repo.name(), "repo");
    }

    #[test]
    fn test_from_url_git_suffix_with_ref() {
        // .git と @ref の組み合わせ
        let repo = from_url("owner/repo.git@v1.0.0").unwrap();
        assert_eq!(repo.owner(), "owner");
        assert_eq!(repo.name(), "repo");
        assert_eq!(repo.git_ref(), Some("v1.0.0"));
    }

    #[test]
    fn test_from_url_https_git_suffix_with_ref() {
        let repo = from_url("https://github.com/owner/repo.git@main").unwrap();
        assert_eq!(repo.owner(), "owner");
        assert_eq!(repo.name(), "repo");
        assert_eq!(repo.git_ref(), Some("main"));
    }

    #[test]
    fn test_from_url_with_tree_path() {
        // /tree/branch などの余剰パスは無視される
        let repo = from_url("https://github.com/owner/repo/tree/main").unwrap();
        assert_eq!(repo.owner(), "owner");
        assert_eq!(repo.name(), "repo");
    }

    #[test]
    fn test_from_url_with_blob_path() {
        let repo = from_url("https://github.com/owner/repo/blob/main/README.md").unwrap();
        assert_eq!(repo.owner(), "owner");
        assert_eq!(repo.name(), "repo");
    }

    #[test]
    fn test_from_url_uppercase_host() {
        // ホスト名の大文字は正規化される
        let repo = from_url("https://GITHUB.COM/owner/repo").unwrap();
        assert_eq!(repo.host(), HostKind::GitHub);
        assert_eq!(repo.owner(), "owner");
        assert_eq!(repo.name(), "repo");
    }

    #[test]
    fn test_from_url_mixed_case_host() {
        let repo = from_url("https://GitHub.Com/owner/repo").unwrap();
        assert_eq!(repo.host(), HostKind::GitHub);
    }

    #[test]
    fn test_from_url_host_with_port() {
        // ポート付きホストはポート除去後に判定
        let repo = from_url("https://github.com:443/owner/repo").unwrap();
        assert_eq!(repo.host(), HostKind::GitHub);
        assert_eq!(repo.owner(), "owner");
        assert_eq!(repo.name(), "repo");
    }

    #[test]
    fn test_from_url_scp_missing_path() {
        // SCP形式でパスがない
        let result = from_url("git@github.com:");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_url_ssh_missing_path() {
        // SSH形式でパスがない
        let result = from_url("ssh://git@github.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_url_http_missing_path() {
        // HTTP形式でパスがない
        let result = from_url("https://github.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_url_ssh_with_port() {
        // SSH形式でポート指定
        let repo = from_url("ssh://git@github.com:22/owner/repo").unwrap();
        assert_eq!(repo.host(), HostKind::GitHub);
        assert_eq!(repo.owner(), "owner");
        assert_eq!(repo.name(), "repo");
    }

    #[test]
    fn test_from_url_multiple_slashes() {
        // 複数スラッシュは先頭2セグメントのみ使用
        let repo = from_url("owner/repo/extra/path").unwrap();
        assert_eq!(repo.owner(), "owner");
        assert_eq!(repo.name(), "repo");
    }

    #[test]
    fn test_from_url_ref_with_slash() {
        // refにスラッシュを含む場合（feature/branchなど）
        // split_once('@') は最初の @ で分割するので "feature/branch" がrefになる
        let repo = from_url("owner/repo@feature/branch").unwrap();
        assert_eq!(repo.owner(), "owner");
        assert_eq!(repo.name(), "repo");
        assert_eq!(repo.git_ref(), Some("feature/branch"));
    }

    #[test]
    fn test_split_ref_with_multiple_dots() {
        // refに複数ドットを含む場合
        let repo = from_url("owner/repo@v1.2.3").unwrap();
        assert_eq!(repo.git_ref(), Some("v1.2.3"));
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    /// owner/repo に使える文字列（英数字、ハイフン、アンダースコア）
    fn valid_name_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z][a-zA-Z0-9_-]{0,19}".prop_map(|s| s)
    }

    /// git ref に使える文字列
    fn valid_ref_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9][a-zA-Z0-9._/-]{0,19}".prop_map(|s| s)
    }

    proptest! {
        /// 異なる形式で同じ owner/repo を指定した場合、同じ結果が得られる
        #[test]
        fn prop_all_formats_produce_same_owner_name(
            owner in valid_name_strategy(),
            repo in valid_name_strategy()
        ) {
            // 短縮形式
            let shorthand = format!("{}/{}", owner, repo);
            let result_short = from_url(&shorthand).unwrap();

            // HTTPS形式
            let https = format!("https://github.com/{}/{}", owner, repo);
            let result_https = from_url(&https).unwrap();

            // SCP形式
            let scp = format!("git@github.com:{}/{}", owner, repo);
            let result_scp = from_url(&scp).unwrap();

            // SSH形式
            let ssh = format!("ssh://git@github.com/{}/{}", owner, repo);
            let result_ssh = from_url(&ssh).unwrap();

            // すべて同じ owner/name になる
            prop_assert_eq!(result_short.owner(), result_https.owner());
            prop_assert_eq!(result_short.owner(), result_scp.owner());
            prop_assert_eq!(result_short.owner(), result_ssh.owner());

            prop_assert_eq!(result_short.name(), result_https.name());
            prop_assert_eq!(result_short.name(), result_scp.name());
            prop_assert_eq!(result_short.name(), result_ssh.name());

            // ホストはすべて GitHub
            prop_assert_eq!(result_short.host(), HostKind::GitHub);
            prop_assert_eq!(result_https.host(), HostKind::GitHub);
            prop_assert_eq!(result_scp.host(), HostKind::GitHub);
            prop_assert_eq!(result_ssh.host(), HostKind::GitHub);
        }

        /// ref 指定時に ref_or_default が正しい値を返す
        #[test]
        fn prop_ref_or_default_returns_ref_when_specified(
            owner in valid_name_strategy(),
            repo in valid_name_strategy(),
            git_ref in valid_ref_strategy()
        ) {
            let input = format!("{}/{}@{}", owner, repo, git_ref);
            let result = from_url(&input).unwrap();

            prop_assert_eq!(result.git_ref(), Some(git_ref.as_str()));
            prop_assert_eq!(result.ref_or_default(), &git_ref);
        }

        /// ref 未指定時に ref_or_default が "HEAD" を返す
        #[test]
        fn prop_ref_or_default_returns_head_when_unspecified(
            owner in valid_name_strategy(),
            repo in valid_name_strategy()
        ) {
            let input = format!("{}/{}", owner, repo);
            let result = from_url(&input).unwrap();

            prop_assert!(result.git_ref().is_none());
            prop_assert_eq!(result.ref_or_default(), "HEAD");
        }

        /// .git サフィックスは除去される
        #[test]
        fn prop_git_suffix_is_removed(
            owner in valid_name_strategy(),
            repo in valid_name_strategy()
        ) {
            let with_git = format!("{}/{}.git", owner, repo);
            let without_git = format!("{}/{}", owner, repo);

            let result_with = from_url(&with_git).unwrap();
            let result_without = from_url(&without_git).unwrap();

            prop_assert_eq!(result_with.name(), result_without.name());
        }

        /// full_name は owner/repo 形式を返す
        #[test]
        fn prop_full_name_format(
            owner in valid_name_strategy(),
            repo in valid_name_strategy()
        ) {
            let input = format!("{}/{}", owner, repo);
            let result = from_url(&input).unwrap();

            let expected_full_name = format!("{}/{}", owner, repo);
            prop_assert_eq!(result.full_name(), expected_full_name);
        }
    }
}
