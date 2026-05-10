//! ターゲットコアドメイン
//!
//! `id`: TargetId 値オブジェクト
//! `paths`: home_dir / base_dir 共通パス計算
//! `registry`: TargetRegistry 状態マシン

mod id;
pub(crate) mod paths;
mod registry;

pub use id::TargetId;
pub use registry::{AddOutcome, RemoveOutcome, TargetRegistry};
