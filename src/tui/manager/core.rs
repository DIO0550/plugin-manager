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

pub use app::{update, view, Model, Tab};
pub use common::dialog_rect;
pub use data::{ComponentKind, DataStore, PluginId};
