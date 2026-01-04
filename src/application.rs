//! アプリケーション層
//!
//! ユースケースを提供する。

mod plugin_catalog;
mod plugin_operations;

pub use plugin_catalog::{list_installed_plugins, PluginSummary};
pub use crate::target::OperationResult;
pub use plugin_operations::{disable_plugin, uninstall_plugin};
