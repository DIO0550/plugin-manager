//! YAML 出力フォーマット

use crate::application::PluginDetail;

pub(super) fn render_yaml(detail: &PluginDetail) -> Result<String, String> {
    serde_yaml::to_string(detail).map_err(|e| format!("Failed to serialize to YAML: {}", e))
}

pub(super) fn print_yaml(detail: &PluginDetail) -> Result<(), String> {
    let s = render_yaml(detail)?;
    print!("{s}");
    Ok(())
}
