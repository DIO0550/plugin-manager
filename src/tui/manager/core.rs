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
#[allow(unused_imports)]
pub use common::{
    content_rect, render_filter_bar, truncate_to_width, BLOCK_BORDER_WIDTH, HORIZONTAL_PADDING,
    LIST_DECORATION_WIDTH, LIST_HIGHLIGHT_WIDTH, MIN_CONTENT_WIDTH,
};
pub use data::{DataStore, MarketplaceItem, PluginId};
pub use filter::filter_plugins;
