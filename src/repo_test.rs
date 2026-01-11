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
