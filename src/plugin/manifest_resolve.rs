//! マニフェストファイルのパス解決
//!
//! プラグインディレクトリ内の `plugin.json` の検索と解決を行う。

use std::path::{Path, PathBuf};

/// マニフェストファイルのパス候補（優先順）
const MANIFEST_PATHS: &[&str] = &[".claude-plugin/plugin.json", "plugin.json"];

/// プラグインディレクトリ内のマニフェストパスを解決する
///
/// 以下の順序でマニフェストを検索:
/// 1. `.claude-plugin/plugin.json` (推奨)
/// 2. `plugin.json` (フォールバック)
///
/// # Arguments
/// * `plugin_dir` - プラグインのルートディレクトリ
///
/// # Returns
/// マニフェストファイルのパス、見つからない場合は None
///
/// # Visibility
/// Infrastructure内部関数。外部（TUI/CLI）からは直接呼ばず、
/// `PluginCache::load_manifest()` や `has_manifest()` を経由して使用する。
pub(crate) fn resolve_manifest_path(plugin_dir: &Path) -> Option<PathBuf> {
    for candidate in MANIFEST_PATHS {
        let path = plugin_dir.join(candidate);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

/// プラグインディレクトリがマニフェストを持つか確認する
pub fn has_manifest(plugin_dir: &Path) -> bool {
    resolve_manifest_path(plugin_dir).is_some()
}

#[cfg(test)]
#[path = "manifest_resolve_test.rs"]
mod tests;
