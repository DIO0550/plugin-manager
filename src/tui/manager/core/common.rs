//! 共通 UI ユーティリティ
//!
//! 複数タブで共有される描画ユーティリティ。

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

/// フィルタ入力欄を描画（ボーダー付き、高さ3行）
pub fn render_filter_bar(f: &mut Frame, area: Rect, filter_text: &str, focused: bool) {
    let (border_color, text_content) = if focused {
        let cursor = "\u{2502}"; // │ カーソル
        let text = if filter_text.is_empty() {
            format!(" \u{1f50e} {}", cursor)
        } else {
            format!(" \u{1f50e} {}{}", filter_text, cursor)
        };
        (
            Color::White,
            Paragraph::new(text).style(Style::default().fg(Color::White)),
        )
    } else if filter_text.is_empty() {
        (
            Color::DarkGray,
            Paragraph::new(" \u{1f50e} Search...").style(Style::default().fg(Color::DarkGray)),
        )
    } else {
        (
            Color::DarkGray,
            Paragraph::new(format!(" \u{1f50e} {}", filter_text))
                .style(Style::default().fg(Color::White)),
        )
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let content = text_content.block(block);
    f.render_widget(content, area);
}

/// コンテンツに合わせたダイアログ領域を計算（左寄せ）
pub fn dialog_rect(width: u16, height: u16, area: Rect) -> Rect {
    Rect::new(
        area.x,
        area.y,
        width.min(area.width),
        height.min(area.height),
    )
}
