//! scan::placement モジュールのテスト

use super::*;

#[test]
fn test_is_instruction_file_known() {
    assert!(is_instruction_file("AGENTS.md"));
    assert!(is_instruction_file("GEMINI.md"));
    assert!(is_instruction_file("copilot-instructions.md"));
}

#[test]
fn test_is_instruction_file_unknown() {
    assert!(!is_instruction_file("plugin_my-skill"));
    assert!(!is_instruction_file(""));
    assert!(!is_instruction_file("AGENTS.txt"));
}

#[test]
fn test_list_placed_components_empty() {
    let items: Vec<String> = vec![];
    let result = list_placed_components(&items);
    assert!(result.is_empty());
}

#[test]
fn test_list_placed_components_single_flat_name() {
    let items = vec!["plg_my-skill".to_string()];
    let result = list_placed_components(&items);
    assert_eq!(result.len(), 1);
    assert!(result.contains("plg_my-skill"));
}

#[test]
fn test_list_placed_components_dedup() {
    let items = vec![
        "plg_skill1".to_string(),
        "plg_skill1".to_string(),
        "plg_skill2".to_string(),
    ];
    let result = list_placed_components(&items);
    assert_eq!(result.len(), 2);
    assert!(result.contains("plg_skill1"));
    assert!(result.contains("plg_skill2"));
}

#[test]
fn test_list_placed_components_excludes_instruction_files() {
    let items = vec![
        "plg_skill".to_string(),
        "AGENTS.md".to_string(),
        "GEMINI.md".to_string(),
        "copilot-instructions.md".to_string(),
    ];
    let result = list_placed_components(&items);
    assert_eq!(result.len(), 1);
    assert!(result.contains("plg_skill"));
    assert!(!result.contains("AGENTS.md"));
    assert!(!result.contains("GEMINI.md"));
    assert!(!result.contains("copilot-instructions.md"));
}

#[test]
fn test_list_placed_components_excludes_empty_string() {
    let items = vec!["".to_string(), "valid_name".to_string()];
    let result = list_placed_components(&items);
    assert_eq!(result.len(), 1);
    assert!(result.contains("valid_name"));
}
