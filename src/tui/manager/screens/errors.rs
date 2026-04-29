//! Errors タブの Model/Msg/update/view
//!
//! エラー表示と再試行。

use crate::tui::manager::core::{
    content_rect, render_filter_bar, DataStore, Tab, HORIZONTAL_PADDING,
};
use crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Tabs};

/// Errors タブの画面状態
pub struct Model {
    // エラー一覧は DataStore.last_error から取得
}

impl Model {
    /// 新しいモデルを作成
    ///
    /// # Arguments
    ///
    /// * `_data` - Shared data store (currently unused on this tab).
    pub fn new(_data: &DataStore) -> Self {
        Self {}
    }
}

/// Errors タブへのメッセージ
pub enum Msg {
    // 将来の拡張用（再試行など）
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
/// * `data` - Shared data store providing the last error.
/// * `filter_text` - Current filter input text.
/// * `filter_focused` - Whether the filter bar has focus.
pub fn view(
    f: &mut Frame,
    _model: &Model,
    data: &DataStore,
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
        .select(Tab::Errors.index())
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .divider(" | ");
    f.render_widget(tabs, chunks[0]);

    // フィルタバー（Errors タブではフィルタ機能は未対応、UI のみ表示）
    render_filter_bar(f, chunks[1], filter_text, filter_focused);

    let message = if let Some(error) = &data.last_error {
        format!("\n  {}", error)
    } else {
        "\n  No errors".to_string()
    };

    let content = Paragraph::new(message)
        .block(Block::default().title(" Errors ").borders(Borders::ALL))
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(content, chunks[2]);

    let help = Paragraph::new(" Tab: switch | q: quit").style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[3]);
}
