//! `--scope` オプション用の共通 Args 部品。

use crate::target::Scope;
use clap::Args as ClapArgs;

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
