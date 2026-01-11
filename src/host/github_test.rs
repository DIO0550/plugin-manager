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
