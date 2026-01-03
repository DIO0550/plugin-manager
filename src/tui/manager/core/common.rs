//! 共通 UI ユーティリティ
//!
//! 複数タブで共有される描画ユーティリティ。

use ratatui::prelude::Rect;

/// コンテンツに合わせたダイアログ領域を計算（左寄せ）
pub fn dialog_rect(width: u16, height: u16, area: Rect) -> Rect {
    Rect::new(area.x, area.y, width.min(area.width), height.min(area.height))
}
