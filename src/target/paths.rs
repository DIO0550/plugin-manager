//! Target 共通のパス計算ユーティリティ

use crate::component::Scope;
use std::path::{Path, PathBuf};

/// ホームディレクトリを返す。
///
/// `HOME` 環境変数を参照する。未設定時は **shell 展開されない literal `"~"`**
/// を返すフォールバックにとどめる（リファクタ前の各 target 実装と同一挙動）。
/// つまり `HOME` が欠落した環境では呼び出し側が `./~/...` のような相対パスを
/// 作ってしまう可能性があるため、`HOME` が確実に設定されている前提で使うこと。
/// 将来的に `HOME` 欠落時のハンドリングを見直す場合は別途方針を定める。
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
