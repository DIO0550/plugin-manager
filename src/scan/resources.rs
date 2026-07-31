//! プラグイン直下のリソース列挙（#393）
//!
//! コンポーネント境界（manifest 解決パス等）に該当しないトップレベルエントリを返す。
//! `ComponentKind` には混ぜない。

use crate::path_ext::PathExt;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// プラグインルートからのリソース（トップレベルエントリ）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginResourceEntry {
    /// プラグインルートからの相対名（単一セグメント）
    pub name: String,
    /// 絶対パス
    pub absolute: PathBuf,
}

/// `plugin_root` 直下のうち、除外集合に入らないエントリを列挙する。
///
/// - 走査は **1 階層**。採用したディレクトリの中身は呼び出し側が再帰コピーする。
/// - `excluded_paths` は絶対パス。エントリ自身、またはその子孫が除外対象ならスキップ
///   （例: `hooks/hooks.json` 除外時は親 `hooks/` も除外）。
/// - `excluded_names` はトップレベル名のリテラル除外（VCS 等）。
/// - symlink はスキップ。UTF-8 不可名はスキップ。
///
/// # Arguments
///
/// * `plugin_root` - プラグインルートディレクトリ
/// * `excluded_paths` - コンポーネント境界など絶対パス除外
/// * `excluded_names` - トップレベル名のリテラル除外
pub fn list_plugin_resources(
    plugin_root: &Path,
    excluded_paths: &HashSet<PathBuf>,
    excluded_names: &HashSet<&str>,
) -> Vec<PluginResourceEntry> {
    if !plugin_root.is_dir() {
        return Vec::new();
    }

    let mut out = Vec::new();
    for entry in plugin_root.read_dir_entries() {
        if is_symlink(&entry) {
            continue;
        }
        let Some(name) = entry.file_name().and_then(|n| n.to_str()).map(String::from) else {
            continue;
        };
        if excluded_names.contains(name.as_str()) {
            continue;
        }
        if is_excluded_entry(&entry, excluded_paths) {
            continue;
        }
        out.push(PluginResourceEntry {
            name,
            absolute: entry,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

fn is_symlink(path: &Path) -> bool {
    std::fs::symlink_metadata(path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

fn is_excluded_entry(entry: &Path, excluded_paths: &HashSet<PathBuf>) -> bool {
    for excluded in excluded_paths {
        if entry == excluded {
            return true;
        }
        // 除外対象がエントリ配下（hooks/hooks.json ∈ hooks/）→ 親ごと除外
        if excluded.starts_with(entry) {
            return true;
        }
        // エントリが除外対象配下（通常トップレベルでは起きない）
        if entry.starts_with(excluded) {
            return true;
        }
    }
    false
}

#[cfg(test)]
#[path = "resources_test.rs"]
mod tests;
