//! テーブル出力フォーマット

use crate::plugin::InstalledPlugin;
use comfy_table::{presets::UTF8_FULL, Table};

/// Prints installed plugins as a formatted table.
///
/// # Arguments
///
/// * `plugins` - Installed plugins to render.
/// * `total_count` - Total number of installed plugins (for empty-state messages).
pub(super) fn print_table(plugins: &[InstalledPlugin], total_count: usize) {
    if plugins.is_empty() {
        super::print_empty_list(total_count);
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_header(vec![
            "Name",
            "Version",
            "Components",
            "Status",
            "Marketplace",
        ])
        .add_rows(plugins.iter().map(plugin_row));

    println!("{table}");
}

/// Builds a table row for a single installed plugin.
///
/// # Arguments
///
/// * `plugin` - Plugin to render as a row.
fn plugin_row(plugin: &InstalledPlugin) -> Vec<String> {
    vec![
        plugin.name().to_string(),
        plugin.version().to_string(),
        format_components(plugin),
        status_label(plugin.enabled()).to_string(),
        plugin.marketplace().unwrap_or("-").to_string(),
    ]
}

/// Returns the human-readable status label for a plugin.
///
/// # Arguments
///
/// * `enabled` - Whether the plugin is enabled.
fn status_label(enabled: bool) -> &'static str {
    if enabled {
        "enabled"
    } else {
        "disabled"
    }
}

/// Formats a plugin's component counts as a comma-separated summary.
///
/// # Arguments
///
/// * `plugin` - Plugin whose component counts should be rendered.
pub(super) fn format_components(plugin: &InstalledPlugin) -> String {
    let counts = plugin.component_type_counts();
    if counts.is_empty() {
        return "-".to_string();
    }
    counts
        .iter()
        .map(|(kind, count)| format!("{} {}", count, kind.plural()))
        .collect::<Vec<_>>()
        .join(", ")
}
