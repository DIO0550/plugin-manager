//! `build_plugin_row_spans` の狭幅フォールバックテスト

use super::*;
use crate::application::InstalledPlugin;
use crate::tui::manager::core::LIST_DECORATION_WIDTH;

fn make_test_plugin(name: &str) -> InstalledPlugin {
    InstalledPlugin::new_for_test(name, "1.0.0", Vec::new(), None, None, true)
}

fn span_texts(spans: &[Span<'_>]) -> Vec<String> {
    spans.iter().map(|s| s.content.to_string()).collect()
}

#[test]
fn build_plugin_row_uses_multi_span_when_wide() {
    let plugin = make_test_plugin("plugin-name");
    let status = UpdateStatusDisplay::AlreadyUpToDate;
    let spans = build_plugin_row_spans(&plugin, false, Some(&status), 80);
    assert!(
        spans.len() >= 3,
        "wide path should keep multiple spans, got {}",
        spans.len()
    );
    let texts = span_texts(&spans);
    assert!(
        texts.iter().any(|s| s.contains("Up to date")),
        "update status span should be present in wide path: {:?}",
        texts
    );
}

#[test]
fn build_plugin_row_falls_back_to_single_span_when_narrow() {
    let plugin = make_test_plugin("very-long-plugin-name-that-exceeds-width");
    let status = UpdateStatusDisplay::AlreadyUpToDate;
    let content_width = 30u16;
    let spans = build_plugin_row_spans(&plugin, false, Some(&status), content_width);
    assert!(
        spans.len() <= 2,
        "narrow path should collapse to <=2 spans, got {}",
        spans.len()
    );
    let texts = span_texts(&spans);
    assert!(
        !texts.iter().any(|s| s.contains("Up to date")),
        "update status span should be dropped in narrow path: {:?}",
        texts
    );
    let list_inner_width = content_width.saturating_sub(LIST_DECORATION_WIDTH) as usize;
    let total_chars: usize = texts.iter().map(|s| s.chars().count()).sum();
    assert!(
        total_chars <= list_inner_width,
        "total {} > list_inner_width {}",
        total_chars,
        list_inner_width
    );
}

#[test]
fn build_plugin_row_does_not_panic_at_zero_width() {
    let plugin = make_test_plugin("p");
    let _ = build_plugin_row_spans(&plugin, false, None, 0);
}

#[test]
fn build_plugin_row_returns_2_line_list_item() {
    let plugin = make_test_plugin("p");
    let item = build_plugin_row(&plugin, false, None, 80, false);
    assert_eq!(item.height(), 2);
}

#[test]
fn build_plugin_row_returns_2_line_list_item_when_marked() {
    let plugin = make_test_plugin("p");
    let item = build_plugin_row(&plugin, true, None, 80, false);
    assert_eq!(item.height(), 2);
}

#[test]
fn build_plugin_row_returns_2_line_list_item_when_narrow() {
    let plugin = make_test_plugin("very-long-plugin-name");
    let item = build_plugin_row(&plugin, false, None, 30, false);
    assert_eq!(item.height(), 2);
}

#[test]
fn build_detail_action_item_returns_height_2() {
    use crate::tui::manager::screens::installed::model::DetailAction;
    for action in DetailAction::for_enabled()
        .iter()
        .chain(DetailAction::for_disabled().iter())
    {
        let item = build_detail_action_item(action, false);
        assert_eq!(item.height(), 2);
    }
}

#[test]
fn build_detail_action_menu_reserves_full_height_for_enabled() {
    use crate::tui::manager::screens::installed::model::DetailAction;
    let actions = DetailAction::for_enabled();
    let (items, rows) = build_detail_action_menu(&actions, None);
    assert_eq!(items.len(), actions.len());
    assert_eq!(rows, actions.len() as u16 * 2);
    for item in &items {
        assert_eq!(item.height(), 2);
    }
}

#[test]
fn build_detail_action_menu_reserves_full_height_for_disabled() {
    use crate::tui::manager::screens::installed::model::DetailAction;
    let actions = DetailAction::for_disabled();
    let (items, rows) = build_detail_action_menu(&actions, None);
    assert_eq!(items.len(), actions.len());
    assert_eq!(rows, actions.len() as u16 * 2);
    for item in &items {
        assert_eq!(item.height(), 2);
    }
}

#[test]
fn build_component_types_item_returns_height_2() {
    use crate::component::ComponentKind;
    let item = build_component_types_item(ComponentKind::Skill, 3, false);
    assert_eq!(item.height(), 2);
}

#[test]
fn build_component_list_item_returns_height_2() {
    let item = build_component_list_item("my-component", false);
    assert_eq!(item.height(), 2);
}
