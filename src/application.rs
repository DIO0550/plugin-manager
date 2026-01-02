//! アプリケーション層
//!
//! ユースケースとDTOを提供する。

mod plugin_catalog;

pub use plugin_catalog::{list_installed_plugins, PluginSummary};
