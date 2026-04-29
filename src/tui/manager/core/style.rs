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
pub fn bordered_block(title: &str) -> Block<'_> {
    Block::default()
        .title(Span::styled(
            title.to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
}

/// 選択行用 Style を返す（fg=Black + bg=Green + BOLD）。
pub fn highlight_style() -> Style {
    Style::default()
        .add_modifier(Modifier::BOLD)
        .fg(Color::Black)
        .bg(Color::Green)
}

/// `bordered_block(title)` + `highlight_style()` + `HIGHLIGHT_SYMBOL`
/// を組み合わせた List ファクトリ。
/// `repeat_highlight_symbol(false)` を明示。
pub fn selectable_list<'a>(items: Vec<ListItem<'a>>, title: &'a str) -> List<'a> {
    List::new(items)
        .block(bordered_block(title))
        .highlight_style(highlight_style())
        .highlight_symbol(HIGHLIGHT_SYMBOL)
        .repeat_highlight_symbol(false)
}

/// Block なしの action menu 用 List ファクトリ。
/// `repeat_highlight_symbol(false)` を明示。
pub fn menu_list<'a>(items: Vec<ListItem<'a>>) -> List<'a> {
    List::new(items)
        .highlight_style(highlight_style())
        .highlight_symbol(HIGHLIGHT_SYMBOL)
        .repeat_highlight_symbol(false)
}

#[cfg(test)]
#[path = "style_test.rs"]
mod style_test;
