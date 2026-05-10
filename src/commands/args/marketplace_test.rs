use super::MarketplaceArgs;

#[test]
fn marketplace_or_default_returns_github_when_none() {
    let args = MarketplaceArgs { marketplace: None };
    assert_eq!(args.marketplace_or_default(), "github");
}

#[test]
fn marketplace_or_default_returns_value_when_some() {
    let args = MarketplaceArgs {
        marketplace: Some("custom".into()),
    };
    assert_eq!(args.marketplace_or_default(), "custom");
}
