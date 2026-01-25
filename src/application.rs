//! アプリケーション層
//!
//! ユースケースを提供する。

mod plugin_action;
mod plugin_catalog;
mod plugin_info;
mod plugin_operations;

pub use crate::target::OperationResult;
pub use plugin_catalog::{list_installed_plugins, PluginSummary};
pub use plugin_info::{get_plugin_info, PluginDetail, PluginSource};
// Re-exported for tests
#[cfg(test)]
pub use plugin_info::{AuthorInfo, ComponentInfo};
pub use plugin_operations::{
    disable_plugin, enable_plugin, get_uninstall_info, uninstall_plugin, UninstallInfo,
};
