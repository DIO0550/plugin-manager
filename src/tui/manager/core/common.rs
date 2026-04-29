//! 共通 UI ユーティリティ
//!
//! 複数タブで共有される描画ユーティリティ。

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

/// フィルタ入力欄を描画（ボーダー付き、高さ3行）
///
/// # Arguments
///
/// * `f` - the `ratatui` frame to draw into
/// * `area` - the rectangle to render the filter bar in
/// * `filter_text` - the current filter query text
/// * `focused` - `true` when the filter input currently has focus
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

/// フレーム全体の左右パディング（cells）。タブバー・フィルタ・コンテンツ・ヘルプ全てに適用。
pub const HORIZONTAL_PADDING: u16 = 2;

/// 通常レイアウトとして扱う最小コンテンツ幅。これを下回ると各行を `truncate_to_width` で切り詰める。
///
/// これは閾値判定にのみ使い、切り詰め予算にはウィジェット種類ごとの装飾幅
/// (`LIST_DECORATION_WIDTH` / `BLOCK_BORDER_WIDTH`) を差し引いた値を使う。
pub const MIN_CONTENT_WIDTH: u16 = 40;

/// `Block(Borders::ALL)` の左右ボーダー分の幅 cells（左右で合計 2 cells）。
///
/// `Paragraph + Block(Borders::ALL)`（`highlight_symbol` を持たないウィジェット）の
/// inner width 算出に使う: `inner = outer.width.saturating_sub(BLOCK_BORDER_WIDTH)`。
pub const BLOCK_BORDER_WIDTH: u16 = 2;

/// `List` ウィジェットの選択記号 `highlight_symbol("> ")` 分の幅 cells（"> " で 2 cells）。
pub const LIST_HIGHLIGHT_WIDTH: u16 = 2;

/// `List` ウィジェットの装飾分（描画不可領域）の合計幅 cells。
///
/// 内訳: ボーダー左右 2 cells (`BLOCK_BORDER_WIDTH`) + `highlight_symbol("> ")` 2 cells
/// (`LIST_HIGHLIGHT_WIDTH`) = 4 cells。
/// `outer.width` から `LIST_DECORATION_WIDTH` を差し引いた値が、`List` の行内に
/// 実際に描画可能な幅。
///
/// 閾値判定 (`MIN_CONTENT_WIDTH`) と切り詰め予算は別物。
/// **Paragraph 系には `BLOCK_BORDER_WIDTH`、List 系には `LIST_DECORATION_WIDTH` を使い分ける**。
pub const LIST_DECORATION_WIDTH: u16 = BLOCK_BORDER_WIDTH + LIST_HIGHLIGHT_WIDTH;

/// フレーム領域から左右パディングのみを差し引いたコンテンツ領域を返す純粋関数。
///
/// 高さ・y 座標は変更しない（モーダルの縦サイズを保つため、上下パディングは加えない）。
///
/// # Arguments
///
/// * `area` - 元の Frame 領域（通常 `f.area()`）
/// * `padding` - 左右に差し引くパディング cells（左右共通、合計 `2 * padding`）
pub fn content_rect(area: Rect, padding: u16) -> Rect {
    let total_pad = padding.saturating_mul(2);
    let width = area.width.saturating_sub(total_pad);
    Rect::new(area.x.saturating_add(padding), area.y, width, area.height)
}

/// 切り詰め時の省略記号文字列。
pub const ELLIPSIS: &str = "...";

/// 切り詰め時の省略記号の文字数（cells）。`ELLIPSIS.chars().count()` と整合させる。
pub const ELLIPSIS_LEN: u16 = 3;

/// 文字列を最大幅 `max_width` cells に収まるように切り詰める純粋関数。
///
/// `text.chars().count() <= max_width` ならそのまま返す。
/// 超える場合、`max_width > ELLIPSIS_LEN` なら先頭 `max_width - ELLIPSIS_LEN` 文字 + `ELLIPSIS`、
/// `max_width <= ELLIPSIS_LEN` なら先頭 `max_width` 文字を返す（省略記号を載せる余地が無い）。
///
/// 想定用途は ASCII / 半角英数字主体のリスト行・タイトル・説明文。
/// 全角文字を含む場合は表示幅と文字数が一致しないため目安として動作する。
pub fn truncate_to_width(text: &str, max_width: u16) -> String {
    let max = max_width as usize;
    if text.chars().count() <= max {
        return text.to_string();
    }
    let ellipsis_len = ELLIPSIS_LEN as usize;
    if max <= ellipsis_len {
        return text.chars().take(max).collect();
    }
    let head: String = text.chars().take(max - ellipsis_len).collect();
    format!("{head}{ELLIPSIS}")
}

/// `content_width` が `MIN_CONTENT_WIDTH` 未満のときだけ `text` を装飾幅 `decoration_width` 引きで切り詰める。
///
/// それ以外（通常幅）は `text` を所有形式に変換するだけで何も変えない。
/// ウィジェット種類で異なる装飾幅を呼び側が選び、共通の if 分岐をここに集約する。
fn truncate_when_narrow(content_width: u16, decoration_width: u16, text: &str) -> String {
    if content_width < MIN_CONTENT_WIDTH {
        let inner_width = content_width.saturating_sub(decoration_width);
        truncate_to_width(text, inner_width)
    } else {
        text.to_string()
    }
}

/// `List`（`Borders::ALL` + `highlight_symbol("> ")`）行用の狭幅フォールバック。
///
/// `content_width < MIN_CONTENT_WIDTH` のときだけ `LIST_DECORATION_WIDTH` を差し引いた
/// inner width で `truncate_to_width` を適用する。それ以外はそのまま返す。
pub fn truncate_for_list(content_width: u16, text: &str) -> String {
    truncate_when_narrow(content_width, LIST_DECORATION_WIDTH, text)
}

/// `Paragraph + Block(Borders::ALL)`（`highlight_symbol` なし）行用の狭幅フォールバック。
///
/// `content_width < MIN_CONTENT_WIDTH` のときだけ `BLOCK_BORDER_WIDTH` を差し引いた
/// inner width で `truncate_to_width` を適用する。それ以外はそのまま返す。
pub fn truncate_for_paragraph(content_width: u16, text: &str) -> String {
    truncate_when_narrow(content_width, BLOCK_BORDER_WIDTH, text)
}

#[cfg(test)]
#[path = "common_test.rs"]
mod common_test;
