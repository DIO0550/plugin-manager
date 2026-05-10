//! `--target` オプション用の共通 Args 部品。

use crate::target::TargetKind;
use clap::Args as ClapArgs;

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
