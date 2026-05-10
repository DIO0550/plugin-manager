//! サブコマンド間で共有するCLI引数部品。
//!
//! 用途別にサブモジュールへ分割し、本ファイルは再エクスポートのみを行う。
//! 旧パス `crate::commands::args::{ListOutputArgs, ...}` は `pub use` 経由で維持する。

mod output;
mod scope;
mod target;

pub use output::ListOutputArgs;
pub use scope::{InteractiveScopeArgs, SyncScopeArgs};
pub use target::{MultiTargetArgs, SingleTargetArgs};
