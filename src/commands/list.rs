//! plm list コマンド
//!
//! インストール済みプラグインの一覧を表示する。

use crate::application::{list_installed_plugins, PluginSummary};
use crate::component::ComponentKind;
use crate::target::TargetKind;
use clap::Parser;
use comfy_table::{presets::UTF8_FULL, Table};

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
}

pub async fn run(args: Args) -> Result<(), String> {
    // 1. プラグイン一覧を取得
    let mut plugins = list_installed_plugins().map_err(|e| e.to_string())?;

    let total_count = plugins.len();

    // 2. ソート（name昇順）
    plugins.sort_by(|a, b| a.name.cmp(&b.name));

    // 3. フィルタリング
    let filtered = filter_plugins(plugins, &args);

    // 4. 出力
    if args.json {
        print_json(&filtered)?;
    } else if args.simple {
        print_simple(&filtered, total_count);
    } else {
        print_table(&filtered, total_count);
    }

    Ok(())
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
