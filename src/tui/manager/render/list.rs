//! プラグイン一覧画面の描画

use super::common::dialog_rect;
use crate::tui::manager::state::{ManagerApp, ManagerTab};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs};

/// プラグイン一覧画面
pub(super) fn render_list_screen(f: &mut Frame, app: &mut ManagerApp) {
    // タブに応じたコンテンツ高さを計算
    let content_height = match app.current_tab {
        ManagerTab::Installed => (app.installed_plugin_count() as u16).max(1) + 6,
        _ => 8, // placeholder tabs
    };
    let dialog_width = 55u16;
    let dialog_height = content_height.min(22);

    let dialog_area = dialog_rect(dialog_width, dialog_height, f.area());

    // 背景クリア
    f.render_widget(Clear, dialog_area);

    // レイアウト（タブ + コンテンツ + ヘルプ）
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // タブバー
            Constraint::Min(1),    // コンテンツ
            Constraint::Length(1), // ヘルプ
        ])
        .split(dialog_area);

    // タブバー
    let tab_titles: Vec<&str> = ManagerTab::all().iter().map(|t| t.title()).collect();
    let tabs = Tabs::new(tab_titles)
        .select(app.current_tab.index())
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .divider(" | ");
    f.render_widget(tabs, chunks[0]);

    // タブに応じたコンテンツを表示
    match app.current_tab {
        ManagerTab::Installed => render_installed_tab(f, app, chunks[1]),
        ManagerTab::Discover => {
            render_placeholder_tab(f, "Discover", "Browse available plugins", chunks[1])
        }
        ManagerTab::Marketplaces => {
            render_placeholder_tab(f, "Marketplaces", "Manage marketplace sources", chunks[1])
        }
        ManagerTab::Errors => render_placeholder_tab(f, "Errors", "No errors", chunks[1]),
    }

    // ヘルプ
    let help_text = match app.current_tab {
        ManagerTab::Installed => " Tab: switch · ↑/↓: move · Enter: details · q: quit",
        _ => " Tab: switch · q: quit",
    };
    let help = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[2]);
}

/// Installed タブのコンテンツ
fn render_installed_tab(f: &mut Frame, app: &mut ManagerApp, area: Rect) {
    let summaries = app.installed_plugin_summaries();
    let items: Vec<ListItem> = summaries
        .iter()
        .map(|s| {
            let marketplace_str = s
                .marketplace
                .as_ref()
                .map(|m| format!(" @{}", m))
                .unwrap_or_default();
            let text = format!("  {}{}  v{}", s.name, marketplace_str, s.version);
            ListItem::new(text)
        })
        .collect();

    let title = format!(" Installed Plugins ({}) ", app.installed_plugin_count());
    let list = List::new(items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Green),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, &mut app.list_state);
}

/// プレースホルダータブのコンテンツ
fn render_placeholder_tab(f: &mut Frame, title: &str, message: &str, area: Rect) {
    let content = Paragraph::new(format!("\n  {}", message))
        .block(
            Block::default()
                .title(format!(" {} ", title))
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(content, area);
}
