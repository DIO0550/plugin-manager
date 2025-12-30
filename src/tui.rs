//! TUI (Terminal User Interface) コンポーネント
//!
//! ratatui/crossterm を使用した選択ダイアログを提供する。

mod dialog;
mod scope_select;
mod target_select;

pub use scope_select::select_scope;
pub use target_select::select_targets;
