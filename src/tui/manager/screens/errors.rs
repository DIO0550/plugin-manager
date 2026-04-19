//! Errors タブの Model/Msg/update/view
//!
//! エラー表示と再試行。

use crate::tui::manager::core::{dialog_rect, render_filter_bar, DataStore, Tab};
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
    /// * `_data` - shared data store (currently unused on this tab)
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
/// * `_key` - pressed key code
pub fn key_to_msg(_key: KeyCode) -> Option<Msg> {
    None
}

/// メッセージに応じて状態を更新
///
/// # Arguments
///
/// * `_model` - mutable screen state
/// * `_msg` - incoming message
/// * `_data` - shared data store
pub fn update(_model: &mut Model, _msg: Msg, _data: &DataStore) {
    // 将来の拡張用
}

/// 画面を描画
///
/// # Arguments
///
/// * `f` - ratatui frame to render into
/// * `_model` - current screen state
/// * `data` - shared data store providing the last error
/// * `filter_text` - current filter input text
/// * `filter_focused` - whether the filter bar has focus
pub fn view(
    f: &mut Frame,
    _model: &Model,
    data: &DataStore,
    filter_text: &str,
    filter_focused: bool,
) {
    let dialog_width = 55u16;
    let dialog_height = 11u16; // +3 for filter bar

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
