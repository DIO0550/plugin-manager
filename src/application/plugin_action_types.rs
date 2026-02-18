//! プラグインアクション型定義
//!
//! ターゲット識別子、スコープ付きパス、ファイル操作の値オブジェクト群。

use crate::error::{PlmError, Result};
use std::path::{Path, PathBuf};

/// ターゲット識別子（型安全）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TargetId(String);

impl TargetId {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for TargetId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl std::fmt::Display for TargetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// スコープ付きパス（project_root配下であることを保証）
///
/// ディレクトリトラバーサル攻撃を防ぐため、
/// パスが指定されたルート配下にあることを型レベルで保証する。
#[derive(Debug, Clone)]
pub struct ScopedPath {
    path: PathBuf,
}

impl ScopedPath {
    /// 検証して生成
    ///
    /// # Errors
    /// - パスが project_root 配下でない場合
    pub fn new(path: PathBuf, project_root: &Path) -> Result<Self> {
        // 絶対パスに変換して比較
        let canonical_root = project_root
            .canonicalize()
            .unwrap_or_else(|_| project_root.to_path_buf());

        // パスがproject_root配下かチェック（パスが存在しない場合は親ディレクトリで判断）
        let check_path = if path.exists() {
            path.canonicalize().unwrap_or_else(|_| path.clone())
        } else {
            // 存在しないパスの場合、親が存在するか確認
            let mut ancestors = path.ancestors();
            let _ = ancestors.next(); // 自分自身をスキップ
            ancestors
                .find(|p| p.exists())
                .and_then(|p| p.canonicalize().ok())
                .unwrap_or_else(|| path.clone())
        };

        // project_root配下であることを確認
        if !check_path.starts_with(&canonical_root) && !path.starts_with(project_root) {
            return Err(PlmError::Validation(format!(
                "Path '{}' is not under project root '{}'",
                path.display(),
                project_root.display()
            )));
        }

        Ok(Self { path })
    }

    /// パスを取得
    pub fn as_path(&self) -> &Path {
        &self.path
    }

    /// PathBuf に変換
    pub fn into_path(self) -> PathBuf {
        self.path
    }
}

/// 低レベルファイル操作（内部用）
#[derive(Debug, Clone)]
pub enum FileOperation {
    CopyFile { source: PathBuf, target: ScopedPath },
    CopyDir { source: PathBuf, target: ScopedPath },
    RemoveFile { path: ScopedPath },
    RemoveDir { path: ScopedPath },
}

impl FileOperation {
    /// 操作の種類を文字列で取得
    pub fn kind(&self) -> &'static str {
        match self {
            FileOperation::CopyFile { .. } => "copy_file",
            FileOperation::CopyDir { .. } => "copy_dir",
            FileOperation::RemoveFile { .. } => "remove_file",
            FileOperation::RemoveDir { .. } => "remove_dir",
        }
    }

    /// ターゲットパスを取得
    pub fn target_path(&self) -> &Path {
        match self {
            FileOperation::CopyFile { target, .. } | FileOperation::CopyDir { target, .. } => {
                target.as_path()
            }
            FileOperation::RemoveFile { path } | FileOperation::RemoveDir { path } => {
                path.as_path()
            }
        }
    }
}

#[cfg(test)]
#[path = "plugin_action_types_test.rs"]
mod tests;
