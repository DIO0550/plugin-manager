//! `--outdated` 出力フォーマット
//!
//! `render_outdated_json` は HostClient に依存しない純粋 render 関数。
//! live ネットワーク呼び出しを含む `run_outdated` の実装はこのファイルにある。

use super::wire::OutdatedWire;
use crate::host::{HostClientFactory, HostKind};
use crate::plugin::{
    fetch_remote_versions, meta, InstalledPlugin, PackageCacheAccess, PluginMeta, UpgradeState,
};
use comfy_table::{presets::UTF8_FULL, Table};

pub(super) fn render_outdated_json(
    entries: &[(&InstalledPlugin, &UpgradeState)],
) -> Result<String, String> {
    let wires: Vec<OutdatedWire<'_>> = entries
        .iter()
        .map(|(plugin, check)| OutdatedWire::from_entry(plugin, check))
        .collect();
    serde_json::to_string_pretty(&wires).map_err(|e| format!("Failed to serialize plugins: {}", e))
}

pub(super) async fn run_outdated(
    cache: &dyn PackageCacheAccess,
    plugins: &[InstalledPlugin],
    json: bool,
    total_count: usize,
) -> Result<(), String> {
    if plugins.is_empty() {
        super::print_empty_list(total_count);
        return Ok(());
    }

    let plugin_metas: Vec<(String, PluginMeta)> = plugins
        .iter()
        .map(|plugin| {
            let plugin_path = cache.plugin_path(plugin.marketplace(), plugin.install_id());
            let plugin_meta = meta::load_meta(&plugin_path).unwrap_or_default();
            (plugin.install_id().to_string(), plugin_meta)
        })
        .collect();

    let factory = HostClientFactory::with_defaults();
    let client = factory.create(HostKind::GitHub);
    let remote_versions = fetch_remote_versions(&plugin_metas, client.as_ref()).await;

    let results: Vec<(&InstalledPlugin, UpgradeState)> = plugins
        .iter()
        .zip(plugin_metas.iter())
        .zip(remote_versions.iter())
        .map(|((plugin, (_, meta)), (_, result))| (plugin, UpgradeState::from_query(meta, result)))
        .collect();

    if json {
        print_outdated_json(&results)?;
    } else {
        print_outdated_table(&results, total_count);
    }

    Ok(())
}

fn print_outdated_json(results: &[(&InstalledPlugin, UpgradeState)]) -> Result<(), String> {
    let entries: Vec<(&InstalledPlugin, &UpgradeState)> =
        results.iter().map(|(p, c)| (*p, c)).collect();
    let s = render_outdated_json(&entries)?;
    println!("{s}");
    Ok(())
}

fn print_outdated_table(results: &[(&InstalledPlugin, UpgradeState)], total_count: usize) {
    let with_updates = results
        .iter()
        .filter(|(_, c)| c.has_update())
        .collect::<Vec<_>>();
    let error_count = results.iter().filter(|(_, c)| c.is_unknown()).count();

    if with_updates.is_empty() {
        print_no_updates_message(total_count);
    } else {
        print_updates_table(&with_updates);
    }

    if error_count > 0 {
        println!(
            "{} plugin(s) could not be checked (use --json for details)",
            error_count
        );
    }
}

fn print_no_updates_message(total_count: usize) {
    let msg = match total_count {
        0 => "No plugins installed",
        _ => "All plugins are up to date",
    };
    println!("{msg}");
}

fn print_updates_table(with_updates: &[&(&InstalledPlugin, UpgradeState)]) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_header(vec!["Name", "Version", "Current SHA", "Latest SHA"])
        .add_rows(with_updates.iter().copied().map(update_row));

    println!("{table}");
    println!("{} plugin(s) have updates available", with_updates.len());
}

fn update_row((plugin, check): &(&InstalledPlugin, UpgradeState)) -> Vec<String> {
    vec![
        plugin.name().to_string(),
        plugin.version().to_string(),
        check
            .current_sha()
            .map(truncate_sha)
            .unwrap_or_else(|| "unknown".to_string()),
        check
            .latest_sha()
            .map(truncate_sha)
            .unwrap_or_else(|| "-".to_string()),
    ]
}

fn truncate_sha(sha: &str) -> String {
    sha.get(..7).unwrap_or(sha).to_string()
}

#[cfg(test)]
#[path = "outdated_test.rs"]
mod tests;
