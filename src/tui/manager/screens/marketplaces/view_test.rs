use super::*;
use crate::marketplace::PluginSource;
use ratatui::prelude::{Color, Style};
use std::collections::HashSet;

// =============================================================================
// should_split_layout
// =============================================================================

#[test]
fn should_split_layout_returns_false_for_zero() {
    assert!(!should_split_layout(0));
}

#[test]
fn should_split_layout_returns_false_below_threshold() {
    assert!(!should_split_layout(59));
}

#[test]
fn should_split_layout_returns_true_at_threshold() {
    assert!(should_split_layout(60));
}

#[test]
fn should_split_layout_returns_true_above_threshold() {
    assert!(should_split_layout(200));
}

// =============================================================================
// browse_checkbox
// =============================================================================

#[test]
fn browse_checkbox_installed() {
    let (mark, style) = browse_checkbox(true, false);
    assert_eq!(mark, "[x] ");
    assert_eq!(style, Style::default().fg(Color::DarkGray));
}

#[test]
fn browse_checkbox_installed_and_selected() {
    // installed が優先される
    let (mark, style) = browse_checkbox(true, true);
    assert_eq!(mark, "[x] ");
    assert_eq!(style, Style::default().fg(Color::DarkGray));
}

#[test]
fn browse_checkbox_selected() {
    let (mark, style) = browse_checkbox(false, true);
    assert_eq!(mark, "[x] ");
    assert_eq!(style, Style::default().fg(Color::Yellow));
}

#[test]
fn browse_checkbox_unselected() {
    let (mark, style) = browse_checkbox(false, false);
    assert_eq!(mark, "[ ] ");
    assert_eq!(style, Style::default());
}

// =============================================================================
// build_browse_list_items
// =============================================================================

fn make_plugin(name: &str, installed: bool) -> BrowsePlugin {
    BrowsePlugin {
        name: name.to_string(),
        description: None,
        version: None,
        source: PluginSource::Local("./test".to_string()),
        installed,
    }
}

#[test]
fn build_browse_list_items_empty() {
    let plugins: Vec<BrowsePlugin> = vec![];
    let selected = HashSet::new();
    let items = build_browse_list_items(&plugins, &selected);
    assert!(items.is_empty());
}

#[test]
fn build_browse_list_items_returns_correct_count() {
    let plugins = vec![
        make_plugin("a", true),
        make_plugin("b", false),
        make_plugin("c", false),
    ];
    let mut selected = HashSet::new();
    selected.insert("b".to_string());
    let items = build_browse_list_items(&plugins, &selected);
    assert_eq!(items.len(), 3);
}

#[test]
fn build_browse_list_items_respects_selected_plugins() {
    let plugins = vec![make_plugin("alpha", false), make_plugin("beta", false)];
    let mut selected = HashSet::new();
    selected.insert("alpha".to_string());

    // selected に含まれるプラグインと含まれないプラグインで異なるアイテムが生成される
    let items_with_selection = build_browse_list_items(&plugins, &selected);
    let items_without_selection = build_browse_list_items(&plugins, &HashSet::new());

    // 選択状態が異なるため、生成されるアイテムも異なるはず
    assert_eq!(items_with_selection.len(), 2);
    assert_eq!(items_without_selection.len(), 2);

    // "alpha" は selected に含まれるため、選択あり/なしでアイテムが異なることを確認する
    assert_ne!(items_with_selection[0], items_without_selection[0]);
}

// =============================================================================
// target_checkbox
// =============================================================================

#[test]
fn target_checkbox_selected() {
    let (mark, style) = target_checkbox(true);
    assert_eq!(mark, "[x] ");
    assert_eq!(style, Style::default().fg(Color::Yellow));
}

#[test]
fn target_checkbox_unselected() {
    let (mark, style) = target_checkbox(false);
    assert_eq!(mark, "[ ] ");
    assert_eq!(style, Style::default());
}

// =============================================================================
// build_target_list_items
// =============================================================================

#[test]
fn build_target_list_items_empty() {
    let targets: Vec<(String, String, bool)> = vec![];
    let items = build_target_list_items(&targets);
    assert!(items.is_empty());
}

#[test]
fn build_target_list_items_all_selected() {
    let targets = vec![
        ("codex".to_string(), "Codex".to_string(), true),
        ("copilot".to_string(), "Copilot".to_string(), true),
    ];
    let items = build_target_list_items(&targets);
    assert_eq!(items.len(), 2);
    // All items should match Yellow-styled selected items
    let expected_0 =
        ListItem::new("  [x] Codex".to_string()).style(Style::default().fg(Color::Yellow));
    let expected_1 =
        ListItem::new("  [x] Copilot".to_string()).style(Style::default().fg(Color::Yellow));
    assert_eq!(items[0], expected_0);
    assert_eq!(items[1], expected_1);
}

#[test]
fn build_target_list_items_none_selected() {
    let targets = vec![
        ("codex".to_string(), "Codex".to_string(), false),
        ("copilot".to_string(), "Copilot".to_string(), false),
    ];
    let items = build_target_list_items(&targets);
    assert_eq!(items.len(), 2);
    // All items should match default-styled unselected items
    let expected_0 = ListItem::new("  [ ] Codex".to_string()).style(Style::default());
    let expected_1 = ListItem::new("  [ ] Copilot".to_string()).style(Style::default());
    assert_eq!(items[0], expected_0);
    assert_eq!(items[1], expected_1);
}

#[test]
fn build_target_list_items_mixed() {
    let targets = vec![
        ("codex".to_string(), "Codex".to_string(), true),
        ("copilot".to_string(), "Copilot".to_string(), false),
    ];
    let items = build_target_list_items(&targets);
    assert_eq!(items.len(), 2);
    // Selected and unselected items should differ
    assert_ne!(items[0], items[1]);
    let expected_selected =
        ListItem::new("  [x] Codex".to_string()).style(Style::default().fg(Color::Yellow));
    let expected_unselected = ListItem::new("  [ ] Copilot".to_string()).style(Style::default());
    assert_eq!(items[0], expected_selected);
    assert_eq!(items[1], expected_unselected);
}

