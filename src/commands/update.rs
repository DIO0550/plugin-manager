//! plm update コマンド
//!
//! プラグインを最新バージョンに更新する。

use crate::plugin::{update_all_plugins, update_plugin, PluginCache, UpdateResult, UpdateStatus};
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

pub async fn run(args: Args) -> Result<(), String> {
    // 排他チェック（どちらも未指定の場合）
    if args.name.is_none() && !args.all {
        return Err("Specify plugin name or --all".to_string());
    }

    let cache = PluginCache::new().map_err(|e| format!("Failed to access cache: {}", e))?;
    let project_root = env::current_dir().unwrap_or_else(|_| ".".into());
    let target_filter = args.target.as_ref().map(|t| t.as_str());

    if args.all {
        let results = update_all_plugins(&cache, &project_root, target_filter).await;
        display_batch_results(&results);

        // 全失敗時のみエラー終了
        if !results.is_empty()
            && results
                .iter()
                .all(|r| matches!(r.status, UpdateStatus::Failed))
        {
            return Err("All updates failed".to_string());
        }
    } else if let Some(name) = &args.name {
        let result = update_plugin(&cache, name, &project_root, target_filter).await;
        display_single_result(&result);

        if matches!(result.status, UpdateStatus::Failed) {
            return Err(result.error.unwrap_or_default());
        }
    }

    Ok(())
}

fn display_single_result(result: &UpdateResult) {
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
