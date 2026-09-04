use super::MarketplaceArgs;

#[test]
fn marketplace_or_default_returns_github_when_none() {
    let args = MarketplaceArgs { marketplace: None };
    assert_eq!(args.marketplace_or_default().unwrap().as_str(), "github");
}

#[test]
fn marketplace_or_default_returns_value_when_some() {
    let args = MarketplaceArgs {
        marketplace: Some("custom".into()),
    };
    assert_eq!(args.marketplace_or_default().unwrap().as_str(), "custom");
}

#[test]
fn marketplace_or_default_normalizes_uppercase_value() {
    let args = MarketplaceArgs {
        marketplace: Some("My-MP".into()),
    };

    assert_eq!(args.marketplace_or_default().unwrap().as_str(), "my-mp");
}

#[test]
fn marketplace_or_default_rejects_invalid_value() {
    let args = MarketplaceArgs {
        marketplace: Some("my marketplace".into()),
    };

    assert!(args.marketplace_or_default().is_err());
}
