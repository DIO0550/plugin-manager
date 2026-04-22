//! Target 共通のパス計算ユーティリティ

use crate::component::Scope;
use std::path::{Path, PathBuf};

/// ホームディレクトリを返す。`HOME` 未設定時は `~` フォールバック。
pub(crate) fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("~"))
}

/// Scope に応じた base ディレクトリを返す。
///
/// - `Personal`: `home_dir().join(personal_subdir)`
/// - `Project`: `project_root.join(project_subdir)`
pub(crate) fn base_dir(
    scope: Scope,
    project_root: &Path,
    personal_subdir: &str,
    project_subdir: &str,
) -> PathBuf {
    match scope {
        Scope::Personal => home_dir().join(personal_subdir),
        Scope::Project => project_root.join(project_subdir),
    }
}

#[cfg(test)]
#[path = "paths_test.rs"]
mod tests;
