//! Target 共通のパス計算ユーティリティ

use crate::component::Scope;
use std::path::{Path, PathBuf};

/// ホームディレクトリを返す。
///
/// `HOME` 環境変数を参照する。以下はすべて「未設定相当」として扱い、
/// **shell 展開されない literal `"~"`** を返すフォールバックにとどめる:
///
/// - `HOME` が未設定
/// - `HOME=""`（空文字）
/// - `HOME="   "`（空白のみ）
///
/// これにより、`HOME=""` が空 `PathBuf` として伝播し personal scope が
/// CWD 配下（例: `./.codex`）に解決される事故を防ぐ（`plugin::loader::
/// cleanup_plugin_directories` の HOME 正規化と同一方針）。
/// ただし literal `"~"` を返すフォールバック自体はこれまでどおり残すので、
/// `HOME` が欠落した環境では呼び出し側が `./~/...` のような相対パスを
/// 作ってしまう可能性がある。`HOME` が確実に設定されている前提で使うこと。
pub(crate) fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("~"))
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
