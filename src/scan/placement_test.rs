//! scan::placement モジュールのテスト

use super::*;

#[test]
fn test_parse_placement_empty() {
    assert_eq!(parse_placement(""), None);
}

#[test]
fn test_parse_placement_no_slash() {
    assert_eq!(parse_placement("plugin"), None);
}

#[test]
fn test_parse_placement_two_segments() {
    assert_eq!(
        parse_placement("marketplace/plugin"),
        Some(("marketplace".to_string(), "plugin".to_string()))
    );
}

#[test]
fn test_parse_placement_three_segments() {
    assert_eq!(
        parse_placement("marketplace/plugin/skill"),
        Some(("marketplace".to_string(), "plugin".to_string()))
    );
}

#[test]
fn test_parse_placement_leading_slash() {
    assert_eq!(parse_placement("/plugin"), None);
}

#[test]
fn test_parse_placement_trailing_slash() {
    assert_eq!(parse_placement("marketplace/"), None);
}

#[test]
fn test_parse_placement_only_slash() {
    assert_eq!(parse_placement("/"), None);
}

#[test]
fn test_list_placed_plugins_empty() {
    let items: Vec<String> = vec![];
    let result = list_placed_plugins(&items);
    assert!(result.is_empty());
}

#[test]
fn test_list_placed_plugins_single() {
    let items = vec!["github/my-plugin/my-skill".to_string()];
    let result = list_placed_plugins(&items);
    assert_eq!(result.len(), 1);
    assert!(result.contains(&("github".to_string(), "my-plugin".to_string())));
}

#[test]
fn test_list_placed_plugins_dedup() {
    let items = vec![
        "github/my-plugin/skill1".to_string(),
        "github/my-plugin/skill2".to_string(),
        "github/other-plugin/agent".to_string(),
    ];
    let result = list_placed_plugins(&items);
    assert_eq!(result.len(), 2);
    assert!(result.contains(&("github".to_string(), "my-plugin".to_string())));
    assert!(result.contains(&("github".to_string(), "other-plugin".to_string())));
}

#[test]
fn test_list_placed_plugins_ignores_invalid() {
    let items = vec![
        "github/my-plugin/skill".to_string(),
        "AGENTS.md".to_string(), // No slash, should be ignored
        "/invalid".to_string(),  // Leading slash, should be ignored
    ];
    let result = list_placed_plugins(&items);
    assert_eq!(result.len(), 1);
    assert!(result.contains(&("github".to_string(), "my-plugin".to_string())));
}
