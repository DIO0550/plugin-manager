//! プラグイン管理 TUI の描画処理
//!
//! 各画面のレンダリング。

use super::state::{ManagerApp, ManagerComponentType, ManagerScreen, ManagerTab};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs};

/// コンテンツに合わせたダイアログ領域を計算（左寄せ）
fn dialog_rect(width: u16, height: u16, area: Rect) -> Rect {
    Rect::new(area.x, area.y, width.min(area.width), height.min(area.height))
}

/// UI をレンダリング
pub(super) fn draw(f: &mut Frame, app: &mut ManagerApp) {
    // 背景をクリア
    f.render_widget(Clear, f.area());

    match &app.screen {
        ManagerScreen::PluginList => render_list_screen(f, app),
        ManagerScreen::ComponentTypes(plugin_idx) => render_component_types_screen(f, app, *plugin_idx),
        ManagerScreen::ComponentList(plugin_idx, type_idx) => {
            render_component_list_screen(f, app, *plugin_idx, *type_idx)
        }
    }
}

/// プラグイン一覧画面
fn render_list_screen(f: &mut Frame, app: &mut ManagerApp) {
    // タブに応じたコンテンツ高さを計算
    let content_height = match app.current_tab {
        ManagerTab::Installed => (app.plugins.len() as u16).max(1) + 6,
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
        ManagerTab::Discover => render_placeholder_tab(f, "Discover", "Browse available plugins", chunks[1]),
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
    let items: Vec<ListItem> = app
        .plugins
        .iter()
        .map(|plugin| {
            let marketplace_str = plugin
                .marketplace
                .as_ref()
                .map(|m| format!(" @{}", m))
                .unwrap_or_default();
            let text = format!("  {}{}  v{}", plugin.name, marketplace_str, plugin.version);
            ListItem::new(text)
        })
        .collect();

    let title = format!(" Installed Plugins ({}) ", app.plugins.len());
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

/// コンポーネント種別選択画面
fn render_component_types_screen(f: &mut Frame, app: &mut ManagerApp, plugin_idx: usize) {
    let plugin = match app.plugins.get(plugin_idx) {
        Some(p) => p,
        None => return,
    };

    // プラグイン情報 + コンポーネント種別
    let mut lines = Vec::new();
    lines.push(format!("  Version: {}", plugin.version));
    if let Some(marketplace) = &plugin.marketplace {
        lines.push(format!("  Marketplace: {}", marketplace));
    }
    lines.push(String::new());

    let types = app.available_types(plugin);
    if types.is_empty() {
        lines.push("  Components: (none)".to_string());
    } else {
        lines.push("  Components:".to_string());
        for t in &types {
            let count = match t {
                ManagerComponentType::Skills => plugin.skills.len(),
                ManagerComponentType::Agents => plugin.agents.len(),
                ManagerComponentType::Commands => plugin.commands.len(),
                ManagerComponentType::Instructions => plugin.instructions.len(),
                ManagerComponentType::Hooks => plugin.hooks.len(),
            };
            lines.push(format!("    {} ({})", t.title(), count));
        }
    }

    // ダイアログサイズ
    let content_height = (lines.len() as u16) + 4;
    let dialog_width = 55u16;
    let dialog_height = content_height.min(15);

    let dialog_area = dialog_rect(dialog_width, dialog_height, f.area());
    f.render_widget(Clear, dialog_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(dialog_area);

    let title = format!(" {} ", plugin.name);
    let content = lines.join("\n");

    if types.is_empty() {
        // コンポーネントがない場合は静的表示
        let paragraph = Paragraph::new(content)
            .block(Block::default().title(title).borders(Borders::ALL));
        f.render_widget(paragraph, chunks[0]);
    } else {
        // コンポーネントがある場合はリスト選択可能
        let items: Vec<ListItem> = types
            .iter()
            .map(|t| {
                let count = match t {
                    ManagerComponentType::Skills => plugin.skills.len(),
                    ManagerComponentType::Agents => plugin.agents.len(),
                    ManagerComponentType::Commands => plugin.commands.len(),
                    ManagerComponentType::Instructions => plugin.instructions.len(),
                    ManagerComponentType::Hooks => plugin.hooks.len(),
                };
                let text = format!("  {} ({})", t.title(), count);
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

    let help_text = if types.is_empty() {
        " Esc: back · q: quit"
    } else {
        " ↑/↓: move · Enter: open · Esc: back · q: quit"
    };
    let help = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[1]);
}

/// コンポーネント一覧画面
fn render_component_list_screen(f: &mut Frame, app: &mut ManagerApp, plugin_idx: usize, type_idx: usize) {
    let plugin = match app.plugins.get(plugin_idx) {
        Some(p) => p,
        None => return,
    };

    let types = app.available_types(plugin);
    let comp_type = match types.get(type_idx) {
        Some(t) => *t,
        None => return,
    };

    let components = app.get_components(plugin, comp_type);
    let items: Vec<ListItem> = components
        .iter()
        .map(|name| ListItem::new(format!("  {}", name)))
        .collect();

    // ダイアログサイズ
    let content_height = (components.len() as u16).max(1) + 4;
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
        plugin.name,
        comp_type.title(),
        components.len()
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
