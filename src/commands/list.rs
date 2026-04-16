//! plm list コマンド
//!
//! インストール済みプラグインの一覧を表示する。

use crate::application::{list_installed_plugins, InstalledPlugin};
use crate::component::ComponentKind;
use crate::host::{HostClientFactory, HostKind};
use crate::plugin::{
    fetch_remote_versions, meta, PackageCache, PackageCacheAccess, PluginMeta, UpgradeState,
};
use crate::target::TargetKind;
use clap::Parser;
use comfy_table::{presets::UTF8_FULL, Table};
use serde::Serialize;

#[derive(Debug, Parser)]
pub struct Args {
    /// Filter by component type
    #[arg(long = "type", value_enum)]
    pub component_type: Option<ComponentKind>,

    /// Filter by target environment (currently filters by enabled status)
    #[arg(long, value_enum)]
    pub target: Option<TargetKind>,

    /// Output in JSON format
    #[arg(long, conflicts_with = "simple")]
    pub json: bool,

    /// Output only plugin names
    #[arg(long, conflicts_with = "json")]
    pub simple: bool,

    /// Show only plugins with available updates. Note: --json includes all plugins with update info.
    #[arg(long, conflicts_with = "simple")]
    pub outdated: bool,
}

/// `--outdated --json` 出力用エントリ（`plugin` と `check` のネスト形式）
#[derive(Debug, Serialize)]
struct OutdatedEntry<'a> {
    plugin: &'a InstalledPlugin,
    check: &'a UpgradeState,
}

pub async fn run(args: Args) -> Result<(), String> {
    // 1. プラグイン一覧を取得
    let cache = PackageCache::new().map_err(|e| format!("Failed to access cache: {e}"))?;
    let mut plugins = list_installed_plugins(&cache)
        .map_err(|e| format!("Failed to list installed plugins: {e}"))?;

    let total_count = plugins.len();

    // 2. ソート（name昇順）
    plugins.sort_by(|a, b| a.name().cmp(b.name()));

    // 3. フィルタリング
    let filtered = filter_plugins(plugins, &args);

    // 4. 出力（--outdated の場合は更新チェックを実行）
    if args.outdated {
        run_outdated(&cache, &filtered, &args, total_count).await?;
    } else if args.json {
        print_json(&filtered)?;
    } else if args.simple {
        print_simple(&filtered, total_count);
    } else {
        print_table(&filtered, total_count);
    }

    Ok(())
}

/// --outdated 用の更新チェック処理
async fn run_outdated(
    cache: &dyn PackageCacheAccess,
    plugins: &[InstalledPlugin],
    args: &Args,
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

    // PluginMeta を収集（宣言的）
    let plugin_metas: Vec<(String, PluginMeta)> = plugins
        .iter()
        .map(|plugin| {
            let plugin_path = cache.plugin_path(plugin.marketplace(), plugin.install_id());
            let plugin_meta = meta::load_meta(&plugin_path).unwrap_or_default();
            (plugin.install_id().to_string(), plugin_meta)
        })
        .collect();

    // GitHub クライアントを作成してリモートバージョンを取得
    let factory = HostClientFactory::with_defaults();
    let client = factory.create(HostKind::GitHub);
    let remote_versions = fetch_remote_versions(&plugin_metas, client.as_ref()).await;

    // InstalledPlugin と UpgradeState を結合
    let results: Vec<(&InstalledPlugin, UpgradeState)> = plugins
        .iter()
        .zip(plugin_metas.iter())
        .zip(remote_versions.iter())
        .map(|((plugin, (_, meta)), (_, result))| {
            (plugin, UpgradeState::from_query(meta, result))
        })
        .collect();

    // 出力
    if args.json {
        print_outdated_json(&results)?;
    } else {
        print_outdated_table(&results, total_count);
    }

    Ok(())
}

fn print_outdated_json(results: &[(&InstalledPlugin, UpgradeState)]) -> Result<(), String> {
    let entries = results
        .iter()
        .map(|(plugin, check)| OutdatedEntry { plugin, check })
        .collect::<Vec<_>>();
    serde_json::to_string_pretty(&entries)
        .map(|json| println!("{json}"))
        .map_err(|e| format!("Failed to serialize plugins: {}", e))
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

/// SHA を短縮表示（先頭7文字）
fn truncate_sha(sha: &str) -> String {
    if sha.len() > 7 {
        sha[..7].to_string()
    } else {
        sha.to_string()
    }
}

fn filter_plugins(plugins: Vec<InstalledPlugin>, args: &Args) -> Vec<InstalledPlugin> {
    plugins
        .into_iter()
        .filter(|p| filter_by_type(p, args.component_type.as_ref()))
        .filter(|p| filter_by_target(p, args.target.as_ref()))
        .collect()
}

fn filter_by_type(plugin: &InstalledPlugin, component_type: Option<&ComponentKind>) -> bool {
    match component_type {
        None => true,
        Some(kind) => plugin.components().iter().any(|c| c.kind == *kind),
    }
}

fn filter_by_target(plugin: &InstalledPlugin, target: Option<&TargetKind>) -> bool {
    // Phase 1: シンプルにenabled状態でフィルタ
    // ターゲット指定時は、そのターゲットで有効なプラグインのみ表示
    // 現状の InstalledPlugin にはターゲット別のデプロイ情報がないため、
    // enabled = true のプラグインを「ターゲットにデプロイ済み」とみなす
    match target {
        None => true,
        Some(_) => plugin.enabled(),
    }
}

fn print_table(plugins: &[InstalledPlugin], total_count: usize) {
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

fn print_json(plugins: &[InstalledPlugin]) -> Result<(), String> {
    // 空の場合も [] を出力
    serde_json::to_string_pretty(plugins)
        .map(|json| println!("{json}"))
        .map_err(|e| format!("Failed to serialize plugins: {}", e))
}

fn print_simple(plugins: &[InstalledPlugin], total_count: usize) {
    if plugins.is_empty() {
        if total_count == 0 {
            println!("No plugins installed");
        } else {
            println!("No plugins matched");
        }
        return;
    }
    for plugin in plugins {
        println!("{}", plugin.name());
    }
}

fn format_components(plugin: &InstalledPlugin) -> String {
    let counts = plugin.component_type_counts();
    if counts.is_empty() {
        return "-".to_string();
    }
    // component_type_counts() は固定順序: Skill → Agent → Command → Instruction → Hook
    counts
        .iter()
        .map(|(kind, count)| format!("{} {}", count, kind.plural()))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
#[path = "list_test.rs"]
mod tests;
