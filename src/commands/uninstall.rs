use crate::tui::manager::screens::installed::actions::{uninstall_plugin, ActionResult};
use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    /// プラグイン名（キャッシュディレクトリ名、例: "DIO0550--cc-plugin"）
    pub name: String,

    /// マーケットプレイス（未指定なら "github"）
    #[arg(long)]
    pub marketplace: Option<String>,
}

pub async fn run(args: Args) -> Result<(), String> {
    println!("Uninstalling plugin: {}", args.name);

    match uninstall_plugin(&args.name, args.marketplace.as_deref()) {
        ActionResult::Success => {
            println!("Plugin '{}' uninstalled successfully.", args.name);
            Ok(())
        }
        ActionResult::Error(e) => Err(format!("Failed to uninstall: {}", e)),
    }
}
