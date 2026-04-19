//! 同期オプションの定義

use crate::component::{ComponentKind, Scope};
use clap::ValueEnum;

/// sync でサポートするコンポーネント種別（CLI 用）
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SyncableKind {
    Skill,
    Agent,
    Command,
    Instruction,
}

impl SyncableKind {
    /// ComponentKind へ変換
    pub fn to_component_kind(self) -> ComponentKind {
        match self {
            SyncableKind::Skill => ComponentKind::Skill,
            SyncableKind::Agent => ComponentKind::Agent,
            SyncableKind::Command => ComponentKind::Command,
            SyncableKind::Instruction => ComponentKind::Instruction,
        }
    }

    /// 全種別を返す
    pub fn all() -> &'static [SyncableKind] {
        &[
            SyncableKind::Skill,
            SyncableKind::Agent,
            SyncableKind::Command,
            SyncableKind::Instruction,
        ]
    }
}

/// 同期オプション
#[derive(Debug, Clone, Default)]
pub struct SyncOptions {
    /// 対象コンポーネント種別（None = 全て）
    pub component_type: Option<SyncableKind>,
    /// 対象スコープ（None = 両方）
    pub scope: Option<Scope>,
    /// true: プレビューのみ、false: 実行
    pub dry_run: bool,
}

impl SyncOptions {
    /// Create a default `SyncOptions` with no filters and `dry_run` disabled.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the component type filter.
    ///
    /// # Arguments
    ///
    /// * `kind` - Component kind to restrict synchronization to.
    pub fn with_component_type(mut self, kind: SyncableKind) -> Self {
        self.component_type = Some(kind);
        self
    }

    /// Set the scope filter.
    ///
    /// # Arguments
    ///
    /// * `scope` - Placement scope (personal or project) to restrict synchronization to.
    pub fn with_scope(mut self, scope: Scope) -> Self {
        self.scope = Some(scope);
        self
    }

    /// Set the dry-run flag.
    ///
    /// # Arguments
    ///
    /// * `dry_run` - When `true`, no file-system mutations are performed.
    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }
}

#[cfg(test)]
#[path = "options_test.rs"]
mod tests;
