//! コアモジュール
//!
//! TUI の基盤となる構造を提供する。
//!
//! - `app`: Model/Screen/Msg/update/view
//! - `data`: DataStore（共有データ）
//! - `common`: 共通 UI ユーティリティ

mod app;
mod common;
mod data;
pub mod filter;
pub mod layout;
pub mod style;

#[cfg(test)]
mod app_test;
#[cfg(test)]
mod data_test;
#[cfg(test)]
mod filter_test;

pub use app::{update, view, Model, Tab};
// `LIST_HIGHLIGHT_WIDTH` / `BLOCK_BORDER_WIDTH` は現状クレート内で直接参照していないが、
// 装飾幅の内訳定数として公開 API を維持する（List 装飾構成や Paragraph 系切り詰め予算を
// 外部から再構成できる参照点として残す）。単独利用がないため `unused_imports` lint を
// 抑制する。他の re-export 項目には影響しない。
pub use common::{
    render_filter_bar, truncate_for_list, truncate_for_paragraph, truncate_to_width,
    LIST_DECORATION_WIDTH, MIN_CONTENT_WIDTH,
};
#[allow(unused_imports)]
pub use common::{BLOCK_BORDER_WIDTH, LIST_HIGHLIGHT_WIDTH};
pub use data::{DataStore, MarketplaceItem, PluginId};
pub use filter::filter_plugins;
