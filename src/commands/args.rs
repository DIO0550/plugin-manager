//! サブコマンド間で共有するCLI引数部品。
//!
//! 用途別にサブモジュールへ分割し、本ファイルは再エクスポートのみを行う。
//! 旧パス `crate::commands::args::{ListOutputArgs, ...}` は `pub use` 経由で維持する。

mod marketplace;
mod output;
mod scope;
mod target;

#[allow(unused_imports)]
pub use marketplace::MarketplaceArgs;
pub use output::ListOutputArgs;
pub use scope::{InteractiveScopeArgs, SyncScopeArgs};
pub use target::{MultiTargetArgs, SingleTargetArgs};
