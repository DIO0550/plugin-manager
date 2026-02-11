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
mod filter_test;

pub use app::{update, view, Model, Tab};
pub use common::{dialog_rect, render_filter_bar};
pub use data::{DataStore, MarketplaceItem, PluginId};
pub use filter::filter_plugins;
