//! 同期アクションの定義

/// 同期アクション
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncAction {
    /// 新規作成
    Create,
    /// 更新（上書き）
    Update,
    /// 削除
    Delete,
}

impl SyncAction {
    /// 表示名を取得
    pub fn display_name(&self) -> &'static str {
        match self {
            SyncAction::Create => "Create",
            SyncAction::Update => "Update",
            SyncAction::Delete => "Delete",
        }
    }

    /// アイコンを取得
    pub fn icon(&self) -> &'static str {
        match self {
            SyncAction::Create => "+",
            SyncAction::Update => "~",
            SyncAction::Delete => "-",
        }
    }
}

#[cfg(test)]
#[path = "action_test.rs"]
mod tests;
