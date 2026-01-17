use super::*;

#[test]
fn test_parse_component_name_valid() {
    let (origin, name) = parse_component_name("github/my-plugin/my-skill").unwrap();
    assert_eq!(origin.marketplace, "github");
    assert_eq!(origin.plugin, "my-plugin");
    assert_eq!(name, "my-skill");
}

#[test]
fn test_parse_component_name_instruction() {
    let (origin, name) = parse_component_name("AGENTS.md").unwrap();
    assert_eq!(origin.marketplace, "");
    assert_eq!(origin.plugin, "");
    assert_eq!(name, "AGENTS.md");
}

#[test]
fn test_parse_component_name_error() {
    let result = parse_component_name("invalid");
    assert!(result.is_err());

    let result = parse_component_name("only/two");
    assert!(result.is_err());
}
