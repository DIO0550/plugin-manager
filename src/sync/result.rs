//! 同期結果の定義

use super::action::SyncAction;
use super::placed::PlacedComponent;

/// 同期結果
#[derive(Debug, Clone, Default)]
pub struct SyncResult {
    /// 作成されたコンポーネント
    pub created: Vec<PlacedComponent>,
    /// 更新されたコンポーネント
    pub updated: Vec<PlacedComponent>,
    /// 削除されたコンポーネント
    pub deleted: Vec<PlacedComponent>,
    /// 変更なしでスキップされたコンポーネント
    pub skipped: Vec<PlacedComponent>,
    /// サポート外でスキップされたコンポーネント
    pub unsupported: Vec<PlacedComponent>,
    /// 失敗したコンポーネント
    pub failed: Vec<SyncFailure>,
    /// dry_run モードだったか
    pub dry_run: bool,
}

impl SyncResult {
    /// dry_run 用の結果を作成
    pub fn dry_run(
        created: Vec<PlacedComponent>,
        updated: Vec<PlacedComponent>,
        deleted: Vec<PlacedComponent>,
        skipped: Vec<PlacedComponent>,
        unsupported: Vec<PlacedComponent>,
    ) -> Self {
        Self {
            created,
            updated,
            deleted,
            skipped,
            unsupported,
            failed: Vec::new(),
            dry_run: true,
        }
    }

    /// 全アイテム数
    pub fn total_count(&self) -> usize {
        self.created.len()
            + self.updated.len()
            + self.deleted.len()
            + self.skipped.len()
            + self.unsupported.len()
            + self.failed.len()
    }

    /// 成功したアクション数（create + update + delete）
    pub fn success_count(&self) -> usize {
        self.created.len() + self.updated.len() + self.deleted.len()
    }

    /// 失敗数
    pub fn failure_count(&self) -> usize {
        self.failed.len()
    }

    /// スキップ数（変更なし + サポート外）
    pub fn skip_count(&self) -> usize {
        self.skipped.len() + self.unsupported.len()
    }

    /// 作成数
    pub fn create_count(&self) -> usize {
        self.created.len()
    }

    /// 更新数
    pub fn update_count(&self) -> usize {
        self.updated.len()
    }

    /// 削除数
    pub fn delete_count(&self) -> usize {
        self.deleted.len()
    }

    /// 結果が空か（何も処理されなかった）
    pub fn is_empty(&self) -> bool {
        self.total_count() == 0
    }

    /// 全て成功したか
    pub fn is_success(&self) -> bool {
        self.failed.is_empty()
    }
}

/// 同期失敗
#[derive(Debug, Clone)]
pub struct SyncFailure {
    /// 失敗したコンポーネント
    pub component: PlacedComponent,
    /// 試行したアクション
    pub action: SyncAction,
    /// エラーメッセージ
    pub error: String,
}

impl SyncFailure {
    pub fn new(component: PlacedComponent, action: SyncAction, error: impl Into<String>) -> Self {
        Self {
            component,
            action,
            error: error.into(),
        }
    }
}

#[cfg(test)]
#[path = "result_test.rs"]
mod tests;
