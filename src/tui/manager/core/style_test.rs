use super::*;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::ListItem;

#[test]
fn list_item_indent_is_two_spaces() {
    assert_eq!(LIST_ITEM_INDENT, "  ");
    assert_eq!(LIST_ITEM_INDENT.chars().count(), 2);
}

#[test]
fn highlight_symbol_is_arrow_space() {
    assert_eq!(HIGHLIGHT_SYMBOL, "> ");
}

#[test]
fn icon_constants_values() {
    assert_eq!(ICON_ENABLED, "●");
    assert_eq!(ICON_DISABLED, "○");
    assert_eq!(ICON_CHECK, "✓");
}

#[test]
fn icon_constants_are_single_char() {
    assert_eq!(ICON_ENABLED.chars().count(), 1);
    assert_eq!(ICON_DISABLED.chars().count(), 1);
    assert_eq!(ICON_CHECK.chars().count(), 1);
}

#[test]
fn mark_constants_have_fixed_len() {
    assert_eq!(MARK_MARKED.chars().count(), MARK_LEN);
    assert_eq!(MARK_UNMARKED.chars().count(), MARK_LEN);
}

#[test]
fn checkbox_constants_have_fixed_len() {
    assert_eq!(CHECKBOX_SELECTED.chars().count(), CHECKBOX_LEN);
    assert_eq!(CHECKBOX_UNSELECTED.chars().count(), CHECKBOX_LEN);
}

#[test]
fn radio_constants_have_fixed_len() {
    assert_eq!(RADIO_SELECTED.chars().count(), RADIO_LEN);
    assert_eq!(RADIO_UNSELECTED.chars().count(), RADIO_LEN);
}

#[test]
fn highlight_style_has_bold_black_fg_and_green_bg() {
    let s = highlight_style();
    assert_eq!(s.fg, Some(Color::Black));
    assert_eq!(s.bg, Some(Color::Green));
    assert!(s.add_modifier.contains(Modifier::BOLD));
}

#[test]
fn bordered_block_smoke() {
    let _block = bordered_block(" Title ");
}

#[test]
fn selectable_list_smoke() {
    let items = vec![ListItem::new("a"), ListItem::new("b")];
    let _list = selectable_list(items, " Title ");
}

#[test]
fn menu_list_smoke() {
    let items = vec![ListItem::new("a"), ListItem::new("b")];
    let _list = menu_list(items);
}

#[test]
fn highlight_line_unselected_preserves_span_styles() {
    let yellow = Style::default().fg(Color::Yellow);
    let spans = vec![Span::styled("alpha", yellow), Span::raw(" beta")];
    let line = highlight_line(spans.clone(), false);
    assert_eq!(line.style, Style::default());
    assert_eq!(line.spans, spans);
}

#[test]
fn highlight_line_selected_paints_line_and_overrides_span_fg() {
    let green_fg = Style::default().fg(Color::Green);
    let yellow_fg = Style::default().fg(Color::Yellow);
    let spans = vec![
        Span::styled("Update now", green_fg),
        Span::styled(" tail", yellow_fg),
    ];
    let line = highlight_line(spans, true);
    let hl = highlight_style();
    // Line.style 全体に highlight_style() が乗る → 余白セルも緑背景
    assert_eq!(line.style, hl);
    // 各 Span は元 fg を上書きされ、bg=Green / fg=Black / BOLD になる
    for span in line.spans.iter() {
        assert_eq!(span.style.fg, Some(Color::Black));
        assert_eq!(span.style.bg, Some(Color::Green));
        assert!(span.style.add_modifier.contains(Modifier::BOLD));
    }
}

#[test]
fn highlight_line_selected_with_empty_spans_returns_styled_line() {
    let spans: Vec<Span<'static>> = Vec::new();
    let line = highlight_line(spans, true);
    assert_eq!(line.style, highlight_style());
    assert!(line.spans.is_empty());
}
