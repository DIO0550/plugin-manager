use super::MarketplaceName;

#[test]
fn parse_normalizes_uppercase_name() {
    let name = MarketplaceName::parse("My-MP").unwrap();

    assert_eq!(name.as_str(), "my-mp");
}

#[test]
fn parse_rejects_invalid_name() {
    assert!(MarketplaceName::parse("my marketplace").is_err());
}

#[test]
fn display_uses_normalized_name() {
    let name = MarketplaceName::parse("My-MP").unwrap();

    assert_eq!(name.to_string(), "my-mp");
}
