//! Marketplaces タブの Model/Msg/update/view
//!
//! マーケットプレースソースの管理（一覧・追加・削除・更新・詳細表示）。

pub mod actions;
mod model;
mod update;
mod view;

// Re-exports
pub use model::{key_to_msg, CacheState, Model, Msg};
pub use update::update;
pub use view::view;
