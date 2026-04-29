//! 共通レイアウト API
//!
//! TUI 各画面で共有するレイアウト計算ユーティリティ。
//! 各 view では通常 `outer_rect` を起点として呼ぶ。
//! `frame_rect` は明示的にパディング量を指定したい場合に使う低レベル API。

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// 外周パディング水平方向（cells）
pub const OUTER_PADDING_H: u16 = 1;

/// 外周パディング垂直方向（cells）
pub const OUTER_PADDING_V: u16 = 1;

/// このターミナル高さ未満はパディング 0 にフォールバック
pub const MIN_TERMINAL_HEIGHT: u16 = 8;

/// このターミナル幅未満は左右パディング 0 にフォールバック
pub const MIN_TERMINAL_WIDTH: u16 = 20;

/// 指定矩形の上下左右にパディングを適用した内側 Rect を返す。
///
/// `area.width < 2*h_pad` または `area.height < 2*v_pad` の場合は
/// 該当方向のみ縮退（パディング 0 に丸める）して overflow を回避する。
pub fn frame_rect(area: Rect, h_pad: u16, v_pad: u16) -> Rect {
    let (h_pad_eff, h_total_eff) = effective_padding(area.width, h_pad);
    let (v_pad_eff, v_total_eff) = effective_padding(area.height, v_pad);

    Rect::new(
        area.x.saturating_add(h_pad_eff),
        area.y.saturating_add(v_pad_eff),
        area.width.saturating_sub(h_total_eff),
        area.height.saturating_sub(v_total_eff),
    )
}

/// `axis_size < 2*pad` のとき pad/total を 0 に縮退する。
fn effective_padding(axis_size: u16, pad: u16) -> (u16, u16) {
    let total = pad.saturating_mul(2);
    if axis_size < total {
        (0, 0)
    } else {
        (pad, total)
    }
}

/// 外周マージンを適用した内側 Rect を返す（**各 view から呼ぶ正式 API**）。
///
/// 極小ターミナル時は自動でマージン 0 にフォールバックする。
/// - `area.height < MIN_TERMINAL_HEIGHT` のとき上下マージンを 0
/// - `area.width < MIN_TERMINAL_WIDTH` のとき左右マージンを 0
pub fn outer_rect(area: Rect) -> Rect {
    let v_pad = if area.height < MIN_TERMINAL_HEIGHT {
        0
    } else {
        OUTER_PADDING_V
    };
    let h_pad = if area.width < MIN_TERMINAL_WIDTH {
        0
    } else {
        OUTER_PADDING_H
    };
    frame_rect(area, h_pad, v_pad)
}

/// タブバー (1) / フィルタ (3) / コンテンツ (Min(1)) / ヘルプ (1) の
/// トップレベル 4 区画に縦分割した Rect を返す。
///
/// **注**: トップレベル専用。詳細画面やモーダルでは使わない。
pub fn framed_layout(outer: Rect) -> [Rect; 4] {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(outer);
    [chunks[0], chunks[1], chunks[2], chunks[3]]
}

/// Installed plugin detail 用 2 区画レイアウト
/// `(info_pane: Min(1), action_menu: Length(action_menu_rows))`
///
/// help 行は呼び出し元で既に `framed_layout` の help_area に確保しているため、
/// ここでは含めない。
///
/// `action_menu_rows` は **action menu の描画行数（論理 action 件数ではない）**。
/// 呼び出し側では固定係数で見積もらず、
/// `items.iter().map(ListItem::height).sum()` のように
/// 実際に描画する item の高さ合計を渡すこと。
pub fn detail_layout(content: Rect, action_menu_rows: u16) -> [Rect; 2] {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(action_menu_rows)])
        .split(content);
    [chunks[0], chunks[1]]
}

/// 中央寄せモーダルの **外枠 (centered area) のみ** を返す。
///
/// 内部分割（コンテンツ / help / gauge など）は本 API の責務外。
/// 各モーダル側で `Layout::default().direction(Vertical).constraints([...]).split(outer)`
/// として内部分割する。
///
/// `width_pct`, `height_pct` は `0..=100`。100 超は `min(100)` に丸める。
pub fn modal_layout(area: Rect, width_pct: u16, height_pct: u16) -> Rect {
    let modal_w = scale_with_min_visible(area.width, width_pct);
    let modal_h = scale_with_min_visible(area.height, height_pct);
    let modal_w = modal_w.min(area.width);
    let modal_h = modal_h.min(area.height);
    let x = area
        .x
        .saturating_add((area.width.saturating_sub(modal_w)) / 2);
    let y = area
        .y
        .saturating_add((area.height.saturating_sub(modal_h)) / 2);
    Rect::new(x, y, modal_w, modal_h)
}

/// `axis_size * pct / 100` を計算し、`pct > 0` かつ `axis_size > 0` の
/// ときは最低 1 セルを返す（極小ターミナルでもモーダルが消えないため）。
/// `pct` は内部で `min(100)` に丸める。
fn scale_with_min_visible(axis_size: u16, pct: u16) -> u16 {
    let p = pct.min(100);
    if p == 0 || axis_size == 0 {
        return 0;
    }
    let scaled = (axis_size as u32 * p as u32) / 100;
    if scaled == 0 {
        1
    } else {
        scaled as u16
    }
}

/// 水平 2 分割。Marketplaces browse の左右分割用。
///
/// `ratio == (0, 0)` でも panic せず、左 Rect が 0 幅・右 Rect が `area` 全体になる。
pub fn split_horizontal(area: Rect, ratio: (u32, u32)) -> [Rect; 2] {
    let total = ratio.0.saturating_add(ratio.1);
    if total == 0 {
        let left = Rect::new(area.x, area.y, 0, area.height);
        return [left, area];
    }
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(ratio.0, total),
            Constraint::Ratio(ratio.1, total),
        ])
        .split(area);
    [chunks[0], chunks[1]]
}

#[cfg(test)]
#[path = "layout_test.rs"]
mod layout_test;
