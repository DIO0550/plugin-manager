//! アプリケーション層
//!
//! ユースケースを提供する。

mod catalog;
mod info;
mod lifecycle;

pub use crate::plugin::InstalledPlugin;
pub use crate::target::{OperationOutcome, OperationResult};
pub use catalog::list_installed_plugins;
pub use info::{get_plugin_info, PluginInfo, Source};
pub use lifecycle::{
    disable_plugin, enable_plugin, get_uninstall_info, uninstall_plugin, UninstallInfo,
};
