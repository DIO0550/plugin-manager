//! plm list コマンド
//!
//! インストール済みプラグインの一覧を表示する。

use crate::application::{list_installed_plugins, PluginSummary};
use crate::component::ComponentKind;
use crate::host::{HostClientFactory, HostKind};
use crate::plugin::{
    fetch_remote_versions, meta, needs_update, PluginCache, PluginMeta, VersionQueryResult,
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

/// プラグイン情報と更新チェック結果を組み合わせた出力用構造体
#[derive(Debug, Clone, Serialize)]
struct PluginWithUpdateInfo {
    #[serde(flatten)]
    summary: PluginSummary,
    #[serde(rename = "current_sha")]
    current_sha: Option<String>,
    #[serde(rename = "latest_sha")]
    latest_sha: Option<String>,
    has_update: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    check_error: Option<String>,
}

pub async fn run(args: Args) -> Result<(), String> {
    // 1. プラグイン一覧を取得
    let mut plugins = list_installed_plugins().map_err(|e| e.to_string())?;

    let total_count = plugins.len();

    // 2. ソート（name昇順）
    plugins.sort_by(|a, b| a.name.cmp(&b.name));

    // 3. フィルタリング
    let filtered = filter_plugins(plugins, &args);

    // 4. 出力（--outdated の場合は更新チェックを実行）
    if args.outdated {
        run_outdated_check(&filtered, &args, total_count).await?;
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
async fn run_outdated_check(
    plugins: &[PluginSummary],
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

    // PluginMeta を収集
    let cache = PluginCache::new().map_err(|e| e.to_string())?;
    let mut plugin_metas: Vec<(String, PluginMeta)> = Vec::new();

    for plugin in plugins {
        let plugin_path = cache.plugin_path(plugin.marketplace.as_deref(), &plugin.name);
        let plugin_meta = meta::load_meta(&plugin_path).unwrap_or_default();
        plugin_metas.push((plugin.name.clone(), plugin_meta));
    }

    // GitHub クライアントを作成してリモートバージョンを取得
    let factory = HostClientFactory::with_defaults();
    let client = factory.create(HostKind::GitHub);
    let remote_versions = fetch_remote_versions(&plugin_metas, client.as_ref()).await;

    // PluginSummary と VersionQueryResult を結合
    let combined: Vec<PluginWithUpdateInfo> = plugins
        .iter()
        .zip(plugin_metas.iter())
        .zip(remote_versions.iter())
        .map(|((summary, (_name, meta)), (_plugin_name, result))| {
            let current_sha = meta.commit_sha.clone();
            let (latest_sha, has_update, check_error) = match result {
                VersionQueryResult::Found(remote) => {
                    let has_update = needs_update(current_sha.as_deref(), &remote.sha);
                    (Some(remote.sha.clone()), has_update, None)
                }
                VersionQueryResult::Failed { message } => (None, false, Some(message.clone())),
            };
            PluginWithUpdateInfo {
                summary: summary.clone(),
                current_sha,
                latest_sha,
                has_update,
                check_error,
            }
        })
        .collect();

    // 出力
    if args.json {
        // JSON出力: 全件出力（フィルタなし）
        print_outdated_json(&combined)?;
    } else {
        // テーブル出力: 更新ありのみフィルタ
        print_outdated_table(&combined, total_count);
    }

    Ok(())
}

fn print_outdated_json(plugins: &[PluginWithUpdateInfo]) -> Result<(), String> {
    serde_json::to_string_pretty(plugins)
        .map(|json| println!("{json}"))
        .map_err(|e| format!("Failed to serialize plugins: {}", e))
}

fn print_outdated_table(plugins: &[PluginWithUpdateInfo], total_count: usize) {
    // 更新ありかつエラーなしのプラグインをフィルタ
    let with_updates: Vec<_> = plugins
        .iter()
        .filter(|p| p.has_update && p.check_error.is_none())
        .collect();

    // エラー件数をカウント
    let error_count = plugins.iter().filter(|p| p.check_error.is_some()).count();

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

        for plugin in &with_updates {
            let current_sha = plugin
                .current_sha
                .as_ref()
                .map(|s| truncate_sha(s))
                .unwrap_or_else(|| "unknown".to_string());
            let latest_sha = plugin
                .latest_sha
                .as_ref()
                .map(|s| truncate_sha(s))
                .unwrap_or_else(|| "-".to_string());

            table.add_row(vec![
                plugin.summary.name.as_str(),
                plugin.summary.version.as_str(),
                &current_sha,
                &latest_sha,
            ]);
        }

        println!("{table}");
        println!("{} plugin(s) have updates available", with_updates.len());
    }

    // エラーサマリ
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

fn filter_plugins(plugins: Vec<PluginSummary>, args: &Args) -> Vec<PluginSummary> {
    plugins
        .into_iter()
        .filter(|p| filter_by_type(p, args.component_type.as_ref()))
        .filter(|p| filter_by_target(p, args.target.as_ref()))
        .collect()
}

fn filter_by_type(plugin: &PluginSummary, component_type: Option<&ComponentKind>) -> bool {
    match component_type {
        None => true,
        Some(ComponentKind::Skill) => !plugin.skills.is_empty(),
        Some(ComponentKind::Agent) => !plugin.agents.is_empty(),
        Some(ComponentKind::Command) => !plugin.commands.is_empty(),
        Some(ComponentKind::Instruction) => !plugin.instructions.is_empty(),
        Some(ComponentKind::Hook) => !plugin.hooks.is_empty(),
    }
}

fn filter_by_target(plugin: &PluginSummary, target: Option<&TargetKind>) -> bool {
    // Phase 1: シンプルにenabled状態でフィルタ
    // ターゲット指定時は、そのターゲットで有効なプラグインのみ表示
    // 現状の PluginSummary にはターゲット別のデプロイ情報がないため、
    // enabled = true のプラグインを「ターゲットにデプロイ済み」とみなす
    match target {
        None => true,
        Some(_) => plugin.enabled,
    }
}

fn print_table(plugins: &[PluginSummary], total_count: usize) {
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
        let status = if plugin.enabled {
            "enabled"
        } else {
            "disabled"
        };
        let marketplace = plugin.marketplace.as_deref().unwrap_or("-");
        let components = format_components(plugin);

        table.add_row(vec![
            plugin.name.as_str(),
            plugin.version.as_str(),
            components.as_str(),
            status,
            marketplace,
        ]);
    }

    println!("{table}");
}

fn print_json(plugins: &[PluginSummary]) -> Result<(), String> {
    // 空の場合も [] を出力
    serde_json::to_string_pretty(plugins)
        .map(|json| println!("{json}"))
        .map_err(|e| format!("Failed to serialize plugins: {}", e))
}

fn print_simple(plugins: &[PluginSummary], total_count: usize) {
    if plugins.is_empty() {
        if total_count == 0 {
            println!("No plugins installed");
        } else {
            println!("No plugins matched");
        }
        return;
    }
    for plugin in plugins {
        println!("{}", plugin.name);
    }
}

fn format_components(plugin: &PluginSummary) -> String {
    let counts = plugin.component_type_counts();
    if counts.is_empty() {
        return "-".to_string();
    }
    // component_type_counts() は固定順序: Skill → Agent → Command → Instruction → Hook
    counts
        .iter()
        .map(|c| format!("{} {}", c.count, c.kind.plural()))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
#[path = "list_test.rs"]
mod tests;
