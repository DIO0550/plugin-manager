//! `content_rect` と `truncate_to_width` の境界値テスト

use super::*;
use ratatui::layout::Rect;

// =============================================================================
// content_rect — 横はパディング差し引き、縦は触らない
// =============================================================================

#[test]
fn content_rect_subtracts_horizontal_padding_from_full_width() {
    let area = Rect::new(0, 0, 100, 30);
    let result = content_rect(area, 2);
    assert_eq!(result, Rect::new(2, 0, 96, 30));
}

#[test]
fn content_rect_with_zero_padding_returns_same_area() {
    let area = Rect::new(0, 0, 100, 30);
    assert_eq!(content_rect(area, 0), area);
}

#[test]
fn content_rect_preserves_height() {
    let area = Rect::new(0, 0, 50, 17);
    let result = content_rect(area, 2);
    assert_eq!(result.height, 17);
}

#[test]
fn content_rect_preserves_y() {
    let area = Rect::new(0, 5, 50, 10);
    let result = content_rect(area, 3);
    assert_eq!(result.y, 5);
}

#[test]
fn content_rect_shifts_x_by_padding() {
    let area = Rect::new(10, 0, 50, 10);
    let result = content_rect(area, 2);
    assert_eq!(result.x, 12);
}

#[test]
fn content_rect_with_width_equal_to_padding_returns_zero_width() {
    let area = Rect::new(0, 0, 4, 10);
    let result = content_rect(area, 2);
    assert_eq!(result.width, 0);
}

#[test]
fn content_rect_with_width_one_above_padding_returns_one() {
    let area = Rect::new(0, 0, 5, 10);
    let result = content_rect(area, 2);
    assert_eq!(result.width, 1);
}

#[test]
fn content_rect_with_narrow_width_saturates_to_zero() {
    let area = Rect::new(0, 0, 3, 10);
    let result = content_rect(area, 2);
    assert_eq!(result.width, 0);
}

#[test]
fn content_rect_with_zero_width_does_not_panic() {
    let area = Rect::new(0, 0, 0, 10);
    let result = content_rect(area, 2);
    assert_eq!(result.width, 0);
}

#[test]
fn content_rect_with_huge_padding_does_not_overflow() {
    let area = Rect::new(0, 0, 10, 10);
    let result = content_rect(area, u16::MAX);
    assert_eq!(result.width, 0);
}

#[test]
fn content_rect_with_max_x_saturates_x_without_panic() {
    // area.x = u16::MAX で `area.x + padding` が桁あふれするケース。
    // saturating_add によって panic せず x が u16::MAX に飽和することを保証する。
    // （`Rect::new` は `x + width` の overflow を防ぐため width を clamp する点に注意）
    let area = Rect::new(u16::MAX, 0, 10, 10);
    let result = content_rect(area, 2);
    assert_eq!(result.x, u16::MAX, "x should saturate to u16::MAX");
    assert_eq!(result.y, 0);
    assert_eq!(result.height, 10, "height is preserved");
}

// =============================================================================
// truncate_to_width — 文字数ベースの切り詰め
// =============================================================================

#[test]
fn truncate_to_width_returns_text_when_within_limit() {
    assert_eq!(truncate_to_width("hello", 10), "hello");
}

#[test]
fn truncate_to_width_returns_text_when_exactly_at_limit() {
    assert_eq!(truncate_to_width("hello", 5), "hello");
}

#[test]
fn truncate_to_width_appends_ellipsis_when_exceeded() {
    assert_eq!(truncate_to_width("hello world", 8), "hello...");
}

#[test]
fn truncate_to_width_with_width_three_truncates_without_ellipsis() {
    assert_eq!(truncate_to_width("abc", 3), "abc");
    assert_eq!(truncate_to_width("abcd", 3), "abc");
}

#[test]
fn truncate_to_width_with_width_one_returns_first_char() {
    assert_eq!(truncate_to_width("abc", 1), "a");
}

#[test]
fn truncate_to_width_with_zero_width_returns_empty() {
    assert_eq!(truncate_to_width("abc", 0), "");
}

#[test]
fn truncate_to_width_with_empty_text_returns_empty() {
    assert_eq!(truncate_to_width("", 10), "");
}

#[test]
fn truncate_to_width_at_boundary_39() {
    let text = "a".repeat(40);
    let result = truncate_to_width(&text, 39);
    assert_eq!(result.chars().count(), 39);
    assert!(result.ends_with("..."));
}

#[test]
fn truncate_to_width_at_boundary_40() {
    let text = "a".repeat(40);
    assert_eq!(truncate_to_width(&text, 40), text);
}

#[test]
fn truncate_to_width_at_boundary_41() {
    let text = "a".repeat(40);
    assert_eq!(truncate_to_width(&text, 41), text);
}

// =============================================================================
// truncate_for_list / truncate_for_paragraph — 狭幅フォールバック
// =============================================================================

#[test]
fn truncate_for_list_returns_text_when_wide() {
    let text = "a".repeat(50);
    assert_eq!(truncate_for_list(80, &text), text);
}

#[test]
fn truncate_for_list_truncates_with_list_decoration_width_when_narrow() {
    let text = "a".repeat(50);
    let result = truncate_for_list(30, &text);
    let inner = (30u16 - LIST_DECORATION_WIDTH) as usize;
    assert_eq!(result.chars().count(), inner);
    assert!(result.ends_with(ELLIPSIS));
}

#[test]
fn truncate_for_paragraph_truncates_with_block_border_width_when_narrow() {
    let text = "a".repeat(50);
    let result = truncate_for_paragraph(30, &text);
    let inner = (30u16 - BLOCK_BORDER_WIDTH) as usize;
    assert_eq!(result.chars().count(), inner);
    assert!(result.ends_with(ELLIPSIS));
}

#[test]
fn truncate_for_list_does_not_panic_at_zero_width() {
    let _ = truncate_for_list(0, "abc");
}
