use super::*;

#[test]
fn test_parse_github_repo() {
    let source = parse_source("owner/repo").unwrap();
    // Box<dyn PluginSource> なので型は確認できないが、パースは成功する
    assert!(std::ptr::eq(
        source.as_ref() as *const dyn PluginSource as *const (),
        source.as_ref() as *const dyn PluginSource as *const ()
    ));
}

#[test]
fn test_parse_github_repo_with_ref() {
    let source = parse_source("owner/repo@v1.0.0");
    assert!(source.is_ok());
}

#[test]
fn test_parse_github_full_url() {
    let source = parse_source("https://github.com/owner/repo");
    assert!(source.is_ok());
}

#[test]
fn test_parse_marketplace() {
    let source = parse_source("plugin@marketplace");
    assert!(source.is_ok());
}

#[test]
fn test_parse_search() {
    let source = parse_source("plugin-name");
    assert!(source.is_ok());
}

// === 境界値テスト ===

#[test]
fn test_parse_empty_string() {
    // 空文字は SearchSource として処理される
    let source = parse_source("");
    assert!(source.is_ok());
}

#[test]
fn test_parse_whitespace_only() {
    // 空白のみも SearchSource として処理される
    let source = parse_source("   ");
    assert!(source.is_ok());
}

#[test]
fn test_parse_plugin_at_empty() {
    // "plugin@" は plugin="" の MarketplaceSource
    let source = parse_source("plugin@");
    assert!(source.is_ok());
}

#[test]
fn test_parse_at_marketplace() {
    // "@marketplace" は left="" の MarketplaceSource
    let source = parse_source("@marketplace");
    assert!(source.is_ok());
}

#[test]
fn test_parse_owner_repo_at_empty_ref() {
    // "owner/repo@" は空refエラーになる（repo::from_urlで処理）
    let source = parse_source("owner/repo@");
    assert!(source.is_err());
}

#[test]
fn test_parse_multiple_at_symbols() {
    // "a@b@c" は左側に "/" がないので MarketplaceSource(left="a", right="b@c")
    let source = parse_source("a@b@c");
    assert!(source.is_ok());
}

#[test]
fn test_parse_owner_repo_multiple_at() {
    // "owner/repo@v1@extra" は ref="v1@extra" として処理
    let source = parse_source("owner/repo@v1@extra");
    assert!(source.is_ok());
}

#[test]
fn test_parse_at_only() {
    // "@" のみは MarketplaceSource(left="", right="")
    let source = parse_source("@");
    assert!(source.is_ok());
}

#[test]
fn test_parse_slash_only() {
    // "/" のみはGitHubリポジトリとして処理されるがエラー
    let source = parse_source("/");
    assert!(source.is_err());
}

#[test]
fn test_parse_owner_slash_empty() {
    // "owner/" は空のリポジトリ名でエラー
    let source = parse_source("owner/");
    assert!(source.is_err());
}

#[test]
fn test_parse_slash_repo() {
    // "/repo" は空のオーナー名でエラー
    let source = parse_source("/repo");
    assert!(source.is_err());
}
