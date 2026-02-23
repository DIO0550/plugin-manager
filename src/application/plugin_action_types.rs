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
        let canonical_root = project_root.canonicalize().map_err(|e| {
            PlmError::Validation(format!(
                "Failed to canonicalize project root '{}': {}",
                project_root.display(),
                e
            ))
        })?;

        let check_path = if path.exists() {
            path.canonicalize().map_err(|e| {
                PlmError::Validation(format!(
                    "Failed to canonicalize path '{}': {}",
                    path.display(),
                    e
                ))
            })?
        } else {
            resolve_nonexistent_path(&path)?
        };

        if !check_path.starts_with(&canonical_root) {
            return Err(PlmError::Validation(format!(
                "Path '{}' is not under project root '{}'",
                path.display(),
                project_root.display()
            )));
        }

        Ok(Self { path: check_path })
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

/// `..` や `.` を論理的に正規化する（ファイルシステムを参照しない）
pub(crate) fn normalize_path(path: &Path) -> PathBuf {
    use std::path::Component;

    let mut normalized = PathBuf::new();
    let mut has_physical_root = false;
    let mut normal_depth: usize = 0;

    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => {
                normalized.push(component.as_os_str());
                has_physical_root = true;
            }
            Component::CurDir => {}
            Component::Normal(_) => {
                normalized.push(component.as_os_str());
                normal_depth += 1;
            }
            Component::ParentDir => {
                if normal_depth > 0 {
                    if normalized.pop() {
                        normal_depth -= 1;
                    }
                } else if !has_physical_root {
                    // 相対パスの先頭付近では `..` を保持する
                    normalized.push(component.as_os_str());
                }
                // ルートを越える `..` は無視する
            }
        }
    }
    normalized
}

/// 存在しないパスを、既存の祖先ディレクトリを基点に正規化する
///
/// `symlink_metadata` を使用してダングリングシンボリックリンクを検出し、
/// 発見した場合はエラーを返す（fail closed）。
fn resolve_nonexistent_path(path: &Path) -> Result<PathBuf> {
    // パス自体がダングリングシンボリックリンクかチェック
    if let Ok(meta) = std::fs::symlink_metadata(path) {
        if meta.is_symlink() {
            return Err(PlmError::Validation(format!(
                "Dangling symlink detected at '{}'",
                path.display()
            )));
        }
    }

    let mut ancestors = path.ancestors();
    let _ = ancestors.next(); // 自分自身をスキップ

    for ancestor in ancestors {
        match std::fs::symlink_metadata(ancestor) {
            Ok(meta) => {
                // ダングリングシンボリックリンクを検出
                if meta.is_symlink() && !ancestor.exists() {
                    return Err(PlmError::Validation(format!(
                        "Dangling symlink detected at '{}' in path '{}'",
                        ancestor.display(),
                        path.display()
                    )));
                }

                // 既存の祖先を正規化（IO/権限エラーは伝播）
                let canonical_ancestor = ancestor.canonicalize().map_err(|e| {
                    PlmError::Validation(format!(
                        "Failed to canonicalize ancestor '{}': {}",
                        ancestor.display(),
                        e
                    ))
                })?;

                if let Ok(relative) = path.strip_prefix(ancestor) {
                    let mut full = canonical_ancestor;
                    full.push(relative);
                    return Ok(normalize_path(&full));
                }
            }
            Err(_) => continue, // ファイルシステム上に存在しない
        }
    }

    Ok(normalize_path(path))
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
