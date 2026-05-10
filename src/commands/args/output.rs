//! サブコマンド間で共有するCLI引数部品。

use clap::Args as ClapArgs;

#[derive(Debug, Clone, ClapArgs)]
pub struct ListOutputArgs {
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
