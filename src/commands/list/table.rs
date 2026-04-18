//! テーブル出力フォーマット

use crate::plugin::InstalledPlugin;
use comfy_table::{presets::UTF8_FULL, Table};

pub(super) fn print_table(plugins: &[InstalledPlugin], total_count: usize) {
    if plugins.is_empty() {
        if total_count == 0 {
            println!("No plugins installed");
        } else {
            println!("No plugins matched");
        }
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        "Name",
        "Version",
        "Components",
        "Status",
        "Marketplace",
    ]);

    for plugin in plugins {
        let status = if plugin.enabled() {
            "enabled"
        } else {
            "disabled"
        };
        let marketplace = plugin.marketplace().unwrap_or("-");
        let components = format_components(plugin);

        table.add_row(vec![
            plugin.name(),
            plugin.version(),
            components.as_str(),
            status,
            marketplace,
        ]);
    }

    println!("{table}");
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
