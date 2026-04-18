//! JSON 出力フォーマット

use super::wire::Wire;
use crate::plugin::InstalledPlugin;

pub(super) fn render_json(plugins: &[InstalledPlugin]) -> Result<String, String> {
    let wires: Vec<Wire<'_>> = plugins.iter().map(Wire::from_installed).collect();
    serde_json::to_string_pretty(&wires).map_err(|e| format!("Failed to serialize plugins: {}", e))
}

pub(super) fn print_json(plugins: &[InstalledPlugin]) -> Result<(), String> {
    let s = render_json(plugins)?;
    println!("{s}");
    Ok(())
}
