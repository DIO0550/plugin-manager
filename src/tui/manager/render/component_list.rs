//! コンポーネント一覧画面の描画

use super::common::dialog_rect;
use crate::tui::manager::state::ManagerApp;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};

/// コンポーネント一覧画面
pub(super) fn render_component_list_screen(
    f: &mut Frame,
    app: &mut ManagerApp,
    plugin_idx: usize,
    type_idx: usize,
) {
    let Some(detail) = app.plugin_detail(plugin_idx) else {
        return;
    };
    let Some(kind) = app.component_type_at(plugin_idx, type_idx) else {
        return;
    };
    let names = app.component_names(plugin_idx, kind);

    let items: Vec<ListItem> = names
        .iter()
        .map(|c| ListItem::new(format!("  {}", c.name)))
        .collect();

    // ダイアログサイズ
    let content_height = (names.len() as u16).max(1) + 4;
    let dialog_width = 55u16;
    let dialog_height = content_height.min(20);

    let dialog_area = dialog_rect(dialog_width, dialog_height, f.area());
    f.render_widget(Clear, dialog_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(dialog_area);

    let title = format!(
        " {} > {} ({}) ",
        detail.name,
        kind.title(),
        names.len()
    );
    let list = List::new(items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Green),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, chunks[0], &mut app.component_state);

    let help = Paragraph::new(" ↑/↓: move · Esc: back · q: quit")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[1]);
}
