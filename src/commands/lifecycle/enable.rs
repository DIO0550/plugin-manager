//! plm enable コマンド
//!
//! プラグインを有効化する。キャッシュからターゲット環境にコンポーネントをデプロイし、
//! `.plm-meta.json` の `statusByTarget` を更新する。

use crate::application::enable_plugin;
use crate::commands::args::MarketplaceArgs;
use crate::commands::lifecycle::toggle::{run_toggle, ToggleArgs, ToggleOp};
use crate::target::TargetKind;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    /// Plugin name (e.g., "owner--repo" or "plugin-name")
    pub name: String,

    /// Target environment to enable (codex, copilot, antigravity, or gemini)
    #[arg(long, value_enum)]
    pub target: Option<TargetKind>,

    #[command(flatten)]
    pub marketplace: MarketplaceArgs,
}

/// # Arguments
///
/// * `args` - Parsed CLI arguments for `plm enable`.
pub async fn run(args: Args) -> Result<(), String> {
    run_toggle(
        ToggleOp::Enable,
        ToggleArgs {
            plugin_name: args.name,
            target: args.target,
            marketplace: args.marketplace,
        },
        |cache, name, marketplace, project_root, target_filter| {
            enable_plugin(cache, name, marketplace, project_root, target_filter)
        },
    )
    .await
}

#[cfg(test)]
#[path = "enable_test.rs"]
mod tests;
