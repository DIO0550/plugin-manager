//! Discover タブの Model/Msg/update/view
//!
//! 利用可能なプラグインの検索と閲覧。

use crate::tui::manager::core::{
    content_rect, render_filter_bar, DataStore, PluginId, Tab, HORIZONTAL_PADDING,
};
use crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, ListState, Paragraph, Tabs};

/// キャッシュ状態（タブ切替時に保持）
#[derive(Debug, Default)]
pub struct CacheState {
    pub selected_id: Option<PluginId>,
}

/// Discover タブの画面状態
pub struct Model {
    pub selected_id: Option<PluginId>,
    pub state: ListState,
}

impl Model {
    /// 新しいモデルを作成
    ///
    /// # Arguments
    ///
    /// * `_data` - Shared data store (currently unused on this tab).
    pub fn new(_data: &DataStore) -> Self {
        Self {
            selected_id: None,
            state: ListState::default(),
        }
    }

    /// キャッシュから復元
    ///
    /// # Arguments
    ///
    /// * `_data` - Shared data store (currently unused on this tab).
    /// * `cache` - Cached state carried across tab switches.
    pub fn from_cache(_data: &DataStore, cache: &CacheState) -> Self {
        Self {
            selected_id: cache.selected_id.clone(),
            state: ListState::default(),
        }
    }

    /// キャッシュ状態を取得
    pub fn to_cache(&self) -> CacheState {
        CacheState {
            selected_id: self.selected_id.clone(),
        }
    }
}

/// Discover タブへのメッセージ
pub enum Msg {
    // 将来の拡張用
}

/// キーコードをメッセージに変換
///
/// # Arguments
///
/// * `_key` - Pressed key code.
pub fn key_to_msg(_key: KeyCode) -> Option<Msg> {
    None
}

/// メッセージに応じて状態を更新
///
/// # Arguments
///
/// * `_model` - Mutable screen state.
/// * `_msg` - Incoming message.
/// * `_data` - Shared data store.
pub fn update(_model: &mut Model, _msg: Msg, _data: &DataStore) {
    // 将来の拡張用
}

/// 画面を描画
///
/// # Arguments
///
/// * `f` - Ratatui frame to render into.
/// * `_model` - Current screen state.
/// * `_data` - Shared data store.
/// * `filter_text` - Current filter input text.
/// * `filter_focused` - Whether the filter bar has focus.
pub fn view(
    f: &mut Frame,
    _model: &Model,
    _data: &DataStore,
    filter_text: &str,
    filter_focused: bool,
) {
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

    let tab_titles: Vec<&str> = Tab::all().iter().map(|t| t.title()).collect();
    let tabs = Tabs::new(tab_titles)
        .select(Tab::Discover.index())
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .divider(" | ");
    f.render_widget(tabs, chunks[0]);

    // フィルタバー（Discover タブではフィルタ機能は未対応、UI のみ表示）
    render_filter_bar(f, chunks[1], filter_text, filter_focused);

    let content = Paragraph::new("\n  Browse available plugins")
        .block(Block::default().title(" Discover ").borders(Borders::ALL))
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(content, chunks[2]);

    let help = Paragraph::new(" Tab: switch | q: quit").style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[3]);
}
