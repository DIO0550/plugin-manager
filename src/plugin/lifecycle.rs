mod action;
mod intent;
pub(crate) mod plugin_resolver;
mod update;

pub use action::PluginAction;
pub use intent::PluginIntent;
pub use plugin_resolver::{find_by_plugin_name, ResolvedPlugin};
pub use update::{update_all_plugins, update_plugin, UpdateResult, UpdateStatus};
