//! テーブル出力フォーマット

use crate::plugin::InstalledPlugin;
use comfy_table::{presets::UTF8_FULL, Table};

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

fn plugin_row(plugin: &InstalledPlugin) -> Vec<String> {
    vec![
        plugin.name().to_string(),
        plugin.version().to_string(),
        format_components(plugin),
        status_label(plugin.enabled()).to_string(),
        plugin.marketplace().unwrap_or("-").to_string(),
    ]
}

fn status_label(enabled: bool) -> &'static str {
    if enabled {
        "enabled"
    } else {
        "disabled"
    }
}

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
