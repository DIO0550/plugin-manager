use crate::application;
use clap::Parser;
use std::env;

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

    let project_root = env::current_dir().map_err(|e| format!("Failed to get current dir: {}", e))?;
    let result = application::uninstall_plugin(&args.name, args.marketplace.as_deref(), &project_root);

    if result.success {
        println!("Plugin '{}' uninstalled successfully.", args.name);
        let target_names = result.affected_targets.target_names();
        if !target_names.is_empty() {
            println!(
                "  Removed {} component(s) from: {}",
                result.affected_targets.total_components(),
                target_names.join(", ")
            );
        }
        Ok(())
    } else {
        Err(format!(
            "Failed to uninstall: {}",
            result.error.unwrap_or_else(|| "Unknown error".to_string())
        ))
    }
}
