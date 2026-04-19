//! YAML 出力フォーマット

use super::wire::Wire;
use crate::application::PluginInfo;

/// Render `PluginInfo` as a YAML string.
///
/// # Arguments
///
/// * `info` - Plugin information to serialize.
pub(super) fn render_yaml(info: &PluginInfo) -> Result<String, String> {
    let wire = Wire::from(info);
    serde_yaml::to_string(&wire).map_err(|e| format!("Failed to serialize to YAML: {}", e))
}

/// Print `PluginInfo` as YAML to stdout.
///
/// # Arguments
///
/// * `info` - Plugin information to print.
pub(super) fn print_yaml(info: &PluginInfo) -> Result<(), String> {
    let s = render_yaml(info)?;
    print!("{s}");
    Ok(())
}
