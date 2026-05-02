//! 低レベルファイル操作（値オブジェクト）

use super::ScopedPath;
use std::path::PathBuf;

/// 低レベルファイル操作（内部用）
#[derive(Debug, Clone)]
pub enum FileOperation {
    CopyFile { source: PathBuf, target: ScopedPath },
    CopyDir { source: PathBuf, target: ScopedPath },
    RemoveFile { path: ScopedPath },
    RemoveDir { path: ScopedPath },
}

#[cfg(test)]
#[path = "file_operation_test.rs"]
mod tests;
