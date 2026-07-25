//! ターゲットコアドメイン
//!
//! `paths`: home_dir / base_dir 共通パス計算
//! `layout`: TargetKind 向け配置パス API（#339）
//! `registry`: TargetRegistry 状態マシン

pub(crate) mod layout;
pub(crate) mod paths;
mod registry;

pub use registry::{AddOutcome, RemoveOutcome, TargetRegistry};
