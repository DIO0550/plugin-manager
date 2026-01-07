//! Installed タブの Model/Msg/update/view
//!
//! インストール済みプラグインの一覧表示と詳細確認。

pub mod actions;
mod model;
mod update;
mod view;

// Re-exports
pub use model::{key_to_msg, CacheState, Model, Msg};
pub use update::update;
pub use view::view;
