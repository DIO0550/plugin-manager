//! 同期関連の型定義

use crate::component::{ComponentKind, Scope};
use clap::ValueEnum;
use std::path::PathBuf;

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
#[derive(Debug, Clone)]
pub struct SyncOptions {
    pub component_type: Option<SyncableKind>,
    pub scope: Option<Scope>,
}

/// 同期アイテム
#[derive(Debug, Clone)]
pub struct SyncItem {
    pub kind: ComponentKind,
    pub name: String,
    pub scope: Scope,
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub action: SyncAction,
}

/// 同期アクション
#[derive(Debug, Clone, PartialEq)]
pub enum SyncAction {
    Create,
    Overwrite,
    Skip { reason: String },
}

impl SyncAction {
    pub fn skip(reason: impl Into<String>) -> Self {
        SyncAction::Skip {
            reason: reason.into(),
        }
    }

    pub fn is_skip(&self) -> bool {
        matches!(self, SyncAction::Skip { .. })
    }

    pub fn skip_reason(&self) -> Option<&str> {
        match self {
            SyncAction::Skip { reason } => Some(reason),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            SyncAction::Create => "Create",
            SyncAction::Overwrite => "Overwrite",
            SyncAction::Skip { .. } => "Skip",
        }
    }
}

/// 同期計画
#[derive(Debug, Clone)]
pub struct SyncPlan {
    pub from_target: String,
    pub to_target: String,
    pub items: Vec<SyncItem>,
}

impl SyncPlan {
    pub fn actionable_count(&self) -> usize {
        self.items.iter().filter(|i| !i.action.is_skip()).count()
    }

    pub fn skip_count(&self) -> usize {
        self.items.iter().filter(|i| i.action.is_skip()).count()
    }

    pub fn create_count(&self) -> usize {
        self.items
            .iter()
            .filter(|i| matches!(i.action, SyncAction::Create))
            .count()
    }

    pub fn overwrite_count(&self) -> usize {
        self.items
            .iter()
            .filter(|i| matches!(i.action, SyncAction::Overwrite))
            .count()
    }
}

/// 同期結果
#[derive(Debug, Clone, Default)]
pub struct SyncResult {
    pub succeeded: Vec<SyncItem>,
    pub failed: Vec<SyncFailure>,
}

/// 同期失敗
#[derive(Debug, Clone)]
pub struct SyncFailure {
    pub item: SyncItem,
    pub error: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_action_skip_reason() {
        let skip = SyncAction::skip("not supported");
        assert!(skip.is_skip());
        assert_eq!(skip.skip_reason(), Some("not supported"));

        let create = SyncAction::Create;
        assert!(!create.is_skip());
        assert_eq!(create.skip_reason(), None);
    }

    #[test]
    fn test_syncable_kind_all() {
        let all = SyncableKind::all();
        assert_eq!(all.len(), 4);
        assert!(all.contains(&SyncableKind::Skill));
        assert!(all.contains(&SyncableKind::Agent));
        assert!(all.contains(&SyncableKind::Command));
        assert!(all.contains(&SyncableKind::Instruction));
    }

    #[test]
    fn test_sync_plan_actionable_count() {
        let plan = SyncPlan {
            from_target: "codex".to_string(),
            to_target: "copilot".to_string(),
            items: vec![
                SyncItem {
                    kind: ComponentKind::Skill,
                    name: "skill1".to_string(),
                    scope: Scope::Personal,
                    source_path: PathBuf::from("/src/skill1"),
                    target_path: PathBuf::from("/dst/skill1"),
                    action: SyncAction::Create,
                },
                SyncItem {
                    kind: ComponentKind::Agent,
                    name: "agent1".to_string(),
                    scope: Scope::Personal,
                    source_path: PathBuf::from("/src/agent1"),
                    target_path: PathBuf::from("/dst/agent1"),
                    action: SyncAction::Overwrite,
                },
                SyncItem {
                    kind: ComponentKind::Command,
                    name: "cmd1".to_string(),
                    scope: Scope::Personal,
                    source_path: PathBuf::from("/src/cmd1"),
                    target_path: PathBuf::from("/dst/cmd1"),
                    action: SyncAction::skip("not supported"),
                },
            ],
        };

        assert_eq!(plan.actionable_count(), 2);
        assert_eq!(plan.skip_count(), 1);
        assert_eq!(plan.create_count(), 1);
        assert_eq!(plan.overwrite_count(), 1);
    }
}
