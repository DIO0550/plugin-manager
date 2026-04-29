//! 共通スタイル API
//!
//! TUI 各画面で共有するスタイル（タイトル装飾、選択行強調、リストアイコン）。

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, List, ListItem};

/// リスト項目の左インデント（highlight_symbol "> " と整合させるため固定）
pub const LIST_ITEM_INDENT: &str = "  ";

/// 選択行プレフィックス
pub const HIGHLIGHT_SYMBOL: &str = "> ";

/// 単一文字状態アイコン
pub const ICON_ENABLED: &str = "●";
pub const ICON_DISABLED: &str = "○";
pub const ICON_CHECK: &str = "✓";

/// Installed plugin list の mark ブロック
pub const MARK_MARKED: &str = "[✓]";
pub const MARK_UNMARKED: &str = "[ ]";
pub const MARK_LEN: usize = 3;

/// Marketplaces target チェックボックス
pub const CHECKBOX_SELECTED: &str = "[x]";
pub const CHECKBOX_UNSELECTED: &str = "[ ]";
pub const CHECKBOX_LEN: usize = 3;

/// Marketplaces scope ラジオ
pub const RADIO_SELECTED: &str = "(●)";
pub const RADIO_UNSELECTED: &str = "( )";
pub const RADIO_LEN: usize = 3;

/// BOLD タイトル付きのボーダーブロックを返す。
///
/// `title` はそのまま `Block` に借用させ、レンダリングごとの `String` 確保を避ける。
pub fn bordered_block<'a>(title: &'a str) -> Block<'a> {
    Block::default()
        .title(Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
}

/// 選択行の **内容セル** にだけ適用する Style（fg=Black + bg=Green + BOLD）。
///
/// 直接 `List::highlight_style` には設定しない。`List::highlight_style` は ListItem
/// 全体に塗布されるため、空行も緑背景になってしまう。代わりに各画面の builder が
/// 内容行の Span にだけ `highlight_spans` 経由でこのスタイルを適用する。
pub fn highlight_style() -> Style {
    Style::default()
        .add_modifier(Modifier::BOLD)
        .fg(Color::Black)
        .bg(Color::Green)
}

/// 選択行のときに各 Span を `highlight_style()` で patch して返す。
///
/// 既存の Span スタイル（fg 色など）の上に highlight_style を上書きすることで、
/// 緑色の文字が緑背景で見えなくなるなどの問題を防ぐ。`is_selected = false` の
/// ときは入力をそのまま返す。
pub fn highlight_spans<'a>(spans: Vec<Span<'a>>, is_selected: bool) -> Vec<Span<'a>> {
    if !is_selected {
        return spans;
    }
    let hl = highlight_style();
    spans
        .into_iter()
        .map(|s| {
            let style = s.style.patch(hl);
            Span::styled(s.content, style)
        })
        .collect()
}

/// `bordered_block(title)` + `HIGHLIGHT_SYMBOL` を組み合わせた List ファクトリ。
///
/// `repeat_highlight_symbol(false)` を明示。`highlight_style` は **設定しない**：
/// 緑背景は内容行の Span 側で `highlight_spans` 経由で適用するため、
/// ListItem 全体（空行を含む）には塗布しない。
pub fn selectable_list<'a>(items: Vec<ListItem<'a>>, title: &'a str) -> List<'a> {
    List::new(items)
        .block(bordered_block(title))
        .highlight_symbol(HIGHLIGHT_SYMBOL)
        .repeat_highlight_symbol(false)
}

/// Block なしの action menu 用 List ファクトリ。
///
/// `repeat_highlight_symbol(false)` を明示。`highlight_style` は **設定しない**：
/// 緑背景は内容行の Span 側で `highlight_spans` 経由で適用する。
pub fn menu_list<'a>(items: Vec<ListItem<'a>>) -> List<'a> {
    List::new(items)
        .highlight_symbol(HIGHLIGHT_SYMBOL)
        .repeat_highlight_symbol(false)
}

#[cfg(test)]
#[path = "style_test.rs"]
mod style_test;
