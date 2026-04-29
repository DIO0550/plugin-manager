//! Marketplaces タブの view（描画）

use super::model::{
    AddFormModel, BrowsePlugin, DetailAction, InstallSummary, Model, OperationStatus,
    PluginInstallResult,
};
use crate::component::Scope;
use crate::marketplace::PluginSource;
use crate::tui::manager::core::layout::{
    framed_layout, modal_layout, outer_rect, split_horizontal,
};
use crate::tui::manager::core::style::{
    bordered_block, menu_list, selectable_list, CHECKBOX_SELECTED, CHECKBOX_UNSELECTED,
    LIST_ITEM_INDENT, MARK_MARKED, RADIO_SELECTED, RADIO_UNSELECTED,
};
use crate::tui::manager::core::{
    render_filter_bar, truncate_for_list, truncate_for_paragraph, DataStore, Tab,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Gauge, ListItem, ListState, Paragraph, Tabs};
use std::collections::HashSet;

/// 描画用共通コンテキスト（DataStore + フィルタ情報）
struct ViewCtx<'a> {
    data: &'a DataStore,
    filter_text: &'a str,
    filter_focused: bool,
}

/// フィルタ情報のみのコンテキスト
struct FilterCtx<'a> {
    text: &'a str,
    focused: bool,
}

/// ブラウズ画面のデータ
struct BrowseData<'a> {
    plugins: &'a [BrowsePlugin],
    selected_plugins: &'a HashSet<String>,
    highlighted_idx: usize,
}

/// 画面を描画
///
/// # Arguments
///
/// * `f` - Ratatui frame to draw into.
/// * `model` - Marketplaces tab model to render.
/// * `data` - Shared data store for marketplaces.
/// * `filter_text` - Current filter input text.
/// * `filter_focused` - Whether the filter bar currently has focus.
pub fn view(
    f: &mut Frame,
    model: &Model,
    data: &DataStore,
    filter_text: &str,
    filter_focused: bool,
) {
    let ctx = ViewCtx {
        data,
        filter_text,
        filter_focused,
    };
    let filter = FilterCtx {
        text: filter_text,
        focused: filter_focused,
    };
    match model {
        Model::MarketList {
            state,
            operation_status,
            error_message,
            ..
        } => {
            view_market_list(f, *state, &ctx, operation_status, error_message);
        }
        Model::MarketDetail {
            marketplace_name,
            state,
            error_message,
            ..
        } => {
            view_market_detail(f, marketplace_name, *state, &ctx, error_message);
        }
        Model::PluginList {
            marketplace_name,
            state,
            plugins,
            ..
        } => {
            view_plugin_list(f, marketplace_name, *state, plugins, &filter);
        }
        Model::AddForm(form) => {
            view_add_form(f, form, filter_text, filter_focused);
        }
        Model::PluginBrowse {
            marketplace_name,
            plugins,
            selected_plugins,
            highlighted_idx,
            state,
        } => {
            let browse = BrowseData {
                plugins,
                selected_plugins,
                highlighted_idx: *highlighted_idx,
            };
            view_plugin_browse(f, marketplace_name, &browse, *state, &filter);
        }
        Model::TargetSelect {
            targets,
            highlighted_idx,
            state,
            ..
        } => {
            view_target_select(f, targets, *highlighted_idx, *state);
        }
        Model::ScopeSelect {
            highlighted_idx,
            state,
            ..
        } => {
            view_scope_select(f, *highlighted_idx, *state);
        }
        Model::Installing {
            plugin_names,
            current_idx,
            total,
            ..
        } => {
            view_installing(f, plugin_names, *current_idx, *total);
        }
        Model::InstallResult { summary, .. } => {
            view_install_result(f, summary);
        }
    }
}

/// タブバーを描画
///
/// # Arguments
///
/// * `f` - Ratatui frame to draw into.
/// * `area` - Target rectangle for the tab bar.
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
///
/// # Arguments
///
/// * `f` - Ratatui frame to draw into.
/// * `state` - List state used for highlight/selection.
/// * `ctx` - Shared view context (data store + filter state).
/// * `operation_status` - In-flight operation status shown below the list.
/// * `error_message` - Latest error message shown below the list.
fn view_market_list(
    f: &mut Frame,
    mut state: ListState,
    ctx: &ViewCtx<'_>,
    operation_status: &Option<OperationStatus>,
    error_message: &Option<String>,
) {
    let has_status = operation_status.is_some();
    let has_error = error_message.is_some();
    let extra_lines = if has_status { 1 } else { 0 } + if has_error { 1 } else { 0 };

    let outer = outer_rect(f.area());
    f.render_widget(Clear, outer);

    let [tabs_area, filter_area, content_area, help_area] = framed_layout(outer);

    // タブバー
    render_tab_bar(f, tabs_area);

    // フィルタバー
    render_filter_bar(f, filter_area, ctx.filter_text, ctx.filter_focused);

    // コンテンツ領域を分割（リスト + ステータス/エラー）
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),              // リスト
            Constraint::Length(extra_lines), // ステータス/エラー
        ])
        .split(content_area);

    // マーケットプレイスリスト
    let title = format!(" Marketplaces ({}) ", ctx.data.marketplaces.len());
    let mut items: Vec<ListItem> = ctx
        .data
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

            let raw = format!(
                "{}{}    {}{}    {}    {}",
                LIST_ITEM_INDENT, m.name, m.source, source_path_info, plugin_info, updated_info
            );
            let line_text = truncate_for_list(outer.width, raw).into_owned();
            ListItem::new(vec![Line::from(line_text), Line::raw("")])
        })
        .collect();

    // "+ Add new marketplace" 項目を末尾に追加
    items.push(
        ListItem::new(vec![
            Line::from(format!("{}+ Add new marketplace", LIST_ITEM_INDENT)),
            Line::raw(""),
        ])
        .style(Style::default().fg(Color::Cyan)),
    );

    let list = selectable_list(items, &title);

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
    f.render_widget(help, help_area);
}

/// マーケットプレイス詳細画面を描画
///
/// # Arguments
///
/// * `f` - Ratatui frame to draw into.
/// * `marketplace_name` - Marketplace whose detail should be shown.
/// * `state` - List state used for action menu highlight.
/// * `ctx` - Shared view context (data store + filter state).
/// * `error_message` - Latest error message shown below the menu.
fn view_market_detail(
    f: &mut Frame,
    marketplace_name: &str,
    mut state: ListState,
    ctx: &ViewCtx<'_>,
    error_message: &Option<String>,
) {
    let outer = outer_rect(f.area());
    f.render_widget(Clear, outer);

    let has_error = error_message.is_some();
    let error_height = if has_error { 1 } else { 0 };

    let [tabs_area, filter_area, content_area, help_area] = framed_layout(outer);

    // タブバー
    render_tab_bar(f, tabs_area);

    // フィルタバー（read-only）
    render_filter_bar(f, filter_area, ctx.filter_text, ctx.filter_focused);

    // アクションメニュー（先に組み立て、描画行数を算出して area を確保する）
    let actions = DetailAction::all();
    let (items, action_menu_rows) = build_market_action_menu(actions);

    // コンテンツ領域を画面固有に再分割（info / action_menu / error）
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),                   // マーケットプレイス情報
            Constraint::Length(action_menu_rows), // アクションメニュー
            Constraint::Length(error_height),     // エラー
        ])
        .split(content_area);

    // マーケットプレイス情報
    let marketplace = ctx.data.find_marketplace(marketplace_name);
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

    let info_para = Paragraph::new(info_lines).block(bordered_block(&title));
    f.render_widget(info_para, content_chunks[0]);

    let list = menu_list(items);

    f.render_stateful_widget(list, content_chunks[1], &mut state);

    // エラー表示
    if let Some(error) = error_message {
        let error_para =
            Paragraph::new(format!(" {}", error)).style(Style::default().fg(Color::Red));
        f.render_widget(error_para, content_chunks[2]);
    }

    // ヘルプ
    let help = Paragraph::new(" ↑↓: move | Enter: select | Esc: back")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, help_area);
}

/// プラグイン一覧画面を描画
///
/// # Arguments
///
/// * `f` - Ratatui frame to draw into.
/// * `marketplace_name` - Marketplace owning the plugins.
/// * `state` - List state used for plugin selection.
/// * `plugins` - Cached plugin name/description pairs.
/// * `filter` - Filter input context for the read-only filter bar.
fn view_plugin_list(
    f: &mut Frame,
    marketplace_name: &str,
    mut state: ListState,
    plugins: &[(String, Option<String>)],
    filter: &FilterCtx<'_>,
) {
    let outer = outer_rect(f.area());
    f.render_widget(Clear, outer);

    let [tabs_area, filter_area, content_area, help_area] = framed_layout(outer);

    // タブバー
    render_tab_bar(f, tabs_area);

    // フィルタバー（read-only）
    render_filter_bar(f, filter_area, filter.text, filter.focused);

    // プラグインリスト
    let title = format!(" {} > Plugins ({}) ", marketplace_name, plugins.len());

    if plugins.is_empty() {
        // 空リスト表示
        let msg = "  No cached data. Run update to fetch plugins.";
        let content = Paragraph::new(msg)
            .block(bordered_block(&title))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(content, content_area);
    } else {
        let items: Vec<ListItem> = plugins
            .iter()
            .map(|(name, desc)| {
                let raw = if let Some(description) = desc {
                    format!("{}{} - {}", LIST_ITEM_INDENT, name, description)
                } else {
                    format!("{}{}", LIST_ITEM_INDENT, name)
                };
                let line_text = truncate_for_list(outer.width, raw).into_owned();
                ListItem::new(vec![Line::from(line_text), Line::raw("")])
            })
            .collect();

        let list = selectable_list(items, &title);

        f.render_stateful_widget(list, content_area, &mut state);
    }

    // ヘルプ
    let help = Paragraph::new(" ↑↓: move | Esc: back").style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, help_area);
}

/// 追加フォーム画面を描画
///
/// # Arguments
///
/// * `f` - Ratatui frame to draw into.
/// * `form` - Current add-form sub-state.
/// * `filter_text` - Current filter input text.
/// * `filter_focused` - Whether the filter bar currently has focus.
fn view_add_form(f: &mut Frame, form: &AddFormModel, filter_text: &str, filter_focused: bool) {
    let outer = outer_rect(f.area());
    f.render_widget(Clear, outer);

    let [tabs_area, filter_area, content_area, help_area] = framed_layout(outer);

    // タブバー
    render_tab_bar(f, tabs_area);

    // フィルタバー（read-only）
    render_filter_bar(f, filter_area, filter_text, filter_focused);

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
            let content = Paragraph::new(lines).block(bordered_block(" Add Marketplace "));
            f.render_widget(content, content_area);
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
            let content = Paragraph::new(lines).block(bordered_block(" Add Marketplace "));
            f.render_widget(content, content_area);
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
            let content = Paragraph::new(lines).block(bordered_block(" Add Marketplace "));
            f.render_widget(content, content_area);
        }
    }

    // ヘルプ
    let help = Paragraph::new(" Type to input | Enter: next | Esc: cancel")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, help_area);
}

/// プラグインブラウズ画面を描画
///
/// # Arguments
///
/// * `f` - Ratatui frame to draw into.
/// * `marketplace_name` - Marketplace being browsed.
/// * `browse` - Browse data (plugin list, current selection, highlight).
/// * `state` - List state used for plugin highlight.
/// * `filter` - Filter input context for the filter bar.
fn view_plugin_browse(
    f: &mut Frame,
    marketplace_name: &str,
    browse: &BrowseData<'_>,
    mut state: ListState,
    filter: &FilterCtx<'_>,
) {
    let outer = outer_rect(f.area());
    f.render_widget(Clear, outer);

    let [tabs_area, filter_area, content_area, help_area] = framed_layout(outer);

    // タブバー
    render_tab_bar(f, tabs_area);

    // フィルタバー
    render_filter_bar(f, filter_area, filter.text, filter.focused);

    // コンテンツ領域
    let title = format!(" {} > Browse ({}) ", marketplace_name, browse.plugins.len());

    if browse.plugins.is_empty() {
        let msg = Paragraph::new("  No plugins available.")
            .block(bordered_block(&title))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(msg, content_area);
    } else if !should_split_layout(content_area.width) {
        // 狭い端末: リストのみ描画
        let items = build_browse_list_items(browse.plugins, browse.selected_plugins);
        let list = selectable_list(items, &title);
        f.render_stateful_widget(list, content_area, &mut state);
    } else {
        // 水平分割レイアウト（40% リスト / 60% 詳細）
        let [list_area, detail_area] = split_horizontal(content_area, (40, 60));

        // 左パネル: プラグインリスト
        let items = build_browse_list_items(browse.plugins, browse.selected_plugins);
        let list = selectable_list(items, &title);
        f.render_stateful_widget(list, list_area, &mut state);

        // 右パネル: プラグイン詳細（state.selected() から導出して一貫性を保つ）
        let detail_idx = state.selected().unwrap_or(browse.highlighted_idx);
        render_plugin_detail(f, browse.plugins, detail_idx, detail_area);
    }

    // ヘルプ
    let help = Paragraph::new(" Space: select | Enter/i: install | Esc: back | q: quit")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, help_area);
}

/// ターゲット選択画面を描画
///
/// # Arguments
///
/// * `f` - Ratatui frame to draw into.
/// * `targets` - Target list as `(name, display_name, selected)` tuples.
/// * `_highlighted_idx` - Currently highlighted index (unused; kept for symmetry).
/// * `state` - List state used for target highlight.
fn view_target_select(
    f: &mut Frame,
    targets: &[(String, String, bool)],
    _highlighted_idx: usize,
    mut state: ListState,
) {
    let outer = outer_rect(f.area());
    let modal_area = modal_layout(outer, 60, 60);
    f.render_widget(Clear, modal_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // コンテンツ
            Constraint::Length(1), // ヘルプ
        ])
        .split(modal_area);

    let items = build_target_list_items(targets);

    let list = selectable_list(items, " Select targets ");

    f.render_stateful_widget(list, chunks[0], &mut state);

    let help = Paragraph::new(" ↑↓: move  space: toggle  enter: ok  esc: back")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[1]);
}

/// スコープ選択画面を描画
///
/// # Arguments
///
/// * `f` - Ratatui frame to draw into.
/// * `highlighted_idx` - Currently highlighted scope index.
/// * `state` - List state used for scope highlight.
fn view_scope_select(f: &mut Frame, highlighted_idx: usize, mut state: ListState) {
    let outer = outer_rect(f.area());
    let modal_area = modal_layout(outer, 60, 50);
    f.render_widget(Clear, modal_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // コンテンツ
            Constraint::Length(1), // ヘルプ
        ])
        .split(modal_area);

    let items = build_scope_list_items(highlighted_idx);

    let list = selectable_list(items, " Select scope ");

    f.render_stateful_widget(list, chunks[0], &mut state);

    let help = Paragraph::new(" ↑↓: move  enter: select  esc: back")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[1]);
}

/// インストール実行中画面を描画
///
/// # Arguments
///
/// * `f` - Ratatui frame to draw into.
/// * `plugin_names` - Plugin names being installed in order.
/// * `current_idx` - Index of the plugin currently being processed.
/// * `total` - Total number of plugins in this install batch.
fn view_installing(f: &mut Frame, plugin_names: &[String], current_idx: usize, total: usize) {
    let outer = outer_rect(f.area());
    let modal_area = modal_layout(outer, 70, 40);
    f.render_widget(Clear, modal_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1)])
        .split(modal_area);

    let current_name = plugin_names
        .get(current_idx)
        .map(|s| s.as_str())
        .unwrap_or("...");

    let display_idx = (current_idx + 1).min(total);
    let ratio = if total > 0 {
        (display_idx as f64 / total as f64).min(1.0)
    } else {
        0.0
    };

    let lines = vec![
        Line::raw(""),
        Line::from(format!("  Installing {}...", current_name)),
        Line::raw(""),
    ];

    // Gauge doesn't support mixed content, so render text + gauge separately
    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(1)])
        .split(chunks[0]);

    let text = Paragraph::new(lines).block(
        Block::default()
            .title(Span::styled(
                " Installing ",
                Style::default().add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT),
    );
    f.render_widget(text, inner_chunks[0]);

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT))
        .gauge_style(Style::default().fg(Color::Green))
        .ratio(ratio)
        .label(format!("{}/{}", display_idx, total));
    f.render_widget(gauge, inner_chunks[1]);
}

/// インストール結果 1 件を描画用の `(行テキスト, 色)` に整形する。
///
/// 成功・失敗で接頭辞・色が異なるが、組み立て・切り詰め・スタイル付与の経路を
/// 共通化するため、ここでは色とテキストの導出だけを担う。
fn format_install_result_line(result: &PluginInstallResult) -> (String, Color) {
    if result.success {
        (format!("  ✓ {}", result.plugin_name), Color::Green)
    } else {
        let error_msg = result.error.as_deref().unwrap_or("Unknown error");
        (
            format!("  ✗ {}: {}", result.plugin_name, error_msg),
            Color::Red,
        )
    }
}

/// インストール結果画面を描画
///
/// # Arguments
///
/// * `f` - Ratatui frame to draw into.
/// * `summary` - Aggregated install summary to display.
fn view_install_result(f: &mut Frame, summary: &InstallSummary) {
    let outer = outer_rect(f.area());
    let modal_area = modal_layout(outer, 80, 70);
    f.render_widget(Clear, modal_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // コンテンツ
            Constraint::Length(1), // ヘルプ
        ])
        .split(modal_area);

    let mut lines = vec![
        Line::raw(""),
        Line::from(format!(
            "  Installed {}/{} plugins",
            summary.succeeded, summary.total
        )),
        Line::raw(""),
    ];

    for result in &summary.results {
        let (raw, color) = format_install_result_line(result);
        lines.push(Line::from(Span::styled(
            truncate_for_paragraph(outer.width, raw),
            Style::default().fg(color),
        )));
    }

    lines.push(Line::raw(""));

    let content = Paragraph::new(lines).block(bordered_block(" Install Result "));
    f.render_widget(content, chunks[0]);

    let help =
        Paragraph::new(" enter/esc: back to browse").style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[1]);
}

/// 右パネルにプラグイン詳細を描画
///
/// # Arguments
///
/// * `f` - Ratatui frame to draw into.
/// * `plugins` - Browse plugin list the detail panel reads from.
/// * `highlighted_idx` - Index of the plugin whose detail is shown.
/// * `area` - Target rectangle for the detail panel.
fn render_plugin_detail(
    f: &mut Frame,
    plugins: &[BrowsePlugin],
    highlighted_idx: usize,
    area: Rect,
) {
    let detail_block = bordered_block(" Detail ");

    if let Some(plugin) = plugins.get(highlighted_idx) {
        let source_text = match &plugin.source {
            PluginSource::Local(path) => format!("local ({})", path),
            PluginSource::External { source, repo } => format!("{} ({})", source, repo),
        };

        let installed_text = if plugin.installed { "Yes" } else { "No" };

        let lines = vec![
            Line::raw(""),
            Line::from(vec![
                Span::raw("  Name: "),
                Span::styled(&*plugin.name, Style::default().fg(Color::White)),
            ]),
            Line::raw(""),
            Line::from(vec![
                Span::raw("  Description: "),
                Span::styled(
                    plugin.description.as_deref().unwrap_or("N/A"),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::raw(""),
            Line::from(vec![
                Span::raw("  Version: "),
                Span::styled(
                    plugin.version.as_deref().unwrap_or("N/A"),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::raw(""),
            Line::from(vec![
                Span::raw("  Source: "),
                Span::styled(source_text, Style::default().fg(Color::White)),
            ]),
            Line::raw(""),
            Line::from(vec![
                Span::raw("  Installed: "),
                Span::styled(installed_text, Style::default().fg(Color::White)),
            ]),
        ];

        let detail = Paragraph::new(lines).block(detail_block);
        f.render_widget(detail, area);
    } else {
        f.render_widget(detail_block, area);
    }
}

/// 水平分割レイアウトを使用すべきかを判定
///
/// 端末幅が60未満の場合はリストのみ表示にフォールバックする。
///
/// # Arguments
///
/// * `width` - Current terminal width in cells.
fn should_split_layout(width: u16) -> bool {
    width >= 60
}

/// ターゲット選択のチェックボックスマークとスタイルを決定
///
/// # Arguments
///
/// * `selected` - Whether the target is currently selected.
fn target_checkbox(selected: bool) -> (&'static str, Style) {
    if selected {
        (CHECKBOX_SELECTED, Style::default().fg(Color::Yellow))
    } else {
        (CHECKBOX_UNSELECTED, Style::default())
    }
}

/// TargetSelect 用のリストアイテムを構築
///
/// # Arguments
///
/// * `targets` - Target list as `(name, display_name, selected)` tuples.
fn build_target_list_items(targets: &[(String, String, bool)]) -> Vec<ListItem<'static>> {
    targets
        .iter()
        .map(|(_name, display_name, selected)| {
            let (mark, style) = target_checkbox(*selected);
            let line_text = format!("{}{} {}", LIST_ITEM_INDENT, mark, display_name);
            ListItem::new(vec![Line::from(line_text), Line::raw("")]).style(style)
        })
        .collect()
}

/// スコープ選択のラジオボタンマークとスタイルを決定
///
/// # Arguments
///
/// * `is_current` - Whether this scope is the currently highlighted one.
fn scope_radio(is_current: bool) -> (&'static str, Style) {
    if is_current {
        (RADIO_SELECTED, Style::default().fg(Color::Yellow))
    } else {
        (RADIO_UNSELECTED, Style::default())
    }
}

/// ScopeSelect 用のリストアイテムを構築
///
/// # Arguments
///
/// * `highlighted_idx` - Currently highlighted scope index.
fn build_scope_list_items(highlighted_idx: usize) -> Vec<ListItem<'static>> {
    let scopes = [(Scope::Personal, "(~/.plm/)"), (Scope::Project, "(./)")];
    let clamped = highlighted_idx.min(scopes.len() - 1);
    scopes
        .iter()
        .enumerate()
        .map(|(idx, (scope, path))| {
            let is_current = idx == clamped;
            let (mark, style) = scope_radio(is_current);
            let line_text = format!(
                "{}{} {} {}",
                LIST_ITEM_INDENT,
                mark,
                scope.display_name(),
                path
            );
            ListItem::new(vec![Line::from(line_text), Line::raw("")]).style(style)
        })
        .collect()
}

/// `view_market_detail` の action メニュー項目を 2 行 ListItem として構築する。
///
/// 返される `ListItem` は所有データのみで構成されるため `'static`。
fn build_market_action_item(action: &DetailAction) -> ListItem<'static> {
    let line_text = format!("{}{}", LIST_ITEM_INDENT, action.label());
    ListItem::new(vec![Line::from(line_text), Line::raw("")]).style(action.style())
}

/// `view_market_detail` の action メニュー全体を組み立て、`(items, action_menu_rows)` を返す。
///
/// `action_menu_rows` は描画行数（`items.iter().map(ListItem::height).sum()`）。
fn build_market_action_menu(actions: &[DetailAction]) -> (Vec<ListItem<'static>>, u16) {
    let items: Vec<ListItem<'static>> = actions.iter().map(build_market_action_item).collect();
    let rows: u16 = items.iter().map(|i| i.height() as u16).sum();
    (items, rows)
}

/// Browse 行の状態ブロックとスタイルを決定（installed / selected / idle の 3 状態）。
///
/// 判定優先度: `installed` > `selected` > `idle`。
///
/// # Arguments
///
/// * `installed` - Whether the plugin is already installed.
/// * `selected` - Whether the plugin is currently selected for install.
fn browse_state_block(installed: bool, selected: bool) -> (&'static str, Style) {
    if installed {
        (MARK_MARKED, Style::default().fg(Color::DarkGray))
    } else if selected {
        (CHECKBOX_SELECTED, Style::default().fg(Color::Yellow))
    } else {
        (CHECKBOX_UNSELECTED, Style::default())
    }
}

/// PluginBrowse 用のリストアイテムを構築
///
/// # Arguments
///
/// * `plugins` - Browse plugin list to render.
/// * `selected_plugins` - Plugin names currently selected for install.
fn build_browse_list_items<'a>(
    plugins: &'a [BrowsePlugin],
    selected_plugins: &HashSet<String>,
) -> Vec<ListItem<'a>> {
    plugins
        .iter()
        .map(|p| {
            let selected = selected_plugins.contains(&p.name);
            let (mark, style) = browse_state_block(p.installed, selected);
            let body = match p.description.as_deref() {
                Some(desc) if !desc.is_empty() => format!("{} — {}", p.name, desc),
                _ => p.name.clone(),
            };
            let spans = vec![
                Span::styled(format!("{}{} ", LIST_ITEM_INDENT, mark), style),
                Span::styled(body, style),
            ];
            ListItem::new(vec![Line::from(spans), Line::raw("")])
        })
        .collect()
}

#[cfg(test)]
#[path = "view_test.rs"]
mod view_test;
