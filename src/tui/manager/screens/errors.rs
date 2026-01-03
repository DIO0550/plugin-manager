//! Errors タブの Model/Msg/update/view
//!
//! エラー表示と再試行。

use crate::tui::manager::core::{dialog_rect, DataStore, Tab};
use crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Tabs};

// ============================================================================
// Model（画面状態）
// ============================================================================

/// Errors タブの画面状態
pub struct Model {
    // エラー一覧は DataStore.last_error から取得
}

impl Model {
    /// 新しいモデルを作成
    pub fn new(_data: &DataStore) -> Self {
        Self {}
    }
}

// ============================================================================
// Msg（メッセージ）
// ============================================================================

/// Errors タブへのメッセージ
pub enum Msg {
    // 将来の拡張用（再試行など）
}

/// キーコードをメッセージに変換
pub fn key_to_msg(_key: KeyCode) -> Option<Msg> {
    None
}

// ============================================================================
// update（状態更新）
// ============================================================================

/// メッセージに応じて状態を更新
pub fn update(_model: &mut Model, _msg: Msg, _data: &DataStore) {
    // 将来の拡張用
}

// ============================================================================
// view（描画）
// ============================================================================

/// 画面を描画
pub fn view(f: &mut Frame, _model: &Model, data: &DataStore) {
    let dialog_width = 55u16;
    let dialog_height = 8u16;

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
        .select(Tab::Errors.index())
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .divider(" | ");
    f.render_widget(tabs, chunks[0]);

    // エラー表示またはプレースホルダー
    let message = if let Some(error) = &data.last_error {
        format!("\n  {}", error)
    } else {
        "\n  No errors".to_string()
    };

    let content = Paragraph::new(message)
        .block(Block::default().title(" Errors ").borders(Borders::ALL))
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(content, chunks[1]);

    // ヘルプ
    let help =
        Paragraph::new(" Tab: switch | q: quit").style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[2]);
}
