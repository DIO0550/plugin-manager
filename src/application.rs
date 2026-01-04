//! アプリケーション層
//!
//! ユースケースを提供する。

mod plugin_catalog;

pub use plugin_catalog::{list_installed_plugins, PluginSummary};
