//! 低レベルファイル操作（値オブジェクト）

use super::ScopedPath;
use std::path::{Path, PathBuf};

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
#[path = "file_operation_test.rs"]
mod tests;
