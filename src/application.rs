//! アプリケーション層
//!
//! ユースケースを提供する。

mod plugin_action;
mod plugin_catalog;
mod plugin_info;
mod plugin_operations;

pub use plugin_action::{FileOperation, PluginAction, PluginIntent, ScopedPath, TargetId};
pub use plugin_catalog::{list_installed_plugins, PluginSummary};
pub use plugin_info::{get_plugin_info, AuthorInfo, ComponentInfo, PluginDetail, PluginSource};
pub use crate::target::OperationResult;
pub use plugin_operations::{
    disable_plugin, enable_plugin, get_uninstall_info, uninstall_plugin, PluginDeployment,
    UninstallInfo,
};
