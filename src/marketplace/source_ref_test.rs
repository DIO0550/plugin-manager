use super::MarketplaceSourceRef;

// ==================== parse ====================

#[test]
fn parse_accepts_internal_form_with_github_prefix() {
    let source: MarketplaceSourceRef = "github:owner/repo".parse().unwrap();
    assert_eq!(source.owner(), "owner");
    assert_eq!(source.name(), "repo");
}

#[test]
fn parse_accepts_display_form_without_prefix() {
    let source: MarketplaceSourceRef = "owner/repo".parse().unwrap();
    assert_eq!(source.owner(), "owner");
    assert_eq!(source.name(), "repo");
}

#[test]
fn parse_accepts_https_url() {
    let source: MarketplaceSourceRef = "https://github.com/owner/repo".parse().unwrap();
    assert_eq!(source.owner(), "owner");
    assert_eq!(source.name(), "repo");
}

#[test]
fn parse_rejects_missing_repo_name() {
    assert!("owner".parse::<MarketplaceSourceRef>().is_err());
    assert!("github:owner".parse::<MarketplaceSourceRef>().is_err());
}

#[test]
fn parse_rejects_empty() {
    assert!("".parse::<MarketplaceSourceRef>().is_err());
    assert!("github:".parse::<MarketplaceSourceRef>().is_err());
}

#[test]
fn parse_rejects_path_unsafe_names() {
    assert!("github:owner/..".parse::<MarketplaceSourceRef>().is_err());
    assert!("github:owner/.".parse::<MarketplaceSourceRef>().is_err());
    assert!("github:../repo".parse::<MarketplaceSourceRef>().is_err());
    assert!(r"github:owner/re\po"
        .parse::<MarketplaceSourceRef>()
        .is_err());
}

// ==================== display / full_name ====================

#[test]
fn display_uses_canonical_internal_form() {
    let source: MarketplaceSourceRef = "owner/repo".parse().unwrap();
    assert_eq!(source.to_string(), "github:owner/repo");
}

#[test]
fn full_name_returns_display_form() {
    let source: MarketplaceSourceRef = "github:owner/repo".parse().unwrap();
    assert_eq!(source.full_name(), "owner/repo");
}

// ==================== to_repo / from_repo ====================

#[test]
fn to_repo_builds_github_repo_without_ref() {
    let source: MarketplaceSourceRef = "github:owner/repo".parse().unwrap();
    let repo = source.to_repo();
    assert_eq!(repo.owner(), "owner");
    assert_eq!(repo.name(), "repo");
    assert!(repo.git_ref().is_none());
}

#[test]
fn to_repo_with_ref_embeds_git_ref() {
    let source: MarketplaceSourceRef = "github:owner/repo".parse().unwrap();
    let repo = source.to_repo_with_ref(Some("main".to_string()));
    assert_eq!(repo.git_ref(), Some("main"));
}

#[test]
fn from_repo_roundtrips_through_to_repo() {
    let source: MarketplaceSourceRef = "owner/repo".parse().unwrap();
    let roundtripped = MarketplaceSourceRef::from_repo(&source.to_repo());
    assert_eq!(roundtripped, source);
}

// ==================== serde ====================

#[test]
fn serializes_as_canonical_internal_form() {
    let source: MarketplaceSourceRef = "owner/repo".parse().unwrap();
    let json = serde_json::to_string(&source).unwrap();
    assert_eq!(json, r#""github:owner/repo""#);
}

#[test]
fn deserializes_internal_form() {
    let source: MarketplaceSourceRef = serde_json::from_str(r#""github:owner/repo""#).unwrap();
    assert_eq!(source.full_name(), "owner/repo");
}

#[test]
fn deserializes_legacy_display_form() {
    // 旧 marketplaces.json はプレフィックスなしで保存されていた
    let source: MarketplaceSourceRef = serde_json::from_str(r#""owner/repo""#).unwrap();
    assert_eq!(source.to_string(), "github:owner/repo");
}

#[test]
fn deserialize_rejects_invalid_source() {
    assert!(serde_json::from_str::<MarketplaceSourceRef>(r#""not-a-repo""#).is_err());
}
