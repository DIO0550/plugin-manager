//! 配置済みプラグイン収集ユーティリティ群
//!
//! `placed`: 全ターゲット横断スキャン
//! `scanner`: フラット 1 階層スキャナ
//! `filter` / `list_helpers` / `placement_helpers` / `scope_support`: env 共通抽出

pub(crate) mod filter;
pub(crate) mod list_helpers;
#[allow(clippy::module_inception)]
mod placed;
pub(crate) mod placement_helpers;
pub mod scanner;
pub(crate) mod scope_support;

pub(crate) use placed::list_all_placed;
