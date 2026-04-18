//! `--outdated` 出力フォーマット
//!
//! `render_outdated_json` は HostClient に依存しない純粋 render 関数。
//! live ネットワーク呼び出しを含む `run` の実装は module root 側。

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
        if total_count == 0 {
            println!("No plugins installed");
        } else {
            println!("No plugins matched");
        }
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
        if total_count == 0 {
            println!("No plugins installed");
        } else {
            println!("All plugins are up to date");
        }
    } else {
        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(vec!["Name", "Version", "Current SHA", "Latest SHA"]);

        with_updates.iter().for_each(|&&(plugin, ref check)| {
            let current_sha = check
                .current_sha()
                .map(truncate_sha)
                .unwrap_or_else(|| "unknown".to_string());
            let latest_sha = check
                .latest_sha()
                .map(truncate_sha)
                .unwrap_or_else(|| "-".to_string());

            table.add_row(vec![
                plugin.name(),
                plugin.version(),
                &current_sha,
                &latest_sha,
            ]);
        });

        println!("{table}");
        println!("{} plugin(s) have updates available", with_updates.len());
    }

    if error_count > 0 {
        println!(
            "{} plugin(s) could not be checked (use --json for details)",
            error_count
        );
    }
}

fn truncate_sha(sha: &str) -> String {
    if sha.len() > 7 {
        sha[..7].to_string()
    } else {
        sha.to_string()
    }
}

#[cfg(test)]
#[path = "outdated_test.rs"]
mod tests;
