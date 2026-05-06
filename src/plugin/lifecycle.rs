mod action;
mod intent;
pub(crate) mod plugin_resolver;
mod update;

pub use action::PluginAction;
pub use intent::PluginIntent;
pub use update::{update_all_plugins, update_plugin, UpdateOutcome, UpdateResult, UpdateStatus};
