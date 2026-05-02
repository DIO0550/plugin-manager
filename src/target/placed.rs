//! 配置済みプラグイン収集ユーティリティ群
//!
//! `placed`: 全ターゲット横断スキャン
//! `placed_common`: Instruction の placement 共通ロジック
//! `scanner`: フラット 1 階層スキャナ

#[allow(clippy::module_inception)]
mod placed;
pub(crate) mod placed_common;
pub mod scanner;

pub(crate) use placed::list_all_placed;
