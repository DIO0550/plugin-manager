use super::GithubCacheId;
use crate::host::HostKind;
use crate::repo::Repo;

#[test]
fn from_repo_encodes_owner_and_name() {
    let repo = Repo::new(HostKind::GitHub, "owner", "repo", None);
    let id = GithubCacheId::from_repo(&repo);
    assert_eq!(id.as_str(), "owner--repo");
}

#[test]
fn from_parts_encodes_owner_and_name() {
    let id = GithubCacheId::from_parts("owner", "repo");
    assert_eq!(id.to_string(), "owner--repo");
}

#[test]
fn parts_decodes_encoded_id() {
    let id = GithubCacheId::from_parts("owner", "repo");
    assert_eq!(id.parts(), Some(("owner", "repo")));
}

#[test]
fn parts_roundtrips_repo_name_containing_separator() {
    // GitHub の owner 名は連続ハイフンを含められないため、
    // repo 名に `--` が含まれても最初の区切りで正しく復元できる
    let id = GithubCacheId::from_parts("owner", "my--repo");
    assert_eq!(id.parts(), Some(("owner", "my--repo")));
}

#[test]
fn parts_returns_none_without_separator() {
    let id = GithubCacheId::from_cache_name("plain-name");
    assert_eq!(id.parts(), None);
}

#[test]
fn parts_returns_none_for_empty_owner_or_name() {
    assert_eq!(GithubCacheId::from_cache_name("--repo").parts(), None);
    assert_eq!(GithubCacheId::from_cache_name("owner--").parts(), None);
}
