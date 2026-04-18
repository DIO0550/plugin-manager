//! JSON 出力フォーマット

use crate::application::PluginDetail;

pub(super) fn render_json(detail: &PluginDetail) -> Result<String, String> {
    serde_json::to_string_pretty(detail).map_err(|e| format!("Failed to serialize to JSON: {}", e))
}

pub(super) fn print_json(detail: &PluginDetail) -> Result<(), String> {
    let s = render_json(detail)?;
    println!("{s}");
    Ok(())
}
