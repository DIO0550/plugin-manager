use super::*;

#[test]
fn parse_github_returns_github_variant() {
    assert_eq!(MarketplaceRef::parse("github"), MarketplaceRef::Github);
}

#[test]
fn parse_named_returns_named_variant() {
    assert_eq!(
        MarketplaceRef::parse("my-market"),
        MarketplaceRef::Named("my-market".to_string())
    );
}

#[test]
fn from_option_none_is_github() {
    assert_eq!(MarketplaceRef::from_option(None), MarketplaceRef::Github);
}

#[test]
fn from_option_some_github_is_normalized_to_github() {
    // None と Some("github") の意味揺れを 1 つの正規形に畳み込む
    assert_eq!(
        MarketplaceRef::from_option(Some("github")),
        MarketplaceRef::Github
    );
}

#[test]
fn from_option_named_is_named() {
    assert_eq!(
        MarketplaceRef::from_option(Some("official")),
        MarketplaceRef::Named("official".to_string())
    );
}

#[test]
fn dir_name_github_is_default_marketplace() {
    assert_eq!(MarketplaceRef::Github.dir_name(), DEFAULT_MARKETPLACE);
    assert_eq!(MarketplaceRef::Github.dir_name(), "github");
}

#[test]
fn dir_name_named_is_marketplace_name() {
    assert_eq!(
        MarketplaceRef::Named("official".to_string()).dir_name(),
        "official"
    );
}

#[test]
fn is_github() {
    assert!(MarketplaceRef::Github.is_github());
    assert!(!MarketplaceRef::Named("official".to_string()).is_github());
}

#[test]
fn into_named() {
    assert_eq!(MarketplaceRef::Github.into_named(), None);
    assert_eq!(
        MarketplaceRef::Named("official".to_string()).into_named(),
        Some("official".to_string())
    );
}

#[test]
fn display_shows_dir_name() {
    assert_eq!(format!("{}", MarketplaceRef::Github), "github");
    assert_eq!(
        format!("{}", MarketplaceRef::Named("official".to_string())),
        "official"
    );
}
