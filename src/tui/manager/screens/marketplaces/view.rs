//! Marketplaces タブの view（描画）

use super::model::{AddFormModel, DetailAction, Model, OperationStatus};
use crate::tui::manager::core::{dialog_rect, render_filter_bar, DataStore, Tab};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs};

/// 画面を描画
pub fn view(
    f: &mut Frame,
    model: &Model,
    data: &DataStore,
    filter_text: &str,
    filter_focused: bool,
) {
    match model {
        Model::MarketList {
            state,
            operation_status,
            error_message,
            ..
        } => {
            view_market_list(
                f,
                *state,
                data,
                filter_text,
                filter_focused,
                operation_status,
                error_message,
            );
        }
        Model::MarketDetail {
            marketplace_name,
            state,
            error_message,
        } => {
            view_market_detail(
                f,
                marketplace_name,
                *state,
                data,
                filter_text,
                filter_focused,
                error_message,
            );
        }
        Model::PluginList {
            marketplace_name,
            state,
            plugins,
            ..
        } => {
            view_plugin_list(
                f,
                marketplace_name,
                *state,
                plugins,
                filter_text,
                filter_focused,
            );
        }
        Model::AddForm(form) => {
            view_add_form(f, form, filter_text, filter_focused);
        }
    }
}

/// タブバーを描画
fn render_tab_bar(f: &mut Frame, area: Rect) {
    let tab_titles: Vec<&str> = Tab::all().iter().map(|t| t.title()).collect();
    let tabs = Tabs::new(tab_titles)
        .select(Tab::Marketplaces.index())
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .divider(" | ");
    f.render_widget(tabs, area);
}

/// マーケットプレイス一覧画面を描画
#[allow(clippy::too_many_arguments)]
fn view_market_list(
    f: &mut Frame,
    mut state: ListState,
    data: &DataStore,
    filter_text: &str,
    filter_focused: bool,
    operation_status: &Option<OperationStatus>,
    error_message: &Option<String>,
) {
    // リスト長: マーケットプレイス数 + 1（"+ Add new"）
    let list_len = data.marketplaces.len() + 1;
    let has_status = operation_status.is_some();
    let has_error = error_message.is_some();
    let extra_lines = if has_status { 1 } else { 0 } + if has_error { 1 } else { 0 };

    let content_height = (list_len as u16).max(1) + 2 + extra_lines; // +2 for borders
    let dialog_width = 65u16;
    let dialog_height = (content_height + 5).min(24); // +5 for tab+filter+help

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
    render_tab_bar(f, chunks[0]);

    // フィルタバー
    render_filter_bar(f, chunks[1], filter_text, filter_focused);

    // コンテンツ領域を分割（リスト + ステータス/エラー）
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),                            // リスト
            Constraint::Length(extra_lines.max(0) as u16), // ステータス/エラー
        ])
        .split(chunks[2]);

    // マーケットプレイスリスト
    let title = format!(" Marketplaces ({}) ", data.marketplaces.len());
    let mut items: Vec<ListItem> = data
        .marketplaces
        .iter()
        .map(|m| {
            let plugin_info = m
                .plugin_count
                .map(|c| format!("{} plugins", c))
                .unwrap_or_else(|| "no cache".to_string());
            let updated_info = m.last_updated.as_deref().unwrap_or("-");
            let source_path_info = m
                .source_path
                .as_ref()
                .map(|p| format!(" ({})", p))
                .unwrap_or_default();

            let text = format!(
                "  {}    {}{}    {}    {}",
                m.name, m.source, source_path_info, plugin_info, updated_info
            );
            ListItem::new(text)
        })
        .collect();

    // "+ Add new marketplace" 項目を末尾に追加
    items.push(ListItem::new("  + Add new marketplace").style(Style::default().fg(Color::Cyan)));

    let list = List::new(items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Green),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, content_chunks[0], &mut state);

    // ステータス/エラー表示
    if has_status || has_error {
        let mut lines = Vec::new();
        if let Some(status) = operation_status {
            let status_text = match status {
                OperationStatus::Updating(name) => format!(" Updating marketplace '{}'...", name),
                OperationStatus::UpdatingAll => " Updating all marketplaces...".to_string(),
                OperationStatus::Removing(name) => format!(" Removing marketplace '{}'...", name),
            };
            lines.push(Line::from(Span::styled(
                status_text,
                Style::default().fg(Color::Yellow),
            )));
        }
        if let Some(error) = error_message {
            lines.push(Line::from(Span::styled(
                format!(" {}", error),
                Style::default().fg(Color::Red),
            )));
        }
        let status_para = Paragraph::new(lines);
        f.render_widget(status_para, content_chunks[1]);
    }

    // ヘルプ
    let help = Paragraph::new(
        " u: update | U: update all | Tab: switch | ↑↓: move | Enter: select | q: quit",
    )
    .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[3]);
}

/// マーケットプレイス詳細画面を描画
#[allow(clippy::too_many_arguments)]
fn view_market_detail(
    f: &mut Frame,
    marketplace_name: &str,
    mut state: ListState,
    data: &DataStore,
    filter_text: &str,
    filter_focused: bool,
    error_message: &Option<String>,
) {
    let dialog_width = 65u16;
    let dialog_height = 20u16;

    let dialog_area = dialog_rect(dialog_width, dialog_height, f.area());
    f.render_widget(Clear, dialog_area);

    let has_error = error_message.is_some();
    let error_height = if has_error { 1 } else { 0 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),            // タブバー
            Constraint::Length(3),            // フィルタバー
            Constraint::Length(6),            // マーケットプレイス情報
            Constraint::Min(1),               // アクションメニュー
            Constraint::Length(error_height), // エラー
            Constraint::Length(1),            // ヘルプ
        ])
        .split(dialog_area);

    // タブバー
    render_tab_bar(f, chunks[0]);

    // フィルタバー（read-only）
    render_filter_bar(f, chunks[1], filter_text, filter_focused);

    // マーケットプレイス情報
    let marketplace = data.find_marketplace(marketplace_name);
    let title = format!(" {} ", marketplace_name);

    let info_lines = if let Some(m) = marketplace {
        let mut lines = vec![Line::from(vec![
            Span::raw("  Source: "),
            Span::styled(&m.source, Style::default().fg(Color::White)),
        ])];
        if let Some(path) = &m.source_path {
            lines.push(Line::from(vec![
                Span::raw("  Path: "),
                Span::styled(path, Style::default().fg(Color::White)),
            ]));
        }
        lines.push(Line::from(vec![
            Span::raw("  Plugins: "),
            Span::styled(
                m.plugin_count
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "N/A".to_string()),
                Style::default().fg(Color::White),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::raw("  Last updated: "),
            Span::styled(
                m.last_updated.as_deref().unwrap_or("never"),
                Style::default().fg(Color::White),
            ),
        ]));
        lines
    } else {
        vec![Line::from(Span::styled(
            "  Marketplace not found",
            Style::default().fg(Color::Red),
        ))]
    };

    let info_block = Block::default().title(title).borders(Borders::ALL);
    let info_para = Paragraph::new(info_lines).block(info_block);
    f.render_widget(info_para, chunks[2]);

    // アクションメニュー
    let actions = DetailAction::all();
    let items: Vec<ListItem> = actions
        .iter()
        .map(|a| ListItem::new(format!("  {}", a.label())).style(a.style()))
        .collect();

    let list = List::new(items)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_stateful_widget(list, chunks[3], &mut state);

    // エラー表示
    if let Some(error) = error_message {
        let error_para =
            Paragraph::new(format!(" {}", error)).style(Style::default().fg(Color::Red));
        f.render_widget(error_para, chunks[4]);
    }

    // ヘルプ
    let help = Paragraph::new(" ↑↓: move | Enter: select | Esc: back")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[5]);
}

/// プラグイン一覧画面を描画
fn view_plugin_list(
    f: &mut Frame,
    marketplace_name: &str,
    mut state: ListState,
    plugins: &[(String, Option<String>)],
    filter_text: &str,
    filter_focused: bool,
) {
    let content_height = (plugins.len() as u16).max(1) + 2; // +2 for borders
    let dialog_width = 65u16;
    let dialog_height = (content_height + 5).min(24);

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
    render_tab_bar(f, chunks[0]);

    // フィルタバー（read-only）
    render_filter_bar(f, chunks[1], filter_text, filter_focused);

    // プラグインリスト
    let title = format!(" {} > Plugins ({}) ", marketplace_name, plugins.len());

    if plugins.is_empty() {
        // 空リスト表示
        let msg = "  No cached data. Run update to fetch plugins.";
        let content = Paragraph::new(msg)
            .block(Block::default().title(title).borders(Borders::ALL))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(content, chunks[2]);
    } else {
        let items: Vec<ListItem> = plugins
            .iter()
            .map(|(name, desc)| {
                let text = if let Some(description) = desc {
                    format!("  {} - {}", name, description)
                } else {
                    format!("  {}", name)
                };
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

        f.render_stateful_widget(list, chunks[2], &mut state);
    }

    // ヘルプ
    let help = Paragraph::new(" ↑↓: move | Esc: back").style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[3]);
}

/// 追加フォーム画面を描画
fn view_add_form(f: &mut Frame, form: &AddFormModel, filter_text: &str, filter_focused: bool) {
    let dialog_width = 65u16;
    let dialog_height = 15u16;

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
    render_tab_bar(f, chunks[0]);

    // フィルタバー（read-only）
    render_filter_bar(f, chunks[1], filter_text, filter_focused);

    // フォームコンテンツ
    match form {
        AddFormModel::Source {
            source_input,
            error_message,
        } => {
            let mut lines = vec![
                Line::raw(""),
                Line::from(vec![Span::raw("  Step 1/3: Enter source")]),
                Line::raw(""),
                Line::from(vec![
                    Span::raw("  Source (owner/repo): "),
                    Span::styled(
                        format!("{}|", source_input),
                        Style::default().fg(Color::White),
                    ),
                ]),
            ];
            if let Some(error) = error_message {
                lines.push(Line::raw(""));
                lines.push(Line::from(Span::styled(
                    format!("  {}", error),
                    Style::default().fg(Color::Red),
                )));
            }
            let content = Paragraph::new(lines).block(
                Block::default()
                    .title(" Add Marketplace ")
                    .borders(Borders::ALL),
            );
            f.render_widget(content, chunks[2]);
        }
        AddFormModel::Name {
            source,
            name_input,
            default_name,
            error_message,
        } => {
            let mut lines = vec![
                Line::raw(""),
                Line::from(vec![Span::raw("  Step 2/3: Enter name")]),
                Line::raw(""),
                Line::from(vec![
                    Span::raw("  Source: "),
                    Span::styled(source, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::raw("  Name: "),
                    Span::styled(
                        format!("{}|", name_input),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(vec![Span::styled(
                    format!("  (Enter for default: {})", default_name),
                    Style::default().fg(Color::DarkGray),
                )]),
            ];
            if let Some(error) = error_message {
                lines.push(Line::raw(""));
                lines.push(Line::from(Span::styled(
                    format!("  {}", error),
                    Style::default().fg(Color::Red),
                )));
            }
            let content = Paragraph::new(lines).block(
                Block::default()
                    .title(" Add Marketplace ")
                    .borders(Borders::ALL),
            );
            f.render_widget(content, chunks[2]);
        }
        AddFormModel::Confirm {
            source,
            name,
            error_message,
        } => {
            let mut lines = vec![
                Line::raw(""),
                Line::from(vec![Span::raw("  Step 3/3: Confirm")]),
                Line::raw(""),
                Line::from(vec![
                    Span::raw("  Source: "),
                    Span::styled(source, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::raw("  Name: "),
                    Span::styled(name, Style::default().fg(Color::White)),
                ]),
                Line::raw(""),
                Line::from(vec![Span::styled(
                    "  Press Enter to add, Esc to cancel",
                    Style::default().fg(Color::DarkGray),
                )]),
            ];
            if let Some(error) = error_message {
                lines.push(Line::raw(""));
                lines.push(Line::from(Span::styled(
                    format!("  {}", error),
                    Style::default().fg(Color::Red),
                )));
            }
            let content = Paragraph::new(lines).block(
                Block::default()
                    .title(" Add Marketplace ")
                    .borders(Borders::ALL),
            );
            f.render_widget(content, chunks[2]);
        }
    }

    // ヘルプ
    let help = Paragraph::new(" Type to input | Enter: next | Esc: cancel")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[3]);
}
