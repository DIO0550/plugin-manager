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

#[cfg(test)]
mod app_test;
#[cfg(test)]
mod data_test;
#[cfg(test)]
mod filter_test;

pub use app::{update, view, Model, Tab};
// `LIST_HIGHLIGHT_WIDTH` は現状内部利用がないが、`LIST_DECORATION_WIDTH` の内訳として
// 公開 API を維持する（将来 List 装飾の構成を変える際の参照点として残す）。
// 単独利用がないため `unused_imports` lint を抑制する。他の re-export 項目には影響しない。
#[allow(unused_imports)]
pub use common::LIST_HIGHLIGHT_WIDTH;
pub use common::{
    content_rect, render_filter_bar, truncate_to_width, BLOCK_BORDER_WIDTH, HORIZONTAL_PADDING,
    LIST_DECORATION_WIDTH, MIN_CONTENT_WIDTH,
};
pub use data::{DataStore, MarketplaceItem, PluginId};
pub use filter::filter_plugins;
