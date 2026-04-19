//! plm list コマンド
//!
//! インストール済みプラグインの一覧を表示する。

mod json;
mod outdated;
mod simple;
mod table;
mod wire;

use crate::application::list_installed_plugins;
use crate::component::ComponentKind;
use crate::plugin::{InstalledPlugin, PackageCache};
use crate::target::TargetKind;
use clap::Parser;

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

/// # Arguments
///
/// * `args` - Parsed CLI arguments for `plm list`.
pub async fn run(args: Args) -> Result<(), String> {
    let cache = PackageCache::new().map_err(|e| format!("Failed to access cache: {e}"))?;
    let mut plugins = list_installed_plugins(&cache)
        .map_err(|e| format!("Failed to list installed plugins: {e}"))?;

    let total_count = plugins.len();

    plugins.sort_by(|a, b| a.name().cmp(b.name()));

    let filtered = filter_plugins(plugins, &args);

    if args.outdated {
        outdated::run_outdated(&cache, &filtered, args.json, total_count).await?;
    } else if args.json {
        json::print_json(&filtered)?;
    } else if args.simple {
        simple::print_simple(&filtered, total_count);
    } else {
        table::print_table(&filtered, total_count);
    }

    Ok(())
}

/// 空のプラグイン一覧表示時のメッセージを出力する
///
/// simple / table / outdated の各フォーマットで共通利用。
///
/// # Arguments
///
/// * `total_count` - Total number of installed plugins before filtering.
pub(super) fn print_empty_list(total_count: usize) {
    let msg = match total_count {
        0 => "No plugins installed",
        _ => "No plugins matched",
    };
    println!("{msg}");
}

/// # Arguments
///
/// * `plugins` - Installed plugins to filter.
/// * `args` - CLI arguments containing the filter criteria.
fn filter_plugins(plugins: Vec<InstalledPlugin>, args: &Args) -> Vec<InstalledPlugin> {
    plugins
        .into_iter()
        .filter(|p| filter_by_type(p, args.component_type.as_ref()))
        .filter(|p| filter_by_target(p, args.target.as_ref()))
        .collect()
}

/// # Arguments
///
/// * `plugin` - Plugin being checked against the filter.
/// * `component_type` - Requested component kind, or `None` to allow any.
fn filter_by_type(plugin: &InstalledPlugin, component_type: Option<&ComponentKind>) -> bool {
    match component_type {
        None => true,
        Some(kind) => plugin.components().iter().any(|c| c.kind == *kind),
    }
}

/// # Arguments
///
/// * `plugin` - Plugin being checked against the filter.
/// * `target` - Requested target kind, or `None` to allow any.
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

#[cfg(test)]
#[path = "list/list_test.rs"]
mod tests;
