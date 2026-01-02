//! コンポーネント種別選択画面の描画

use super::common::dialog_rect;
use crate::tui::manager::state::ManagerApp;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};

/// コンポーネント種別選択画面
pub(super) fn render_component_types_screen(f: &mut Frame, app: &mut ManagerApp, plugin_idx: usize) {
    let Some(detail) = app.plugin_detail(plugin_idx) else {
        return;
    };
    let type_counts = app.component_type_counts(plugin_idx);

    // ダイアログサイズを計算
    let base_lines = if detail.marketplace.is_some() { 4 } else { 3 };
    let type_lines = if type_counts.is_empty() { 1 } else { type_counts.len() };
    let content_height = (base_lines + type_lines) as u16 + 4;
    let dialog_width = 55u16;
    let dialog_height = content_height.min(15);

    let dialog_area = dialog_rect(dialog_width, dialog_height, f.area());
    f.render_widget(Clear, dialog_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(dialog_area);

    let title = format!(" {} ", detail.name);

    if type_counts.is_empty() {
        // コンポーネントがない場合は静的表示
        let mut lines = Vec::new();
        lines.push(format!("  Version: {}", detail.version));
        if let Some(marketplace) = &detail.marketplace {
            lines.push(format!("  Marketplace: {}", marketplace));
        }
        lines.push(String::new());
        lines.push("  Components: (none)".to_string());

        let paragraph = Paragraph::new(lines.join("\n"))
            .block(Block::default().title(title).borders(Borders::ALL));
        f.render_widget(paragraph, chunks[0]);
    } else {
        // コンポーネントがある場合はリスト選択可能
        let items: Vec<ListItem> = type_counts
            .iter()
            .map(|tc| {
                let text = format!("  {} ({})", tc.kind.title(), tc.count);
                ListItem::new(text)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().title(title).borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Green),
            )
            .highlight_symbol("> ");

        f.render_stateful_widget(list, chunks[0], &mut app.component_type_state);
    }

    let help_text = if type_counts.is_empty() {
        " Esc: back · q: quit"
    } else {
        " ↑/↓: move · Enter: open · Esc: back · q: quit"
    };
    let help = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[1]);
}
