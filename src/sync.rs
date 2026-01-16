//! 同期 Feature
//!
//! 異なるターゲット環境間でコンポーネントを同期する。

mod executor;
mod types;

pub use executor::SyncExecutor;
pub use types::{SyncAction, SyncFailure, SyncItem, SyncOptions, SyncPlan, SyncResult, SyncableKind};
