use super::*;
use crate::marketplace::PluginSource;

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

    // "alpha" は selected に含まれるため、選択あり/なしで描画内容が異なることを確認する
    let alpha_with_selection = format!("{:?}", &items_with_selection[0]);
    let alpha_without_selection = format!("{:?}", &items_without_selection[0]);
    assert_ne!(alpha_with_selection, alpha_without_selection);
}
