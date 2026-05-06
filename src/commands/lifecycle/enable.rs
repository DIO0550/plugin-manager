//! plm enable コマンド
//!
//! プラグインを有効化する。キャッシュからターゲット環境にコンポーネントをデプロイし、
//! `.plm-meta.json` の `statusByTarget` を更新する。

use crate::application::{enable_plugin, OperationOutcome};
use crate::plugin::{meta, PackageCache, PackageCacheAccess};
use clap::{Parser, ValueEnum};
use std::env;

#[derive(Debug, Clone, ValueEnum)]
pub enum TargetKind {
    Codex,
    Copilot,
}

impl TargetKind {
    fn as_str(&self) -> &'static str {
        match self {
            TargetKind::Codex => "codex",
            TargetKind::Copilot => "copilot",
        }
    }
}

#[derive(Debug, Parser)]
pub struct Args {
    /// Plugin name (e.g., "owner--repo" or "plugin-name")
    pub name: String,

    /// Target environment to enable (codex or copilot)
    #[arg(long, value_enum)]
    pub target: Option<TargetKind>,

    /// Marketplace name (default: github)
    #[arg(long, short = 'm', default_value = "github")]
    pub marketplace: String,
}

/// # Arguments
///
/// * `args` - Parsed CLI arguments for `plm enable`.
pub async fn run(args: Args) -> Result<(), String> {
    let cache = PackageCache::new().map_err(|e| format!("Failed to access cache: {}", e))?;

    if !cache.is_cached(Some(&args.marketplace), &args.name) {
        return Err(format!(
            "Error: Plugin '{}' not found in cache (marketplace: {})",
            args.name, args.marketplace
        ));
    }

    let project_root = env::current_dir().unwrap_or_else(|_| ".".into());
    let target_filter = args.target.as_ref().map(|t| t.as_str());

    let result = enable_plugin(
        &cache,
        &args.name,
        Some(&args.marketplace),
        &project_root,
        target_filter,
    );

    let plugin_path = cache.plugin_path(Some(&args.marketplace), &args.name);
    update_status_after_enable(&plugin_path, &result);

    if result.success {
        display_result(&args.name, &result, target_filter);
        Ok(())
    } else {
        let successful_targets = result.affected_targets.target_names();
        if !successful_targets.is_empty() {
            println!(
                "  Partially enabled: {} target(s) succeeded",
                successful_targets.len()
            );
        }
        if let Some(error) = &result.error {
            Err(format!(
                "Error: Failed to enable plugin '{}': {}",
                args.name, error
            ))
        } else {
            Err(format!("Error: Failed to enable plugin '{}'", args.name))
        }
    }
}

/// enable 後のステータス更新
///
/// # Arguments
///
/// * `plugin_path` - Filesystem path of the cached plugin.
/// * `result` - Outcome returned by `enable_plugin`.
fn update_status_after_enable(plugin_path: &std::path::Path, result: &OperationOutcome) {
    let mut plugin_meta = meta::load_meta(plugin_path).unwrap_or_default();

    let target_names = result.affected_targets.target_names();
    for target_name in target_names {
        plugin_meta.set_status(target_name, "enabled");
    }

    // When a filter is given and the target is unsupported, it is not present
    // in `affected_targets`, so nothing needs to be updated for it.

    if let Err(e) = meta::write_meta(plugin_path, &plugin_meta) {
        eprintln!("Warning: Failed to update .plm-meta.json: {}", e);
    }
}

/// 成功時の結果を表示
///
/// # Arguments
///
/// * `plugin_name` - Plugin identifier shown in the output.
/// * `result` - Outcome returned by `enable_plugin`.
/// * `target_filter` - Optional target name filter that was requested.
fn display_result(plugin_name: &str, result: &OperationOutcome, target_filter: Option<&str>) {
    let targets = result.affected_targets.target_names();
    if targets.is_empty() {
        if let Some(filter) = target_filter {
            println!(
                "Skipped: Plugin '{}' has no components for target '{}'",
                plugin_name, filter
            );
        } else {
            println!("Enabled: Plugin '{}' (no components deployed)", plugin_name);
        }
    } else {
        let target_list = targets.join(", ");
        let component_count = result.affected_targets.total_components();
        println!(
            "Enabled: Plugin '{}' ({} component(s) deployed to {})",
            plugin_name, component_count, target_list
        );
    }
}

#[cfg(test)]
#[path = "enable_test.rs"]
mod tests;
