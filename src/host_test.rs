use super::*;

#[test]
fn test_host_kind_as_str() {
    assert_eq!(HostKind::GitHub.as_str(), "github");
    assert_eq!(HostKind::GitLab.as_str(), "gitlab");
    assert_eq!(HostKind::Bitbucket.as_str(), "bitbucket");
}

#[test]
fn test_host_kind_display() {
    assert_eq!(format!("{}", HostKind::GitHub), "github");
}

#[test]
fn test_factory_creation() {
    let factory = HostClientFactory::with_defaults();
    // GitHubクライアントは生成できる
    let _client = factory.create(HostKind::GitHub);
}

// === 境界値テスト ===

#[test]
#[should_panic(expected = "GitLab is not yet supported")]
fn test_factory_gitlab_panics() {
    let factory = HostClientFactory::with_defaults();
    let _client = factory.create(HostKind::GitLab);
}

#[test]
#[should_panic(expected = "Bitbucket is not yet supported")]
fn test_factory_bitbucket_panics() {
    let factory = HostClientFactory::with_defaults();
    let _client = factory.create(HostKind::Bitbucket);
}
