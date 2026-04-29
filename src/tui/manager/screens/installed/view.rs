//! Installed タブの view（描画）
//!
//! 各画面状態に応じた描画ロジック。

use super::model::{DetailAction, Model, UpdateStatusDisplay};
use crate::component::ComponentKind;
use crate::application::InstalledPlugin;
use crate::tui::manager::core::{
    content_rect, filter_plugins, render_filter_bar, truncate_to_width, DataStore, PluginId, Tab,
    HORIZONTAL_PADDING, LIST_DECORATION_WIDTH, MIN_CONTENT_WIDTH,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs};
use std::collections::{HashMap, HashSet};

/// 描画用共通コンテキスト
struct ViewCtx<'a> {
    data: &'a DataStore,
    filter_text: &'a str,
    filter_focused: bool,
}

/// ComponentKind の表示用タイトルを取得（複数形）
///
/// # Arguments
///
/// * `kind` - Component kind to format.
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
///
/// # Arguments
///
/// * `f` - Ratatui frame to draw into.
/// * `model` - Installed tab model to render.
/// * `data` - Shared data store for plugins.
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
    match model {
        Model::PluginList {
            state,
            marked_ids,
            update_statuses,
            ..
        } => {
            view_plugin_list(f, *state, &ctx, marked_ids, update_statuses);
        }
        Model::PluginDetail {
            plugin_id, state, ..
        } => {
            view_plugin_detail(f, plugin_id, *state, &ctx);
        }
        Model::ComponentTypes {
            plugin_id, state, ..
        } => {
            view_component_types(f, plugin_id, *state, &ctx);
        }
        Model::ComponentList {
            plugin_id,
            kind,
            state,
            ..
        } => {
            view_component_list(f, plugin_id, *kind, *state, &ctx);
        }
    }
}

/// reason テキストをリスト行表示用にサニタイズ（改行除去・長さ制限）
///
/// # Arguments
///
/// * `reason` - Raw reason text to normalize for list display.
fn sanitize_reason(reason: &str) -> String {
    let single_line: String = reason
        .chars()
        .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
        .collect();
    const MAX_LEN: usize = 40;
    let char_count = single_line.chars().count();
    if char_count > MAX_LEN {
        let truncated: String = single_line.chars().take(MAX_LEN).collect();
        format!("{truncated}...")
    } else {
        single_line
    }
}

/// プラグイン 1 行分の Span 列を構築する。
///
/// 通常時は `[prefix + name/version/disabled + update_status]` の複数 Span を返し、
/// `content_width < MIN_CONTENT_WIDTH` の狭幅時は更新ステータス Span を落とし、
/// `prefix + body` の最大 2 Span に切り詰める。`content_width` には `outer.width` を渡す。
pub(super) fn build_plugin_row_spans<'a>(
    plugin: &'a InstalledPlugin,
    is_marked: bool,
    update_status: Option<&'a UpdateStatusDisplay>,
    content_width: u16,
) -> Vec<Span<'a>> {
    let mark_indicator = if is_marked { "[x] " } else { "[ ] " };
    let prefix = format!("  {}", mark_indicator);

    let marketplace_str = plugin
        .marketplace()
        .map(|m| format!(" @{}", m))
        .unwrap_or_default();
    let status_str = if plugin.enabled() { "" } else { " [disabled]" };
    let name_text = format!(
        "{}{}  v{}{}",
        plugin.name(),
        marketplace_str,
        plugin.version(),
        status_str
    );

    let base_style = if is_marked {
        Style::default().fg(Color::Yellow)
    } else if plugin.enabled() {
        Style::default()
    } else {
        Style::default().fg(Color::DarkGray)
    };

    if content_width < MIN_CONTENT_WIDTH {
        let list_inner_width = content_width.saturating_sub(LIST_DECORATION_WIDTH);
        let prefix_w = prefix.chars().count() as u16;
        let body_budget = list_inner_width.saturating_sub(prefix_w);
        let body = truncate_to_width(&name_text, body_budget);
        vec![
            Span::styled(prefix, base_style),
            Span::styled(body, base_style),
        ]
    } else {
        let mut spans = vec![
            Span::styled(prefix, base_style),
            Span::styled(name_text, base_style),
        ];
        if let Some(status) = update_status {
            spans.push(update_status_span(status));
        }
        spans
    }
}

/// プラグイン 1 行分の `ListItem` を構築する。
pub(super) fn build_plugin_row<'a>(
    plugin: &'a InstalledPlugin,
    is_marked: bool,
    update_status: Option<&'a UpdateStatusDisplay>,
    content_width: u16,
) -> ListItem<'a> {
    let spans = build_plugin_row_spans(plugin, is_marked, update_status, content_width);
    ListItem::new(Line::from(spans))
}

/// 更新ステータスの表示文字列とスタイルを取得
///
/// # Arguments
///
/// * `status` - Update status to format as a `Span`.
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
            let text = format!(" Skipped: {}", sanitize_reason(reason));
            Span::styled(text, Style::default().fg(Color::DarkGray))
        }
        UpdateStatusDisplay::Failed(reason) => {
            let text = format!(" Failed: {}", sanitize_reason(reason));
            Span::styled(text, Style::default().fg(Color::Red))
        }
    }
}

/// プラグイン一覧画面を描画
///
/// # Arguments
///
/// * `f` - Ratatui frame to draw into.
/// * `state` - List state used for highlight/selection.
/// * `ctx` - Shared view context (data store + filter state).
/// * `marked_ids` - Plugin ids currently marked for batch actions.
/// * `update_statuses` - Per-plugin update status for indicator display.
fn view_plugin_list(
    f: &mut Frame,
    mut state: ListState,
    ctx: &ViewCtx<'_>,
    marked_ids: &HashSet<PluginId>,
    update_statuses: &HashMap<PluginId, UpdateStatusDisplay>,
) {
    let filtered = filter_plugins(&ctx.data.plugins, ctx.filter_text);
    let outer = content_rect(f.area(), HORIZONTAL_PADDING);
    f.render_widget(Clear, outer);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // タブバー
            Constraint::Length(3), // フィルタバー
            Constraint::Min(1),    // コンテンツ
            Constraint::Length(1), // ヘルプ
        ])
        .split(outer);

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
    render_filter_bar(f, chunks[1], ctx.filter_text, ctx.filter_focused);

    // タイトル（マーク数表示付き）
    let marked_count = marked_ids.len();
    let title = if marked_count > 0 {
        format!(
            " Installed Plugins ({}/{}) [{} marked] ",
            filtered.len(),
            ctx.data.plugins.len(),
            marked_count
        )
    } else {
        format!(
            " Installed Plugins ({}/{}) ",
            filtered.len(),
            ctx.data.plugins.len()
        )
    };

    // プラグインリスト（フィルタ済み）
    if filtered.is_empty() {
        let no_match = Paragraph::new("  No matching plugins")
            .block(Block::default().title(title).borders(Borders::ALL))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(no_match, chunks[2]);
    } else {
        let items: Vec<ListItem> = filtered
            .iter()
            .map(|p| {
                let is_marked = marked_ids.contains(p.id());
                let update_status = update_statuses.get(p.id());
                build_plugin_row(p, is_marked, update_status, outer.width)
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
        " Space: mark | a: all | U: update | A: update all | Tab: switch | ↑↓: move | Enter: details | q: quit",
    )
    .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[3]);
}

/// プラグイン詳細画面を描画
///
/// # Arguments
///
/// * `f` - Ratatui frame to draw into.
/// * `plugin_id` - Plugin id whose detail should be shown.
/// * `state` - List state used for action menu highlight.
/// * `ctx` - Shared view context (data store + filter state).
fn view_plugin_detail(
    f: &mut Frame,
    plugin_id: &PluginId,
    mut state: ListState,
    ctx: &ViewCtx<'_>,
) {
    let Some(plugin) = ctx.data.find_plugin(plugin_id) else {
        return;
    };

    let outer = content_rect(f.area(), HORIZONTAL_PADDING);
    f.render_widget(Clear, outer);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // タブバー
            Constraint::Length(3), // フィルタバー
            Constraint::Length(7), // プラグイン情報
            Constraint::Min(1),    // アクションメニュー
            Constraint::Length(1), // ヘルプ
        ])
        .split(outer);

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
    render_filter_bar(f, chunks[1], ctx.filter_text, ctx.filter_focused);

    // プラグイン情報
    let marketplace_str = plugin
        .marketplace()
        .map(|m| format!(" @ {}", m))
        .unwrap_or_default();
    let title = format!(" {}{} ", plugin.name(), marketplace_str);

    let (status_text, status_color) = if plugin.enabled() {
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
            Span::styled(plugin.version(), Style::default().fg(Color::White)),
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
    let actions = DetailAction::for_plugin(plugin.enabled());
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
///
/// # Arguments
///
/// * `f` - Ratatui frame to draw into.
/// * `plugin_id` - Plugin id whose component kinds should be listed.
/// * `state` - List state used for kind selection.
/// * `ctx` - Shared view context (data store + filter state).
fn view_component_types(
    f: &mut Frame,
    plugin_id: &PluginId,
    mut state: ListState,
    ctx: &ViewCtx<'_>,
) {
    let Some(plugin) = ctx.data.find_plugin(plugin_id) else {
        return;
    };

    let counts = ctx.data.available_component_kinds(plugin);

    let outer = content_rect(f.area(), HORIZONTAL_PADDING);
    f.render_widget(Clear, outer);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // フィルタバー
            Constraint::Min(1),    // コンテンツ
            Constraint::Length(1), // ヘルプ
        ])
        .split(outer);

    // フィルタバー（read-only）
    render_filter_bar(f, chunks[0], ctx.filter_text, ctx.filter_focused);

    let title = format!(" {} ", plugin.name());

    if counts.is_empty() {
        // コンポーネントがない場合
        let mut lines = Vec::new();
        lines.push(format!("  Version: {}", plugin.version()));
        if let Some(marketplace) = plugin.marketplace() {
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
            .map(|(kind, count)| {
                let text = format!("  {} ({})", component_kind_title(*kind), count);
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
///
/// # Arguments
///
/// * `f` - Ratatui frame to draw into.
/// * `plugin_id` - Plugin id the components belong to.
/// * `kind` - Component kind to list.
/// * `state` - List state used for component selection.
/// * `ctx` - Shared view context (data store + filter state).
fn view_component_list(
    f: &mut Frame,
    plugin_id: &PluginId,
    kind: ComponentKind,
    mut state: ListState,
    ctx: &ViewCtx<'_>,
) {
    let Some(plugin) = ctx.data.find_plugin(plugin_id) else {
        return;
    };

    let components = ctx.data.component_names(plugin, kind);
    let items: Vec<ListItem> = components
        .iter()
        .map(|c| ListItem::new(format!("  {}", c)))
        .collect();

    let outer = content_rect(f.area(), HORIZONTAL_PADDING);
    f.render_widget(Clear, outer);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // フィルタバー
            Constraint::Min(1),    // コンテンツ
            Constraint::Length(1), // ヘルプ
        ])
        .split(outer);

    // フィルタバー（read-only）
    render_filter_bar(f, chunks[0], ctx.filter_text, ctx.filter_focused);

    let title = format!(
        " {} > {} ({}) ",
        plugin.name(),
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

#[cfg(test)]
#[path = "view_test.rs"]
mod view_test;
