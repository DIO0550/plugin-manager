use super::*;
use ratatui::layout::Rect;

#[test]
fn frame_rect_applies_padding_for_normal_size() {
    let area = Rect::new(0, 0, 80, 24);
    let inner = frame_rect(area, 1, 1);
    assert_eq!(inner, Rect::new(1, 1, 78, 22));
}

#[test]
fn frame_rect_collapses_when_width_smaller_than_padding() {
    let area = Rect::new(0, 0, 1, 24);
    let inner = frame_rect(area, 1, 1);
    assert_eq!(inner, Rect::new(0, 1, 1, 22));
}

#[test]
fn frame_rect_collapses_when_height_smaller_than_padding() {
    let area = Rect::new(0, 0, 80, 1);
    let inner = frame_rect(area, 1, 1);
    assert_eq!(inner, Rect::new(1, 0, 78, 1));
}

#[test]
fn frame_rect_collapses_completely_for_tiny_input() {
    let area = Rect::new(0, 0, 1, 1);
    let inner = frame_rect(area, 1, 1);
    assert_eq!(inner, Rect::new(0, 0, 1, 1));
}

#[test]
fn frame_rect_handles_zero_input_without_panic() {
    let area = Rect::new(0, 0, 0, 0);
    let inner = frame_rect(area, 1, 1);
    assert_eq!(inner, Rect::new(0, 0, 0, 0));
}

#[test]
fn outer_rect_applies_default_margin() {
    let area = Rect::new(0, 0, 80, 24);
    let inner = outer_rect(area);
    assert_eq!(inner, Rect::new(1, 1, 78, 22));
}

#[test]
fn outer_rect_collapses_when_height_below_min() {
    let area = Rect::new(0, 0, 80, 4);
    let inner = outer_rect(area);
    assert_eq!(inner.y, 0);
    assert_eq!(inner.height, 4);
}

#[test]
fn outer_rect_collapses_when_width_below_min() {
    let area = Rect::new(0, 0, 10, 24);
    let inner = outer_rect(area);
    assert_eq!(inner.x, 0);
    assert_eq!(inner.width, 10);
}

#[test]
fn outer_rect_uses_zero_padding_for_small_height() {
    let area = Rect::new(0, 0, 80, 6);
    let inner = outer_rect(area);
    assert_eq!(inner.y, 0);
    assert_eq!(inner.height, 6);
}

#[test]
fn framed_layout_distributes_constraints_for_24_rows() {
    let outer = Rect::new(0, 0, 80, 24);
    let chunks = framed_layout(outer);
    let total: u16 = chunks.iter().map(|r| r.height).sum();
    assert_eq!(total, 24);
    assert_eq!(chunks[0].height, 1);
    assert_eq!(chunks[1].height, 3);
    assert!(chunks[2].height >= 1);
    assert_eq!(chunks[3].height, 1);
}

#[test]
fn framed_layout_min_terminal_height() {
    let outer = Rect::new(0, 0, 80, 6);
    let chunks = framed_layout(outer);
    assert!(chunks[2].height >= 1);
}

#[test]
fn detail_layout_returns_3_chunks_with_specified_action_menu_height() {
    let content = Rect::new(0, 0, 80, 20);
    let chunks = detail_layout(content, 4);
    assert_eq!(chunks[1].height, 4);
    assert_eq!(chunks[2].height, 1);
    assert!(chunks[0].height >= 1);
}

#[test]
fn detail_layout_respects_action_menu_rows() {
    let content = Rect::new(0, 0, 80, 20);
    let chunks = detail_layout(content, 10);
    assert_eq!(chunks[0].height, 9);
    assert_eq!(chunks[1].height, 10);
    assert_eq!(chunks[2].height, 1);
}

#[test]
fn detail_layout_action_menu_rows_zero_returns_empty_menu() {
    let content = Rect::new(0, 0, 80, 20);
    let chunks = detail_layout(content, 0);
    assert_eq!(chunks[1].height, 0);
    assert_eq!(chunks[2].height, 1);
    assert!(chunks[0].height >= 1);
}

#[test]
fn modal_layout_centers_within_area() {
    let area = Rect::new(0, 0, 100, 50);
    let inner = modal_layout(area, 50, 50);
    // 中央寄せ。height=25 (50%), width=50 (50%)
    assert_eq!(inner.width, 50);
    assert_eq!(inner.height, 25);
}

#[test]
fn modal_layout_with_pct_over_100_clamps_to_100() {
    let area = Rect::new(0, 0, 100, 50);
    let inner = modal_layout(area, 150, 200);
    assert_eq!(inner.width, area.width);
    assert_eq!(inner.height, area.height);
}

#[test]
fn modal_layout_with_zero_pct_returns_zero_size() {
    let area = Rect::new(0, 0, 100, 50);
    let inner = modal_layout(area, 0, 0);
    assert!(inner.width == 0 || inner.height == 0);
}

#[test]
fn split_horizontal_respects_ratio() {
    let area = Rect::new(0, 0, 100, 10);
    let chunks = split_horizontal(area, (30, 70));
    assert_eq!(chunks[0].width, 30);
    assert_eq!(chunks[1].width, 70);
}

#[test]
fn split_horizontal_with_zero_ratio_does_not_panic() {
    let area = Rect::new(0, 0, 100, 10);
    let chunks = split_horizontal(area, (0, 0));
    assert_eq!(chunks[0].width, 0);
    assert_eq!(chunks[1].width, 100);
}
