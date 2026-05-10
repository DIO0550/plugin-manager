use super::*;
use crate::component::Scope;
use crate::marketplace::PluginSource;
use crate::tui::manager::core::style::{
    CHECKBOX_SELECTED, CHECKBOX_UNSELECTED, LIST_ITEM_INDENT, MARK_MARKED, RADIO_SELECTED,
    RADIO_UNSELECTED,
};
use crate::tui::manager::core::MarketplaceItem;
use ratatui::prelude::{Color, Line, Style};
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
// browse_state_block
// =============================================================================

#[test]
fn browse_state_block_installed() {
    let (mark, style) = browse_state_block(true, false);
    assert_eq!(mark, MARK_MARKED);
    assert_eq!(style, Style::default().fg(Color::DarkGray));
}

#[test]
fn browse_state_block_installed_takes_precedence_over_selected() {
    let (mark, style) = browse_state_block(true, true);
    assert_eq!(mark, MARK_MARKED);
    assert_eq!(style, Style::default().fg(Color::DarkGray));
}

#[test]
fn browse_state_block_selected() {
    let (mark, style) = browse_state_block(false, true);
    assert_eq!(mark, CHECKBOX_SELECTED);
    assert_eq!(style, Style::default().fg(Color::Yellow));
}

#[test]
fn browse_state_block_idle() {
    let (mark, style) = browse_state_block(false, false);
    assert_eq!(mark, CHECKBOX_UNSELECTED);
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
    let items = build_browse_list_items(&plugins, &selected, 80, None);
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
    let items = build_browse_list_items(&plugins, &selected, 80, None);
    assert_eq!(items.len(), 3);
}

#[test]
fn build_browse_list_items_respects_selected_plugins() {
    let plugins = vec![make_plugin("alpha", false), make_plugin("beta", false)];
    let mut selected = HashSet::new();
    selected.insert("alpha".to_string());

    let items_with_selection = build_browse_list_items(&plugins, &selected, 80, None);
    let items_without_selection = build_browse_list_items(&plugins, &HashSet::new(), 80, None);

    assert_eq!(items_with_selection.len(), 2);
    assert_eq!(items_without_selection.len(), 2);

    assert_ne!(items_with_selection[0], items_without_selection[0]);
}

#[test]
fn build_browse_list_items_have_height_2() {
    let plugins = vec![make_plugin("a", false), make_plugin("b", true)];
    let items = build_browse_list_items(&plugins, &HashSet::new(), 80, None);
    for item in &items {
        assert_eq!(item.height(), 2);
    }
}

// =============================================================================
// target_checkbox
// =============================================================================

#[test]
fn target_checkbox_selected() {
    let (mark, style) = target_checkbox(true);
    assert_eq!(mark, CHECKBOX_SELECTED);
    assert_eq!(style, Style::default().fg(Color::Yellow));
}

#[test]
fn target_checkbox_unselected() {
    let (mark, style) = target_checkbox(false);
    assert_eq!(mark, CHECKBOX_UNSELECTED);
    assert_eq!(style, Style::default());
}

// =============================================================================
// scope_radio
// =============================================================================

#[test]
fn scope_radio_current() {
    let (mark, style) = scope_radio(true);
    assert_eq!(mark, RADIO_SELECTED);
    assert_eq!(style, Style::default().fg(Color::Yellow));
}

#[test]
fn scope_radio_not_current() {
    let (mark, style) = scope_radio(false);
    assert_eq!(mark, RADIO_UNSELECTED);
    assert_eq!(style, Style::default());
}

// =============================================================================
// build_target_list_items
// =============================================================================

fn target_item(mark: &str, label: &str, style: Style) -> ListItem<'static> {
    use ratatui::text::Span;
    let line_text = format!("{}{} {}", LIST_ITEM_INDENT, mark, label);
    ListItem::new(vec![Line::from(Span::styled(line_text, style))])
}

#[test]
fn build_target_list_items_empty() {
    let targets: Vec<(String, String, bool)> = vec![];
    let items = build_target_list_items(&targets, None);
    assert!(items.is_empty());
}

#[test]
fn build_target_list_items_all_selected() {
    let targets = vec![
        ("codex".to_string(), "Codex".to_string(), true),
        ("copilot".to_string(), "Copilot".to_string(), true),
    ];
    let items = build_target_list_items(&targets, None);
    assert_eq!(items.len(), 2);
    let yellow = Style::default().fg(Color::Yellow);
    assert_eq!(items[0], target_item(CHECKBOX_SELECTED, "Codex", yellow));
    assert_eq!(items[1], target_item(CHECKBOX_SELECTED, "Copilot", yellow));
}

#[test]
fn build_target_list_items_none_selected() {
    let targets = vec![
        ("codex".to_string(), "Codex".to_string(), false),
        ("copilot".to_string(), "Copilot".to_string(), false),
    ];
    let items = build_target_list_items(&targets, None);
    assert_eq!(items.len(), 2);
    let default = Style::default();
    assert_eq!(items[0], target_item(CHECKBOX_UNSELECTED, "Codex", default));
    assert_eq!(
        items[1],
        target_item(CHECKBOX_UNSELECTED, "Copilot", default)
    );
}

#[test]
fn build_target_list_items_mixed() {
    let targets = vec![
        ("codex".to_string(), "Codex".to_string(), true),
        ("copilot".to_string(), "Copilot".to_string(), false),
    ];
    let items = build_target_list_items(&targets, None);
    assert_eq!(items.len(), 2);
    assert_ne!(items[0], items[1]);
    assert_eq!(
        items[0],
        target_item(
            CHECKBOX_SELECTED,
            "Codex",
            Style::default().fg(Color::Yellow)
        )
    );
    assert_eq!(
        items[1],
        target_item(CHECKBOX_UNSELECTED, "Copilot", Style::default())
    );
}

#[test]
fn build_target_list_items_have_height_1() {
    let targets = vec![("codex".to_string(), "Codex".to_string(), true)];
    let items = build_target_list_items(&targets, None);
    for item in &items {
        assert_eq!(item.height(), 1);
    }
}

// =============================================================================
// build_scope_list_items
// =============================================================================

fn scope_item(mark: &str, scope: Scope, path: &str, style: Style) -> ListItem<'static> {
    use ratatui::text::Span;
    let line_text = format!(
        "{}{} {} {}",
        LIST_ITEM_INDENT,
        mark,
        scope.display_name(),
        path
    );
    ListItem::new(vec![Line::from(Span::styled(line_text, style))])
}

#[test]
fn build_scope_list_items_personal_selected() {
    let items = build_scope_list_items(0, None);
    assert_eq!(items.len(), 2);
    let yellow = Style::default().fg(Color::Yellow);
    assert_eq!(
        items[0],
        scope_item(RADIO_SELECTED, Scope::Personal, "(~/.plm/)", yellow)
    );
    assert_eq!(
        items[1],
        scope_item(RADIO_UNSELECTED, Scope::Project, "(./)", Style::default())
    );
}

#[test]
fn build_scope_list_items_project_selected() {
    let items = build_scope_list_items(1, None);
    assert_eq!(items.len(), 2);
    let yellow = Style::default().fg(Color::Yellow);
    assert_eq!(
        items[0],
        scope_item(
            RADIO_UNSELECTED,
            Scope::Personal,
            "(~/.plm/)",
            Style::default()
        )
    );
    assert_eq!(
        items[1],
        scope_item(RADIO_SELECTED, Scope::Project, "(./)", yellow)
    );
}

#[test]
fn build_scope_list_items_out_of_range_clamps_to_last() {
    let items = build_scope_list_items(99, None);
    assert_eq!(items.len(), 2);
    let yellow = Style::default().fg(Color::Yellow);
    assert_eq!(
        items[0],
        scope_item(
            RADIO_UNSELECTED,
            Scope::Personal,
            "(~/.plm/)",
            Style::default()
        )
    );
    assert_eq!(
        items[1],
        scope_item(RADIO_SELECTED, Scope::Project, "(./)", yellow)
    );
}

#[test]
fn build_scope_list_items_have_height_1() {
    let items = build_scope_list_items(0, None);
    for item in &items {
        assert_eq!(item.height(), 1);
    }
}

// =============================================================================
// build_market_action_item
// =============================================================================

#[test]
fn build_market_action_item_returns_height_1() {
    for action in super::DetailAction::all().iter() {
        let item = build_market_action_item(action, false);
        assert_eq!(item.height(), 1);
    }
}

#[test]
fn build_market_action_menu_reserves_full_height() {
    let actions = super::DetailAction::all();
    let (items, rows) = build_market_action_menu(actions, None);
    assert_eq!(items.len(), actions.len());
    assert_eq!(rows, actions.len() as u16);
    for item in &items {
        assert_eq!(item.height(), 1);
    }
}

// =============================================================================
// build_marketplaces_list_item / build_add_marketplace_item
// =============================================================================

fn make_marketplace(name: &str) -> MarketplaceItem {
    MarketplaceItem {
        name: name.to_string(),
        source: "github".to_string(),
        source_path: None,
        plugin_count: Some(3),
        last_updated: Some("2026-04-29".to_string()),
    }
}

#[test]
fn build_marketplaces_list_item_returns_height_2() {
    let m = make_marketplace("alpha");
    let item = build_marketplaces_list_item(&m, 80, false);
    assert_eq!(item.height(), 2);
}

#[test]
fn build_marketplaces_list_item_returns_height_2_when_selected() {
    let m = make_marketplace("alpha");
    let item = build_marketplaces_list_item(&m, 80, true);
    assert_eq!(item.height(), 2);
}

#[test]
fn build_add_marketplace_item_returns_height_2() {
    let item = build_add_marketplace_item(false);
    assert_eq!(item.height(), 2);
}

#[test]
fn build_add_marketplace_item_returns_height_2_when_selected() {
    let item = build_add_marketplace_item(true);
    assert_eq!(item.height(), 2);
}

// =============================================================================
// build_plugin_list_item
// =============================================================================

#[test]
fn build_plugin_list_item_with_description_returns_height_2() {
    let item = build_plugin_list_item("plugin-a", Some("description"), 80, false);
    assert_eq!(item.height(), 2);
}

#[test]
fn build_plugin_list_item_without_description_returns_height_2() {
    let item = build_plugin_list_item("plugin-a", None, 80, false);
    assert_eq!(item.height(), 2);
}

#[test]
fn build_plugin_list_item_returns_height_2_when_selected() {
    let item = build_plugin_list_item("plugin-a", Some("description"), 80, true);
    assert_eq!(item.height(), 2);
}

// =============================================================================
// installing_inner_layout
// =============================================================================

#[test]
fn installing_inner_layout_returns_4_3_min() {
    use ratatui::layout::Rect;
    let rects = installing_inner_layout(Rect::new(0, 0, 80, 22));
    assert_eq!(rects[0].height, 4);
    assert_eq!(rects[1].height, 3);
    assert_eq!(rects[2].height, 15);
}

#[test]
fn installing_inner_layout_returns_continuous_y_offsets() {
    use ratatui::layout::Rect;
    let rects = installing_inner_layout(Rect::new(0, 0, 80, 22));
    assert_eq!(rects[0].y + rects[0].height, rects[1].y);
    assert_eq!(rects[1].y + rects[1].height, rects[2].y);
}

#[test]
fn installing_inner_layout_collapses_padding_when_height_is_minimum() {
    use ratatui::layout::Rect;
    let rects = installing_inner_layout(Rect::new(0, 0, 80, 7));
    assert_eq!(rects[0].height, 4);
    assert_eq!(rects[1].height, 3);
    assert_eq!(rects[2].height, 0);
}

#[test]
fn installing_inner_layout_does_not_panic_for_tiny_height() {
    use ratatui::layout::Rect;
    let _ = installing_inner_layout(Rect::new(0, 0, 80, 5));
}

// =============================================================================
// 描画テスト（`Clear, f.area()` の効果保証）
// =============================================================================

fn fill_buffer_with_sentinel(buffer: &mut ratatui::buffer::Buffer, sentinel: char) {
    let area = buffer.area;
    for y in area.y..area.bottom() {
        for x in area.x..area.right() {
            if let Some(cell) = buffer.cell_mut((x, y)) {
                cell.set_char(sentinel);
            }
        }
    }
}

#[test]
fn modal_clears_outer_frame_cells_at_normal_size_for_target_select() {
    use ratatui::backend::TestBackend;
    use ratatui::widgets::ListState;
    use ratatui::Terminal;

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("terminal");

    // 1 回目: 内部バッファを番兵で埋める
    terminal
        .draw(|frame| {
            fill_buffer_with_sentinel(frame.buffer_mut(), 'X');
        })
        .expect("seed sentinel draw");

    // 2 回目: 対象モーダルを描画
    let targets = vec![("codex".to_string(), "Codex".to_string(), false)];
    terminal
        .draw(|frame| {
            view_target_select(frame, &targets, 0, ListState::default());
        })
        .expect("render modal");

    let buffer = terminal.backend().buffer();
    let width = buffer.area.width;
    let height = buffer.area.height;
    // 最外周セルを走査し、番兵 'X' が残っていないことを確認
    for x in 0..width {
        let top = buffer[(x, 0)].symbol();
        let bottom = buffer[(x, height - 1)].symbol();
        assert_ne!(top, "X", "top row cell ({}, 0) still has sentinel", x);
        assert_ne!(
            bottom,
            "X",
            "bottom row cell ({}, {}) still has sentinel",
            x,
            height - 1
        );
    }
    for y in 0..height {
        let left = buffer[(0, y)].symbol();
        let right = buffer[(width - 1, y)].symbol();
        assert_ne!(left, "X", "left col cell (0, {}) still has sentinel", y);
        assert_ne!(
            right,
            "X",
            "right col cell ({}, {}) still has sentinel",
            width - 1,
            y
        );
    }
}

#[test]
fn modal_clears_outer_frame_cells_at_normal_size_for_install_result() {
    use super::{InstallSummary, PluginInstallOutcome};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("terminal");

    terminal
        .draw(|frame| {
            fill_buffer_with_sentinel(frame.buffer_mut(), 'X');
        })
        .expect("seed sentinel draw");

    let summary = InstallSummary {
        results: vec![PluginInstallOutcome {
            plugin_name: "alpha".to_string(),
            success: true,
            error: None,
        }],
        total: 1,
        succeeded: 1,
        failed: 0,
    };
    terminal
        .draw(|frame| {
            view_install_result(frame, &summary);
        })
        .expect("render modal");

    let buffer = terminal.backend().buffer();
    let width = buffer.area.width;
    let height = buffer.area.height;
    for x in 0..width {
        assert_ne!(buffer[(x, 0)].symbol(), "X");
        assert_ne!(buffer[(x, height - 1)].symbol(), "X");
    }
    for y in 0..height {
        assert_ne!(buffer[(0, y)].symbol(), "X");
        assert_ne!(buffer[(width - 1, y)].symbol(), "X");
    }
}

#[test]
fn modal_renders_without_panic_at_tiny_size() {
    use ratatui::backend::TestBackend;
    use ratatui::widgets::ListState;
    use ratatui::Terminal;

    let backend = TestBackend::new(10, 4);
    let mut terminal = Terminal::new(backend).expect("terminal");

    let targets = vec![("codex".to_string(), "Codex".to_string(), false)];
    terminal
        .draw(|frame| {
            view_target_select(frame, &targets, 0, ListState::default());
        })
        .expect("render target_select tiny");

    let plugin_names = vec!["alpha".to_string()];
    terminal
        .draw(|frame| {
            view_installing(frame, &plugin_names, 0, 1);
        })
        .expect("render installing tiny");
}
