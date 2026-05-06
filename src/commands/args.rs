//! サブコマンド間で共有するCLI引数部品。

use crate::target::{Scope, TargetKind};
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

#[derive(Debug, Clone, ClapArgs)]
pub struct SingleTargetArgs {
    /// Filter by target environment (currently filters by enabled status)
    #[arg(long, value_enum)]
    pub target: Option<TargetKind>,
}

#[derive(Debug, Clone, ClapArgs)]
pub struct MultiTargetArgs {
    /// Target environments to deploy to (if not specified, TUI selection)
    #[arg(long, value_enum)]
    pub target: Option<Vec<TargetKind>>,
}

#[derive(Debug, Clone, ClapArgs)]
pub struct InteractiveScopeArgs {
    #[arg(
        long,
        value_enum,
        help = "Deployment scope (if not specified, TUI selection)"
    )]
    pub scope: Option<Scope>,
}

#[derive(Debug, Clone, ClapArgs)]
pub struct SyncScopeArgs {
    #[arg(
        long,
        value_enum,
        help = "Scope to sync (if not specified, both personal and project)"
    )]
    pub scope: Option<Scope>,
}
