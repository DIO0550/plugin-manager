//! Plugin 直下の付属リソース検出（#393）
//!
//! プラグインルート 1 階層を走査し、Component / 予約エントリ以外を列挙する。
//! `ComponentKind` には混ぜず、呼び出し側が除外集合を渡す。

use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// プラグインルート直下の付属エントリ（ファイルまたはディレクトリ）。
///
/// `relative` はプラグインルートからの相対パス（通常は 1 セグメント）。
/// ディレクトリの場合、配置時に配下を再帰コピーする。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachedEntry {
    pub relative: PathBuf,
}

impl AttachedEntry {
    pub fn new(relative: impl Into<PathBuf>) -> Self {
        Self {
            relative: relative.into(),
        }
    }

    pub fn name(&self) -> Option<&OsStr> {
        self.relative.file_name()
    }
}

/// プラグインルート直下の付属リソースを列挙する。
///
/// # Arguments
///
/// * `plugin_root` - プラグインルート
/// * `excluded_rel_paths` - 除外する相対パス（マニフェスト解決済みコンポーネント dir 等）
///
/// # Behavior
///
/// - 走査は **1 階層のみ**。採用したディレクトリはその内容ごと対象（再帰コピーは配置側）
/// - symlink はスキップ
/// - `excluded_rel_paths` に一致するエントリ、および除外パスの祖先にあたるトップレベル
///   エントリはスキップ（カスタム `skills: "lib/skills"` で `lib/` 全体を誤同梱しない）
/// - 固定除外・プレフィックス除外は呼び出し側が `excluded_rel_paths` に含めるか、
///   [`is_attached_name_excluded`] で判定する
pub fn list_plugin_attached_resources(
    plugin_root: &Path,
    excluded_rel_paths: &HashSet<PathBuf>,
) -> Vec<AttachedEntry> {
    if !plugin_root.is_dir() {
        return Vec::new();
    }

    let Ok(read_dir) = std::fs::read_dir(plugin_root) else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for entry in read_dir.flatten() {
        let path = entry.path();
        let Ok(meta) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if meta.file_type().is_symlink() {
            continue;
        }

        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        let relative = PathBuf::from(&name);

        if is_attached_name_excluded(&name) {
            continue;
        }
        if is_path_excluded(&relative, excluded_rel_paths) {
            continue;
        }

        out.push(AttachedEntry::new(relative));
    }

    out.sort_by(|a, b| a.relative.cmp(&b.relative));
    out
}

/// 固定除外・プレフィックス除外に該当するか。
pub fn is_attached_name_excluded(name: &str) -> bool {
    use crate::placement_names::{
        ALL_INSTRUCTION_FILENAMES, ATTACHED_EXACT_EXCLUSIONS, ATTACHED_PREFIX_EXCLUSIONS,
    };

    if ATTACHED_EXACT_EXCLUSIONS
        .iter()
        .any(|ex| name.eq_ignore_ascii_case(ex))
    {
        return true;
    }
    if ALL_INSTRUCTION_FILENAMES
        .iter()
        .any(|ex| name.eq_ignore_ascii_case(ex))
    {
        return true;
    }
    let upper = name.to_ascii_uppercase();
    ATTACHED_PREFIX_EXCLUSIONS.iter().any(|prefix| {
        upper == *prefix
            || upper.starts_with(&format!("{prefix}."))
            || upper.starts_with(&format!("{prefix}-"))
            || upper.starts_with(&format!("{prefix}_"))
    })
}

/// `relative` が除外集合に含まれるか、除外パスの祖先か。
fn is_path_excluded(relative: &Path, excluded: &HashSet<PathBuf>) -> bool {
    if excluded.contains(relative) {
        return true;
    }
    // 大文字小文字を無視した一致
    for ex in excluded {
        if paths_eq_ignore_ascii_case(relative, ex) {
            return true;
        }
        // relative が除外パスの祖先（例: relative=lib, ex=lib/skills）
        if ex.starts_with(relative) && ex != relative {
            return true;
        }
    }
    false
}

fn paths_eq_ignore_ascii_case(a: &Path, b: &Path) -> bool {
    let a = a.to_string_lossy();
    let b = b.to_string_lossy();
    a.eq_ignore_ascii_case(&b)
}

#[cfg(test)]
#[path = "attached_test.rs"]
mod tests;
