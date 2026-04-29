//! Installed タブの view（描画）
//!
//! 各画面状態に応じた描画ロジック。

use super::model::{DetailAction, Model, UpdateStatusDisplay};
use crate::application::InstalledPlugin;
use crate::component::ComponentKind;
use crate::tui::manager::core::layout::{detail_layout, framed_layout, outer_rect};
use crate::tui::manager::core::style::{
    bordered_block, menu_list, selectable_list, ICON_DISABLED, ICON_ENABLED, LIST_ITEM_INDENT,
    MARK_MARKED, MARK_UNMARKED,
};
use crate::tui::manager::core::{
    filter_plugins, render_filter_bar, truncate_to_width, DataStore, PluginId, Tab,
    LIST_DECORATION_WIDTH, MIN_CONTENT_WIDTH,
};
use ratatui::prelude::*;
use ratatui::widgets::{Clear, ListItem, ListState, Paragraph, Tabs};
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

/// マーク状態 / 有効状態に応じた行のベーススタイルを返す。
fn plugin_row_style(is_marked: bool, enabled: bool) -> Style {
    match (is_marked, enabled) {
        (true, _) => Style::default().fg(Color::Yellow),
        (false, true) => Style::default(),
        (false, false) => Style::default().fg(Color::DarkGray),
    }
}

/// マーク状態 + enabled 状態から行の prefix を組み立てる。
///
/// 完成形例: `"  [✓] ● "` (marked + enabled) / `"  [ ] ○ "` (unmarked + disabled)
fn plugin_row_prefix(is_marked: bool, enabled: bool) -> String {
    let mark = if is_marked {
        MARK_MARKED
    } else {
        MARK_UNMARKED
    };
    let icon = if enabled { ICON_ENABLED } else { ICON_DISABLED };
    format!("{}{} {} ", LIST_ITEM_INDENT, mark, icon)
}

/// 名前 + (marketplace) + バージョン + (disabled) を組み立てる。
fn plugin_row_name_text(plugin: &InstalledPlugin) -> String {
    let marketplace_str = plugin
        .marketplace()
        .map(|m| format!(" @{}", m))
        .unwrap_or_default();
    let status_str = if plugin.enabled() { "" } else { " [disabled]" };
    format!(
        "{}{}  v{}{}",
        plugin.name(),
        marketplace_str,
        plugin.version(),
        status_str
    )
}

/// 狭幅 (`content_width < MIN_CONTENT_WIDTH`) フォールバック用の Span 列。
///
/// 更新ステータス Span を落とし、`prefix` のスタイルを保ったまま
/// 残り幅 (`list_inner_width - prefix_w`) で `name_text` を切り詰めた最大 2 Span 構成。
fn narrow_plugin_row_spans<'a>(
    prefix: String,
    name_text: &str,
    content_width: u16,
    base_style: Style,
) -> Vec<Span<'a>> {
    let list_inner_width = content_width.saturating_sub(LIST_DECORATION_WIDTH);
    let prefix_w = prefix.chars().count() as u16;
    let body_budget = list_inner_width.saturating_sub(prefix_w);
    let body = truncate_to_width(name_text, body_budget);
    vec![
        Span::styled(prefix, base_style),
        Span::styled(body, base_style),
    ]
}

/// 通常幅用の Span 列。`prefix + name_text` に更新ステータス Span を追記する。
fn wide_plugin_row_spans<'a>(
    prefix: String,
    name_text: String,
    update_status: Option<&'a UpdateStatusDisplay>,
    base_style: Style,
) -> Vec<Span<'a>> {
    let mut spans = vec![
        Span::styled(prefix, base_style),
        Span::styled(name_text, base_style),
    ];
    if let Some(status) = update_status {
        spans.push(update_status_span(status));
    }
    spans
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
    let prefix = plugin_row_prefix(is_marked, plugin.enabled());
    let name_text = plugin_row_name_text(plugin);
    let base_style = plugin_row_style(is_marked, plugin.enabled());

    if content_width < MIN_CONTENT_WIDTH {
        narrow_plugin_row_spans(prefix, &name_text, content_width, base_style)
    } else {
        wide_plugin_row_spans(prefix, name_text, update_status, base_style)
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
    ListItem::new(vec![Line::raw(""), Line::from(spans), Line::raw("")])
}

/// 詳細画面の action メニュー項目を 3 行 ListItem (上下空行 + 内容) として構築する。
///
/// 内容行を上下空行で挟むことで、強調表示時に文字が縦中央に来る。
/// 返される `ListItem` は所有データのみで構成されるため `'static`。
fn build_detail_action_item(action: &DetailAction) -> ListItem<'static> {
    let line_text = format!("{}{}", LIST_ITEM_INDENT, action.label());
    ListItem::new(vec![Line::raw(""), Line::from(line_text), Line::raw("")]).style(action.style())
}

/// 詳細画面の action メニュー全体を組み立て、`(items, action_menu_rows)` を返す。
///
/// `action_menu_rows` は描画行数（`items.iter().map(ListItem::height).sum()`）。
/// 呼び出し側は `detail_layout` にこの値を渡して全 action の領域を確保する。
fn build_detail_action_menu(actions: &[DetailAction]) -> (Vec<ListItem<'static>>, u16) {
    let items: Vec<ListItem<'static>> = actions.iter().map(build_detail_action_item).collect();
    let rows: u16 = items.iter().map(|i| i.height() as u16).sum();
    (items, rows)
}

/// `view_component_types` のリスト項目を 3 行 ListItem (上下空行 + 内容) として構築する。
fn build_component_types_item<'a>(kind: ComponentKind, count: usize) -> ListItem<'a> {
    let line_text = format!(
        "{}{} ({})",
        LIST_ITEM_INDENT,
        component_kind_title(kind),
        count
    );
    ListItem::new(vec![Line::raw(""), Line::from(line_text), Line::raw("")])
}

/// `view_component_list` のリスト項目を 3 行 ListItem (上下空行 + 内容) として構築する。
fn build_component_list_item<'a>(component_name: &str) -> ListItem<'a> {
    let line_text = format!("{}{}", LIST_ITEM_INDENT, component_name);
    ListItem::new(vec![Line::raw(""), Line::from(line_text), Line::raw("")])
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
    let outer = outer_rect(f.area());
    f.render_widget(Clear, outer);

    let [tabs_area, filter_area, content_area, help_area] = framed_layout(outer);

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
    f.render_widget(tabs, tabs_area);

    // フィルタバー
    render_filter_bar(f, filter_area, ctx.filter_text, ctx.filter_focused);

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
            .block(bordered_block(&title))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(no_match, content_area);
    } else {
        let items: Vec<ListItem> = filtered
            .iter()
            .map(|p| {
                let is_marked = marked_ids.contains(p.id());
                let update_status = update_statuses.get(p.id());
                build_plugin_row(p, is_marked, update_status, outer.width)
            })
            .collect();

        let list = selectable_list(items, &title);

        f.render_stateful_widget(list, content_area, &mut state);
    }

    // ヘルプ
    let help = Paragraph::new(
        " Space: mark | a: all | U: update | A: update all | Tab: switch | ↑↓: move | Enter: details | q: quit",
    )
    .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, help_area);
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

    let outer = outer_rect(f.area());
    f.render_widget(Clear, outer);

    let [tabs_area, filter_area, content_area, help_area] = framed_layout(outer);

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
    f.render_widget(tabs, tabs_area);

    // フィルタバー（read-only）
    render_filter_bar(f, filter_area, ctx.filter_text, ctx.filter_focused);

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
            Span::raw("    Version: "),
            Span::styled(plugin.version(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::raw("Author: "),
            Span::styled("N/A", Style::default().fg(Color::DarkGray)),
            Span::raw("    Status: "),
            Span::styled(status_text, Style::default().fg(status_color)),
        ]),
    ];

    // アクションメニュー（enabled 状態に応じて動的に切り替え）
    let actions = DetailAction::for_plugin(plugin.enabled());
    let (items, action_menu_rows) = build_detail_action_menu(&actions);

    let [info_area, menu_area, _detail_help] = detail_layout(content_area, action_menu_rows);

    let info_para = Paragraph::new(info_lines).block(bordered_block(&title));
    f.render_widget(info_para, info_area);

    let list = menu_list(items);
    f.render_stateful_widget(list, menu_area, &mut state);

    // ヘルプ
    let help = Paragraph::new(" Navigate: ↑↓ • Select: Enter • Back: Esc")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, help_area);
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

    let outer = outer_rect(f.area());
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

        let paragraph = Paragraph::new(lines.join("\n")).block(bordered_block(&title));
        f.render_widget(paragraph, chunks[1]);
    } else {
        // コンポーネントがある場合
        let items: Vec<ListItem> = counts
            .iter()
            .map(|(kind, count)| build_component_types_item(*kind, *count))
            .collect();

        let list = selectable_list(items, &title);

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
        .map(|c| build_component_list_item(c))
        .collect();

    let outer = outer_rect(f.area());
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
    let list = selectable_list(items, &title);

    f.render_stateful_widget(list, chunks[1], &mut state);

    // ヘルプ
    let help = Paragraph::new(" ↑↓: move | Esc: back | q: quit")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[2]);
}

#[cfg(test)]
#[path = "view_test.rs"]
mod view_test;
