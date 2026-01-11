//! ターゲット操作の結果
//!
//! ターゲットへの操作（install/uninstall）の結果を表す値オブジェクト。

/// ターゲットへの影響（値オブジェクト）
#[derive(Debug, Clone)]
pub struct TargetEffect {
    target_name: String,
    component_count: usize,
}

impl TargetEffect {
    /// 新規作成
    pub fn new(target_name: impl Into<String>, component_count: usize) -> Self {
        Self {
            target_name: target_name.into(),
            component_count,
        }
    }

    /// ターゲット名を取得
    pub fn target_name(&self) -> &str {
        &self.target_name
    }

    /// コンポーネント数を取得
    pub fn component_count(&self) -> usize {
        self.component_count
    }
}

/// ターゲットエラー（値オブジェクト）
#[derive(Debug, Clone)]
pub struct TargetError {
    target_name: String,
    message: String,
}

impl TargetError {
    /// 新規作成
    pub fn new(target_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            target_name: target_name.into(),
            message: message.into(),
        }
    }

    /// ターゲット名を取得
    pub fn target_name(&self) -> &str {
        &self.target_name
    }

    /// エラーメッセージを取得
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// 影響を受けたターゲット（値オブジェクト）
///
/// 操作結果を記録し、最終的に OperationResult を生成する。
#[derive(Debug, Clone, Default)]
pub struct AffectedTargets {
    effects: Vec<TargetEffect>,
    errors: Vec<TargetError>,
}

impl AffectedTargets {
    /// 新規作成
    pub fn new() -> Self {
        Self::default()
    }

    /// 成功を記録
    pub fn record_success(&mut self, target_name: &str, component_count: usize) {
        if component_count > 0 {
            self.effects.push(TargetEffect::new(target_name, component_count));
        }
    }

    /// エラーを記録
    pub fn record_error(&mut self, target_name: &str, message: impl Into<String>) {
        self.errors.push(TargetError::new(target_name, message));
    }

    /// 総コンポーネント数
    pub fn total_components(&self) -> usize {
        self.effects.iter().map(|e| e.component_count).sum()
    }

    /// ターゲット名一覧
    pub fn target_names(&self) -> Vec<&str> {
        self.effects.iter().map(|e| e.target_name()).collect()
    }

    /// エラーがあるか
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// エラー一覧
    pub fn errors(&self) -> &[TargetError] {
        &self.errors
    }

    /// エラーメッセージを結合
    fn error_message(&self) -> Option<String> {
        if self.errors.is_empty() {
            None
        } else {
            Some(
                self.errors
                    .iter()
                    .map(|e| format!("{}: {}", e.target_name, e.message))
                    .collect::<Vec<_>>()
                    .join("; "),
            )
        }
    }

    /// OperationResult を生成（値オブジェクトがファクトリ）
    pub fn into_result(self) -> OperationResult {
        if self.errors.is_empty() {
            OperationResult {
                success: true,
                error: None,
                affected_targets: self,
            }
        } else {
            let error = self.error_message();
            OperationResult {
                success: false,
                error,
                affected_targets: self,
            }
        }
    }
}

/// 操作結果
#[derive(Debug, Clone)]
pub struct OperationResult {
    /// 成功したか
    pub success: bool,
    /// エラーメッセージ（失敗時）
    pub error: Option<String>,
    /// 影響を受けたターゲット
    pub affected_targets: AffectedTargets,
}

impl OperationResult {
    /// エラー結果を生成（事前検証失敗用）
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            error: Some(message.into()),
            affected_targets: AffectedTargets::new(),
        }
    }
}

#[cfg(test)]
#[path = "effect_test.rs"]
mod tests;
