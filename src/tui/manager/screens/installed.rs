//! Installed タブの Model/Msg/update/view
//!
//! インストール済みプラグインの一覧表示と詳細確認。

pub mod actions;

use crate::tui::manager::core::{dialog_rect, ComponentKind, DataStore, PluginId, Tab};
use crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs};

// ============================================================================
// CacheState（タブ切替時の保持状態）
// ============================================================================

/// キャッシュ状態（タブ切替時に保持）
#[derive(Debug, Default)]
pub struct CacheState {
    pub selected_plugin_id: Option<PluginId>,
}

// ============================================================================
// DetailAction（プラグイン詳細画面のアクション）
// ============================================================================

/// プラグイン詳細画面のアクション
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DetailAction {
    DisablePlugin,
    MarkForUpdate,
    UpdateNow,
    Uninstall,
    ViewComponents,
    Back,
}

impl DetailAction {
    fn all() -> &'static [DetailAction] {
        &[
            DetailAction::DisablePlugin,
            DetailAction::MarkForUpdate,
            DetailAction::UpdateNow,
            DetailAction::Uninstall,
            DetailAction::ViewComponents,
            DetailAction::Back,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            DetailAction::DisablePlugin => "Disable plugin",
            DetailAction::MarkForUpdate => "Mark for update",
            DetailAction::UpdateNow => "Update now",
            DetailAction::Uninstall => "Uninstall",
            DetailAction::ViewComponents => "View components",
            DetailAction::Back => "Back to plugin list",
        }
    }

    fn style(&self) -> Style {
        match self {
            DetailAction::UpdateNow => Style::default().fg(Color::Green),
            DetailAction::Uninstall => Style::default().fg(Color::Red),
            _ => Style::default(),
        }
    }
}

// ============================================================================
// Model（画面状態）
// ============================================================================

/// Installed タブの画面状態
pub enum Model {
    /// プラグイン一覧画面
    PluginList {
        selected_id: Option<PluginId>,
        state: ListState,
    },
    /// プラグイン詳細画面
    PluginDetail {
        plugin_id: PluginId,
        state: ListState,
    },
    /// コンポーネント種別選択画面
    ComponentTypes {
        plugin_id: PluginId,
        selected_kind_idx: usize,
        state: ListState,
    },
    /// コンポーネント一覧画面
    ComponentList {
        plugin_id: PluginId,
        kind: ComponentKind,
        selected_idx: usize,
        state: ListState,
    },
}

impl Model {
    /// 新しいモデルを作成
    pub fn new(data: &DataStore) -> Self {
        let selected_id = data.plugins.first().map(|p| p.name.clone());
        let mut state = ListState::default();
        if selected_id.is_some() {
            state.select(Some(0));
        }
        Model::PluginList { selected_id, state }
    }

    /// キャッシュから復元
    pub fn from_cache(data: &DataStore, cache: &CacheState) -> Self {
        let selected_id = cache
            .selected_plugin_id
            .clone()
            .filter(|id| data.find_plugin(id).is_some())
            .or_else(|| data.plugins.first().map(|p| p.name.clone()));

        let index = selected_id
            .as_ref()
            .and_then(|id| data.plugin_index(id))
            .or(if data.plugins.is_empty() { None } else { Some(0) });

        let mut state = ListState::default();
        state.select(index);

        Model::PluginList { selected_id, state }
    }

    /// キャッシュ状態を取得
    pub fn to_cache(&self) -> CacheState {
        match self {
            Model::PluginList { selected_id, .. } => CacheState {
                selected_plugin_id: selected_id.clone(),
            },
            Model::PluginDetail { plugin_id, .. } => CacheState {
                selected_plugin_id: Some(plugin_id.clone()),
            },
            Model::ComponentTypes { plugin_id, .. } => CacheState {
                selected_plugin_id: Some(plugin_id.clone()),
            },
            Model::ComponentList { plugin_id, .. } => CacheState {
                selected_plugin_id: Some(plugin_id.clone()),
            },
        }
    }

    /// トップレベル（タブ切替可能）かどうか
    pub fn is_top_level(&self) -> bool {
        matches!(self, Model::PluginList { .. })
    }

    /// 現在選択中の ListState を取得
    fn current_state_mut(&mut self) -> &mut ListState {
        match self {
            Model::PluginList { state, .. } => state,
            Model::PluginDetail { state, .. } => state,
            Model::ComponentTypes { state, .. } => state,
            Model::ComponentList { state, .. } => state,
        }
    }
}

// ============================================================================
// Msg（メッセージ）
// ============================================================================

/// Installed タブへのメッセージ
pub enum Msg {
    Up,
    Down,
    Enter,
    Back,
}

/// キーコードをメッセージに変換
pub fn key_to_msg(key: KeyCode) -> Option<Msg> {
    match key {
        KeyCode::Up | KeyCode::Char('k') => Some(Msg::Up),
        KeyCode::Down | KeyCode::Char('j') => Some(Msg::Down),
        KeyCode::Enter => Some(Msg::Enter),
        KeyCode::Esc => Some(Msg::Back),
        _ => None,
    }
}

// ============================================================================
// update（状態更新）
// ============================================================================

/// メッセージに応じて状態を更新
pub fn update(model: &mut Model, msg: Msg, data: &mut DataStore) {
    match msg {
        Msg::Up => select_prev(model, data),
        Msg::Down => select_next(model, data),
        Msg::Enter => enter(model, data),
        Msg::Back => back(model),
    }
}

/// 選択を上に移動
fn select_prev(model: &mut Model, data: &DataStore) {
    let len = list_len(model, data);
    if len == 0 {
        return;
    }
    let state = model.current_state_mut();
    let current = state.selected().unwrap_or(0);
    let prev = current.saturating_sub(1);
    state.select(Some(prev));

    // selected_id を更新
    update_selected_id(model, data);
}

/// 選択を下に移動
fn select_next(model: &mut Model, data: &DataStore) {
    let len = list_len(model, data);
    if len == 0 {
        return;
    }
    let state = model.current_state_mut();
    let current = state.selected().unwrap_or(0);
    let next = (current + 1).min(len.saturating_sub(1));
    state.select(Some(next));

    // selected_id を更新
    update_selected_id(model, data);
}

/// 次の階層へ遷移
fn enter(model: &mut Model, data: &mut DataStore) {
    match model {
        Model::PluginList { selected_id, .. } => {
            // PluginList → PluginDetail へ遷移
            if let Some(plugin_id) = selected_id.clone() {
                if data.find_plugin(&plugin_id).is_some() {
                    let mut new_state = ListState::default();
                    new_state.select(Some(0));
                    *model = Model::PluginDetail {
                        plugin_id,
                        state: new_state,
                    };
                }
            }
        }
        Model::PluginDetail { plugin_id, state } => {
            // アクションに応じて遷移
            let detail_actions = DetailAction::all();
            let selected = state.selected().unwrap_or(0);

            // プラグイン情報を取得
            let plugin = data.find_plugin(plugin_id).cloned();
            let marketplace = plugin.as_ref().and_then(|p| p.marketplace.clone());

            match detail_actions.get(selected) {
                Some(DetailAction::DisablePlugin) => {
                    // Disable: デプロイ先から削除、キャッシュは残す
                    let result =
                        actions::disable_plugin(plugin_id, marketplace.as_deref());
                    match result {
                        actions::ActionResult::Success => {
                            // 成功 - ステータス表示を更新（TODO: 実装）
                        }
                        actions::ActionResult::Error(e) => {
                            data.last_error = Some(e);
                        }
                    }
                }
                Some(DetailAction::Uninstall) => {
                    // Uninstall: デプロイ先 + キャッシュ削除
                    let result =
                        actions::uninstall_plugin(plugin_id, marketplace.as_deref());
                    match result {
                        actions::ActionResult::Success => {
                            // 成功 - プラグインを一覧から削除して PluginList に戻る
                            data.remove_plugin(plugin_id);
                            let mut new_state = ListState::default();
                            if !data.plugins.is_empty() {
                                new_state.select(Some(0));
                            }
                            *model = Model::PluginList {
                                selected_id: data.plugins.first().map(|p| p.name.clone()),
                                state: new_state,
                            };
                        }
                        actions::ActionResult::Error(e) => {
                            data.last_error = Some(e);
                        }
                    }
                }
                Some(DetailAction::ViewComponents) => {
                    // ComponentTypes に遷移
                    let mut new_state = ListState::default();
                    new_state.select(Some(0));
                    *model = Model::ComponentTypes {
                        plugin_id: plugin_id.clone(),
                        selected_kind_idx: 0,
                        state: new_state,
                    };
                }
                Some(DetailAction::Back) => {
                    // PluginList に戻る
                    let id = plugin_id.clone();
                    let mut new_state = ListState::default();
                    new_state.select(Some(0));
                    *model = Model::PluginList {
                        selected_id: Some(id),
                        state: new_state,
                    };
                }
                _ => {
                    // 他のアクションは現時点では何もしない（UI表示のみ）
                }
            }
        }
        Model::ComponentTypes {
            plugin_id,
            state,
            ..
        } => {
            if let Some(plugin) = data.find_plugin(plugin_id) {
                let kinds = data.available_component_kinds(plugin);
                let selected_idx = state.selected().unwrap_or(0);
                if let Some(&kind) = kinds.get(selected_idx) {
                    let components = data.component_names(plugin, kind);
                    if !components.is_empty() {
                        let mut new_state = ListState::default();
                        new_state.select(Some(0));
                        *model = Model::ComponentList {
                            plugin_id: plugin_id.clone(),
                            kind,
                            selected_idx: 0,
                            state: new_state,
                        };
                    }
                }
            }
        }
        Model::ComponentList { .. } => {
            // 最下層なので何もしない
        }
    }
}

/// 前の階層へ戻る
fn back(model: &mut Model) {
    match model {
        Model::PluginList { .. } => {
            // PluginList での Back は app.rs で Quit 処理される
        }
        Model::PluginDetail { plugin_id, .. } => {
            // PluginDetail → PluginList へ戻る
            let id = plugin_id.clone();
            let mut new_state = ListState::default();
            new_state.select(Some(0));
            *model = Model::PluginList {
                selected_id: Some(id),
                state: new_state,
            };
        }
        Model::ComponentTypes { plugin_id, .. } => {
            // ComponentTypes → PluginDetail へ戻る
            let plugin_id = plugin_id.clone();
            let mut new_state = ListState::default();
            new_state.select(Some(0));
            *model = Model::PluginDetail {
                plugin_id,
                state: new_state,
            };
        }
        Model::ComponentList { plugin_id, kind: _, .. } => {
            let plugin_id = plugin_id.clone();
            let mut new_state = ListState::default();
            new_state.select(Some(0));
            *model = Model::ComponentTypes {
                plugin_id,
                selected_kind_idx: 0,
                state: new_state,
            };
        }
    }
}

/// 現在の画面のリスト長を取得
fn list_len(model: &Model, data: &DataStore) -> usize {
    match model {
        Model::PluginList { .. } => data.plugins.len(),
        Model::PluginDetail { .. } => DetailAction::all().len(),
        Model::ComponentTypes { plugin_id, .. } => {
            if let Some(plugin) = data.find_plugin(plugin_id) {
                data.available_component_kinds(plugin).len().max(1)
            } else {
                0
            }
        }
        Model::ComponentList {
            plugin_id, kind, ..
        } => {
            if let Some(plugin) = data.find_plugin(plugin_id) {
                data.component_names(plugin, *kind).len()
            } else {
                0
            }
        }
    }
}

/// selected_id を現在のインデックスから更新
fn update_selected_id(model: &mut Model, data: &DataStore) {
    if let Model::PluginList { selected_id, state } = model {
        if let Some(idx) = state.selected() {
            *selected_id = data.plugins.get(idx).map(|p| p.name.clone());
        }
    }
}

// ============================================================================
// view（描画）
// ============================================================================

/// 画面を描画
pub fn view(f: &mut Frame, model: &Model, data: &DataStore) {
    match model {
        Model::PluginList { state, .. } => {
            view_plugin_list(f, state.clone(), data);
        }
        Model::PluginDetail {
            plugin_id, state, ..
        } => {
            view_plugin_detail(f, plugin_id, state.clone(), data);
        }
        Model::ComponentTypes {
            plugin_id, state, ..
        } => {
            view_component_types(f, plugin_id, state.clone(), data);
        }
        Model::ComponentList {
            plugin_id,
            kind,
            state,
            ..
        } => {
            view_component_list(f, plugin_id, *kind, state.clone(), data);
        }
    }
}

/// プラグイン一覧画面を描画
fn view_plugin_list(f: &mut Frame, mut state: ListState, data: &DataStore) {
    let content_height = (data.plugins.len() as u16).max(1) + 6;
    let dialog_width = 55u16;
    let dialog_height = content_height.min(22);

    let dialog_area = dialog_rect(dialog_width, dialog_height, f.area());
    f.render_widget(Clear, dialog_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // タブバー
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

    // プラグインリスト
    let items: Vec<ListItem> = data
        .plugins
        .iter()
        .map(|p| {
            let marketplace_str = p
                .marketplace
                .as_ref()
                .map(|m| format!(" @{}", m))
                .unwrap_or_default();
            let text = format!("  {}{}  v{}", p.name, marketplace_str, p.version);
            ListItem::new(text)
        })
        .collect();

    let title = format!(" Installed Plugins ({}) ", data.plugins.len());
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
    let help = Paragraph::new(" Tab: switch | up/down: move | Enter: details | q: quit")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[2]);
}

/// プラグイン詳細画面を描画
fn view_plugin_detail(
    f: &mut Frame,
    plugin_id: &PluginId,
    mut state: ListState,
    data: &DataStore,
) {
    let Some(plugin) = data.find_plugin(plugin_id) else {
        return;
    };

    // ダイアログサイズ
    let dialog_width = 65u16;
    let dialog_height = 18u16;
    let dialog_area = dialog_rect(dialog_width, dialog_height, f.area());
    f.render_widget(Clear, dialog_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // タブバー
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

    // プラグイン情報
    let marketplace_str = plugin
        .marketplace
        .as_ref()
        .map(|m| format!(" @ {}", m))
        .unwrap_or_default();
    let title = format!(" {}{} ", plugin.name, marketplace_str);

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
            Span::styled("Enabled", Style::default().fg(Color::Green)),
        ]),
    ];

    let info_block = Block::default().title(title).borders(Borders::ALL);
    let info_para = Paragraph::new(info_lines).block(info_block);
    f.render_widget(info_para, chunks[1]);

    // アクションメニュー
    let actions = DetailAction::all();
    let items: Vec<ListItem> = actions
        .iter()
        .map(|a| ListItem::new(format!("  {}", a.label())).style(a.style()))
        .collect();

    let list = List::new(items)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_stateful_widget(list, chunks[2], &mut state);

    // ヘルプ
    let help = Paragraph::new(" Navigate: ↑↓ • Select: Enter • Back: Esc")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[3]);
}

/// コンポーネント種別選択画面を描画
fn view_component_types(
    f: &mut Frame,
    plugin_id: &PluginId,
    mut state: ListState,
    data: &DataStore,
) {
    let Some(plugin) = data.find_plugin(plugin_id) else {
        return;
    };

    let kinds = data.available_component_kinds(plugin);
    let has_marketplace = plugin.marketplace.is_some();
    let base_lines = if has_marketplace { 4 } else { 3 };
    let type_lines = if kinds.is_empty() { 1 } else { kinds.len() };
    let content_height = (base_lines + type_lines) as u16 + 4;
    let dialog_width = 55u16;
    let dialog_height = content_height.min(15);

    let dialog_area = dialog_rect(dialog_width, dialog_height, f.area());
    f.render_widget(Clear, dialog_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(dialog_area);

    let title = format!(" {} ", plugin.name);

    if kinds.is_empty() {
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
        f.render_widget(paragraph, chunks[0]);
    } else {
        // コンポーネントがある場合
        let items: Vec<ListItem> = kinds
            .iter()
            .map(|kind| {
                let count = data.component_names(plugin, *kind).len();
                let text = format!("  {} ({})", kind.title(), count);
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

        f.render_stateful_widget(list, chunks[0], &mut state);
    }

    // ヘルプ
    let help_text = if kinds.is_empty() {
        " Esc: back | q: quit"
    } else {
        " up/down: move | Enter: open | Esc: back | q: quit"
    };
    let help = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[1]);
}

/// コンポーネント一覧画面を描画
fn view_component_list(
    f: &mut Frame,
    plugin_id: &PluginId,
    kind: ComponentKind,
    mut state: ListState,
    data: &DataStore,
) {
    let Some(plugin) = data.find_plugin(plugin_id) else {
        return;
    };

    let components = data.component_names(plugin, kind);
    let items: Vec<ListItem> = components
        .iter()
        .map(|c| ListItem::new(format!("  {}", c)))
        .collect();

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
        kind.title(),
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

    f.render_stateful_widget(list, chunks[0], &mut state);

    // ヘルプ
    let help = Paragraph::new(" up/down: move | Esc: back | q: quit")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[1]);
}
