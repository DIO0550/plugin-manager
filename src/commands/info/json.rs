//! JSON 出力フォーマット

use super::wire::Wire;
use crate::application::PluginInfo;

pub(super) fn render_json(info: &PluginInfo) -> Result<String, String> {
    let wire = Wire::from(info);
    serde_json::to_string_pretty(&wire).map_err(|e| format!("Failed to serialize to JSON: {}", e))
}

pub(super) fn print_json(info: &PluginInfo) -> Result<(), String> {
    let s = render_json(info)?;
    println!("{s}");
    Ok(())
}
