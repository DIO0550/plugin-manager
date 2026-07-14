//! plm disable コマンド
//!
//! プラグインを無効化する。ターゲット環境からコンポーネントを削除し（キャッシュは残す）、
//! `.plm-meta.json` の `statusByTarget` を更新する。

use crate::application::disable_plugin;
use crate::commands::args::MarketplaceArgs;
use crate::commands::lifecycle::toggle::{run_toggle, ToggleArgs, ToggleOp};
use crate::target::TargetKind;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    /// Plugin name (e.g., "owner--repo" or "plugin-name")
    pub name: String,

    /// Target environment to disable (codex, copilot, antigravity, or gemini)
    #[arg(long, value_enum)]
    pub target: Option<TargetKind>,

    #[command(flatten)]
    pub marketplace: MarketplaceArgs,
}

/// # Arguments
///
/// * `args` - Parsed CLI arguments for `plm disable`.
pub async fn run(args: Args) -> Result<(), String> {
    // Cache is required to identify components to remove from the manifest.
    // A "Hint:" is appended to the error when the cache is not found.
    run_toggle(
        ToggleOp::Disable,
        ToggleArgs {
            plugin_name: args.name,
            target: args.target,
            marketplace: args.marketplace,
        },
        |cache, name, marketplace, project_root, target_filter| {
            disable_plugin(cache, name, marketplace, project_root, target_filter)
        },
    )
    .await
    .map_err(|e| {
        // disable に固有のヒントメッセージを追記する
        if e.contains("not found in cache") {
            format!(
                "{}\nHint: Cache is required to identify components to remove.",
                e
            )
        } else {
            e
        }
    })
}

#[cfg(test)]
#[path = "disable_test.rs"]
mod tests;
