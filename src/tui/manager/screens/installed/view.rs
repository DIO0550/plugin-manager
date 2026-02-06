//! Installed タブの view（描画）
//!
//! 各画面状態に応じた描画ロジック。

use super::model::{DetailAction, Model, UpdateStatusDisplay};
use crate::component::ComponentKind;
use crate::tui::manager::core::{
    dialog_rect, filter_plugins, render_filter_bar, DataStore, PluginId, Tab,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs};
use std::collections::{HashMap, HashSet};

/// ComponentKind の表示用タイトルを取得（複数形）
fn component_kind_title(kind: ComponentKind) -> &'static str {
    match kind {
        ComponentKind::Skill => "Skills",
        ComponentKind::Agent => "Agents",
        ComponentKind::Command => "Commands",
        ComponentKind::Instruction => "Instructions",
        ComponentKind::Hook => "Hooks",
    }
}

/// 画面を描画
pub fn view(
    f: &mut Frame,
    model: &Model,
    data: &DataStore,
    filter_text: &str,
    filter_focused: bool,
) {
    match model {
        Model::PluginList {
            state,
            marked_ids,
            update_statuses,
            ..
        } => {
            view_plugin_list(
                f,
                *state,
                data,
                filter_text,
                filter_focused,
                marked_ids,
                update_statuses,
            );
        }
        Model::PluginDetail {
            plugin_id, state, ..
        } => {
            view_plugin_detail(f, plugin_id, *state, data, filter_text, filter_focused);
        }
        Model::ComponentTypes {
            plugin_id, state, ..
        } => {
            view_component_types(f, plugin_id, *state, data, filter_text, filter_focused);
        }
        Model::ComponentList {
            plugin_id,
            kind,
            state,
            ..
        } => {
            view_component_list(
                f,
                plugin_id,
                *kind,
                *state,
                data,
                filter_text,
                filter_focused,
            );
        }
    }
}

/// 更新ステータスの表示文字列とスタイルを取得
fn update_status_span(status: &UpdateStatusDisplay) -> Span<'_> {
    match status {
        UpdateStatusDisplay::Updating => {
            Span::styled(" Updating...", Style::default().fg(Color::Yellow))
        }
        UpdateStatusDisplay::Updated => Span::styled(" Updated", Style::default().fg(Color::Green)),
        UpdateStatusDisplay::AlreadyUpToDate => {
            Span::styled(" Up to date", Style::default().fg(Color::DarkGray))
        }
        UpdateStatusDisplay::Skipped(reason) => {
            let text = format!(" Skipped: {}", reason);
            Span::styled(text, Style::default().fg(Color::DarkGray))
        }
        UpdateStatusDisplay::Failed(reason) => {
            let text = format!(" Failed: {}", reason);
            Span::styled(text, Style::default().fg(Color::Red))
        }
    }
}

/// プラグイン一覧画面を描画
#[allow(clippy::too_many_arguments)]
fn view_plugin_list(
    f: &mut Frame,
    mut state: ListState,
    data: &DataStore,
    filter_text: &str,
    filter_focused: bool,
    marked_ids: &HashSet<PluginId>,
    update_statuses: &HashMap<PluginId, UpdateStatusDisplay>,
) {
    let filtered = filter_plugins(&data.plugins, filter_text);
    let content_height = (filtered.len() as u16).max(1) + 9; // +3 for filter bar
    let dialog_width = 55u16;
    let dialog_height = content_height.min(24);

    let dialog_area = dialog_rect(dialog_width, dialog_height, f.area());
    f.render_widget(Clear, dialog_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // タブバー
            Constraint::Length(3), // フィルタバー
            Constraint::Min(1),    // コンテンツ
            Constraint::Length(1), // ヘルプ
        ])
        .split(dialog_area);

    // タブバー
    let tab_titles: Vec<&str> = Tab::all().iter().map(|t| t.title()).collect();
    let tabs = Tabs::new(tab_titles)
        .select(Tab::Installed.index())
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .divider(" | ");
    f.render_widget(tabs, chunks[0]);

    // フィルタバー
    render_filter_bar(f, chunks[1], filter_text, filter_focused);

    // タイトル（マーク数表示付き）
    let marked_count = marked_ids.len();
    let title = if marked_count > 0 {
        format!(
            " Installed Plugins ({}/{}) [{} marked] ",
            filtered.len(),
            data.plugins.len(),
            marked_count
        )
    } else {
        format!(
            " Installed Plugins ({}/{}) ",
            filtered.len(),
            data.plugins.len()
        )
    };

    // プラグインリスト（フィルタ済み）
    if filtered.is_empty() {
        let no_match = Paragraph::new("  No matching plugins")
            .block(
                Block::default()
                    .title(format!(" Installed Plugins (0/{}) ", data.plugins.len()))
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(no_match, chunks[2]);
    } else {
        let items: Vec<ListItem> = filtered
            .iter()
            .map(|p| {
                let is_marked = marked_ids.contains(&p.name);
                let mark_indicator = if is_marked { "[x] " } else { "[ ] " };

                let marketplace_str = p
                    .marketplace
                    .as_ref()
                    .map(|m| format!(" @{}", m))
                    .unwrap_or_default();
                let status_str = if p.enabled { "" } else { " [disabled]" };

                // 行のベーススタイル（マーク・disabled 状態に応じて統一）
                let base_style = if is_marked {
                    Style::default().fg(Color::Yellow)
                } else if p.enabled {
                    Style::default()
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                let mut spans = vec![];

                // マークインジケータ（ベーススタイルと統一）
                spans.push(Span::styled(format!("  {}", mark_indicator), base_style));

                // プラグイン名
                let name_text = format!(
                    "{}{}  v{}{}",
                    p.name, marketplace_str, p.version, status_str
                );
                spans.push(Span::styled(name_text, base_style));

                // 更新ステータス
                if let Some(update_status) = update_statuses.get(&p.name) {
                    spans.push(update_status_span(update_status));
                }

                ListItem::new(Line::from(spans))
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

        f.render_stateful_widget(list, chunks[2], &mut state);
    }

    // ヘルプ
    let help = Paragraph::new(
        " Space: mark | a: all | U: update | Tab: switch | ↑↓: move | Enter: details | q: quit",
    )
    .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[3]);
}

/// プラグイン詳細画面を描画
fn view_plugin_detail(
    f: &mut Frame,
    plugin_id: &PluginId,
    mut state: ListState,
    data: &DataStore,
    filter_text: &str,
    filter_focused: bool,
) {
    let Some(plugin) = data.find_plugin(plugin_id) else {
        return;
    };

    // ダイアログサイズ
    let dialog_width = 65u16;
    let dialog_height = 21u16; // +3 for filter bar
    let dialog_area = dialog_rect(dialog_width, dialog_height, f.area());
    f.render_widget(Clear, dialog_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // タブバー
            Constraint::Length(3), // フィルタバー
            Constraint::Length(7), // プラグイン情報
            Constraint::Min(1),    // アクションメニュー
            Constraint::Length(1), // ヘルプ
        ])
        .split(dialog_area);

    // タブバー
    let tab_titles: Vec<&str> = Tab::all().iter().map(|t| t.title()).collect();
    let tabs = Tabs::new(tab_titles)
        .select(Tab::Installed.index())
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .divider(" | ");
    f.render_widget(tabs, chunks[0]);

    // フィルタバー（read-only）
    render_filter_bar(f, chunks[1], filter_text, filter_focused);

    // プラグイン情報
    let marketplace_str = plugin
        .marketplace
        .as_ref()
        .map(|m| format!(" @ {}", m))
        .unwrap_or_default();
    let title = format!(" {}{} ", plugin.name, marketplace_str);

    let (status_text, status_color) = if plugin.enabled {
        ("Enabled", Color::Green)
    } else {
        ("Disabled", Color::DarkGray)
    };

    let info_lines = vec![
        Line::from(vec![
            Span::raw("Scope: "),
            Span::styled("project", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::raw("Version: "),
            Span::styled(&plugin.version, Style::default().fg(Color::White)),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::raw("Author: "),
            Span::styled("N/A", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::raw("Status: "),
            Span::styled(status_text, Style::default().fg(status_color)),
        ]),
    ];

    let info_block = Block::default().title(title).borders(Borders::ALL);
    let info_para = Paragraph::new(info_lines).block(info_block);
    f.render_widget(info_para, chunks[2]);

    // アクションメニュー（enabled 状態に応じて動的に切り替え）
    let actions = DetailAction::for_plugin(plugin.enabled);
    let items: Vec<ListItem> = actions
        .iter()
        .map(|a| ListItem::new(format!("  {}", a.label())).style(a.style()))
        .collect();

    let list = List::new(items)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_stateful_widget(list, chunks[3], &mut state);

    // ヘルプ
    let help = Paragraph::new(" Navigate: ↑↓ • Select: Enter • Back: Esc")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[4]);
}

/// コンポーネント種別選択画面を描画
fn view_component_types(
    f: &mut Frame,
    plugin_id: &PluginId,
    mut state: ListState,
    data: &DataStore,
    filter_text: &str,
    filter_focused: bool,
) {
    let Some(plugin) = data.find_plugin(plugin_id) else {
        return;
    };

    let counts = data.available_component_kinds(plugin);
    let has_marketplace = plugin.marketplace.is_some();
    let base_lines = if has_marketplace { 4 } else { 3 };
    let type_lines = if counts.is_empty() { 1 } else { counts.len() };
    let content_height = (base_lines + type_lines) as u16 + 7; // +3 for filter bar
    let dialog_width = 55u16;
    let dialog_height = content_height.min(18);

    let dialog_area = dialog_rect(dialog_width, dialog_height, f.area());
    f.render_widget(Clear, dialog_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // フィルタバー
            Constraint::Min(1),    // コンテンツ
            Constraint::Length(1), // ヘルプ
        ])
        .split(dialog_area);

    // フィルタバー（read-only）
    render_filter_bar(f, chunks[0], filter_text, filter_focused);

    let title = format!(" {} ", plugin.name);

    if counts.is_empty() {
        // コンポーネントがない場合
        let mut lines = Vec::new();
        lines.push(format!("  Version: {}", plugin.version));
        if let Some(marketplace) = &plugin.marketplace {
            lines.push(format!("  Marketplace: {}", marketplace));
        }
        lines.push(String::new());
        lines.push("  Components: (none)".to_string());

        let paragraph = Paragraph::new(lines.join("\n"))
            .block(Block::default().title(title).borders(Borders::ALL));
        f.render_widget(paragraph, chunks[1]);
    } else {
        // コンポーネントがある場合
        let items: Vec<ListItem> = counts
            .iter()
            .map(|count| {
                let text = format!("  {} ({})", count.title(), count.count);
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

        f.render_stateful_widget(list, chunks[1], &mut state);
    }

    // ヘルプ
    let help_text = if counts.is_empty() {
        " Esc: back | q: quit"
    } else {
        " ↑↓: move | Enter: open | Esc: back | q: quit"
    };
    let help = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[2]);
}

/// コンポーネント一覧画面を描画
fn view_component_list(
    f: &mut Frame,
    plugin_id: &PluginId,
    kind: ComponentKind,
    mut state: ListState,
    data: &DataStore,
    filter_text: &str,
    filter_focused: bool,
) {
    let Some(plugin) = data.find_plugin(plugin_id) else {
        return;
    };

    let components = data.component_names(plugin, kind);
    let items: Vec<ListItem> = components
        .iter()
        .map(|c| ListItem::new(format!("  {}", c.name)))
        .collect();

    let content_height = (components.len() as u16).max(1) + 7; // +3 for filter bar
    let dialog_width = 55u16;
    let dialog_height = content_height.min(23);

    let dialog_area = dialog_rect(dialog_width, dialog_height, f.area());
    f.render_widget(Clear, dialog_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // フィルタバー
            Constraint::Min(1),    // コンテンツ
            Constraint::Length(1), // ヘルプ
        ])
        .split(dialog_area);

    // フィルタバー（read-only）
    render_filter_bar(f, chunks[0], filter_text, filter_focused);

    let title = format!(
        " {} > {} ({}) ",
        plugin.name,
        component_kind_title(kind),
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

    f.render_stateful_widget(list, chunks[1], &mut state);

    // ヘルプ
    let help = Paragraph::new(" ↑↓: move | Esc: back | q: quit")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[2]);
}
