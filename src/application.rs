//! アプリケーション層
//!
//! ユースケースを提供する。

mod plugin_action;
mod plugin_action_types;
mod plugin_catalog;
mod plugin_deployment;
mod plugin_info;
mod plugin_info_types;
mod plugin_intent;
mod plugin_operations;

pub use crate::target::OperationResult;
pub use plugin_catalog::{list_installed_plugins, PluginSummary};
pub use plugin_info::get_plugin_info;
pub use plugin_info_types::{PluginDetail, PluginSource};
// Re-exported for tests
#[cfg(test)]
pub use plugin_info_types::{AuthorInfo, ComponentInfo};
pub use plugin_operations::{
    disable_plugin, enable_plugin, get_uninstall_info, uninstall_plugin, UninstallInfo,
};
