//! plm update コマンド
//!
//! プラグインを最新バージョンに更新する。

use crate::plugin::{
    update_all_plugins, update_plugin, PackageCache, UpdateOutcome, UpdateResult, UpdateStatus,
};
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
    /// Plugin name to update
    #[arg(conflicts_with = "all")]
    pub name: Option<String>,

    /// Update all installed plugins
    #[arg(long, conflicts_with = "name")]
    pub all: bool,

    /// Target environment filter (codex or copilot)
    #[arg(long, value_enum)]
    pub target: Option<TargetKind>,
}

/// # Arguments
///
/// * `args` - Parsed CLI arguments for `plm update`.
pub async fn run(args: Args) -> Result<(), String> {
    if args.name.is_none() && !args.all {
        return Err("Specify plugin name or --all".to_string());
    }

    let cache = PackageCache::new().map_err(|e| format!("Failed to access cache: {}", e))?;
    let project_root = env::current_dir().unwrap_or_else(|_| ".".into());
    let target_filter = args.target.as_ref().map(|t| t.as_str());

    if args.all {
        let results = update_all_plugins(&cache, &project_root, target_filter).await;
        display_batch_results(&results);

        // Exit with an error only when every plugin failed.
        if !results.is_empty()
            && results
                .iter()
                .all(|r| matches!(r.status, UpdateStatus::Failed))
        {
            return Err("All updates failed".to_string());
        }
    } else if let Some(name) = &args.name {
        let (plugin_input, marketplace_hint) = match name.split_once('@') {
            Some((p, m)) if !p.is_empty() && !m.is_empty() => (p, Some(m)),
            _ => (name.as_str(), None),
        };
        let result = update_plugin(
            &cache,
            plugin_input,
            marketplace_hint,
            &project_root,
            target_filter,
        )
        .await;
        display_single_result(&result);

        if matches!(result.status, UpdateStatus::Failed) {
            return Err(result.error.unwrap_or_default());
        }
    }

    Ok(())
}

/// # Arguments
///
/// * `result` - Single-plugin update outcome to render.
fn display_single_result(result: &UpdateOutcome) {
    match &result.status {
        UpdateStatus::Updated { from_sha, to_sha } => {
            let from = from_sha.as_deref().unwrap_or("unknown");
            println!("Updated: {} ({} -> {})", result.plugin_name, from, to_sha);
            for target in &result.deployed_targets {
                println!("  - Deployed to {}", target);
            }
            for target in &result.failed_targets {
                eprintln!(
                    "Warning: Failed to deploy to {} (marked as disabled)",
                    target
                );
            }
        }
        UpdateStatus::AlreadyUpToDate => {
            println!("{}: Already up to date", result.plugin_name);
        }
        UpdateStatus::Skipped { reason } => {
            println!("{}: Skipped ({})", result.plugin_name, reason);
        }
        UpdateStatus::Failed => {
            eprintln!("Error: Failed to update '{}'", result.plugin_name);
            if let Some(e) = &result.error {
                eprintln!("  {}", e);
            }
            eprintln!("  Previous version retained.");
        }
    }
}

/// # Arguments
///
/// * `results` - Update outcomes from a batch run to summarize.
fn display_batch_results(results: &[UpdateResult]) {
    let updated = results
        .iter()
        .filter(|r| matches!(r.status, UpdateStatus::Updated { .. }))
        .count();
    let up_to_date = results
        .iter()
        .filter(|r| matches!(r.status, UpdateStatus::AlreadyUpToDate))
        .count();
    let skipped = results
        .iter()
        .filter(|r| matches!(r.status, UpdateStatus::Skipped { .. }))
        .count();
    let failed = results
        .iter()
        .filter(|r| matches!(r.status, UpdateStatus::Failed))
        .count();

    println!("\nSummary:");
    println!("  Updated: {}", updated);
    println!("  Up to date: {}", up_to_date);
    if skipped > 0 {
        println!("  Skipped: {}", skipped);
    }
    if failed > 0 {
        println!("  Failed: {}", failed);
    }
}
