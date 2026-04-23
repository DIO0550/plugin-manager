//! プラグインのロードとディレクトリクリーンアップ
//!
//! キャッシュからのマニフェスト読み込み (`load_plugin`) と、
//! アンインストール後の空ディレクトリ整理 (`cleanup_plugin_directories`) を提供する。

use crate::fs::{FileSystem, RealFs};
use crate::plugin::{PackageCacheAccess, Plugin};
use crate::target::{PluginOrigin, TargetKind};
use std::path::{Path, PathBuf};

/// キャッシュから Plugin を読み込む
///
/// # Arguments
///
/// * `cache` - package cache access used to read the manifest and path
/// * `marketplace` - marketplace name (`None` defaults to `"github"`)
/// * `plugin_name` - id (cache directory name; e.g. `owner--repo` for GitHub)
pub(crate) fn load_plugin(
    cache: &dyn PackageCacheAccess,
    marketplace: Option<&str>,
    plugin_name: &str,
) -> Result<Plugin, String> {
    let manifest = cache
        .load_manifest(marketplace, plugin_name)
        .map_err(|e| format!("Failed to load manifest: {}", e))?;

    let origin = PluginOrigin::from_cached_plugin(marketplace, plugin_name);
    let plugin_path = cache.plugin_path(marketplace, plugin_name);

    Ok(Plugin::new(manifest, plugin_path, origin))
}

/// プラグインディレクトリをクリーンアップ
///
/// コンポーネント削除後に空になったプラグインディレクトリを削除する。
///
/// # Arguments
///
/// * `kind` - target environment kind determining directory layout
/// * `origin` - plugin origin providing marketplace and plugin segments
/// * `project_root` - project root under which project-scope deploy directories live
///
/// `HOME` 環境変数が未設定の場合、personal scope のクリーンアップは
/// スキップされる（project scope のみ実行）。これにより `HOME` 欠落時に
/// literal `~` がカレント配下に解決され誤削除されるリスクを避ける。
pub(crate) fn cleanup_plugin_directories(
    kind: TargetKind,
    origin: &PluginOrigin,
    project_root: &Path,
) {
    let fs = RealFs;
    // HOME="" や HOME="   " を未設定相当として扱う。
    // そのまま PathBuf::from("") を使うと personal cleanup が `./.codex` 等の
    // CWD 配下パスで走ってしまうため、trim 後に空なら None にフォールバックする。
    let home = std::env::var("HOME")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from);
    cleanup_plugin_directories_impl(&fs, kind, home.as_deref(), origin, project_root);
}

/// 内部実装 — `home` と `fs` を注入可能にし、テストから直接呼ぶ。
///
/// `home` が `None` の場合、personal scope のクリーンアップはスキップされる。
pub(crate) fn cleanup_plugin_directories_impl(
    fs: &dyn FileSystem,
    kind: TargetKind,
    home: Option<&Path>,
    origin: &PluginOrigin,
    project_root: &Path,
) {
    for (base, kind_subdir) in cleanup_specs(kind, home, project_root) {
        cleanup_one(fs, &base, kind_subdir, origin);
    }
}

/// TargetKind ごとに (base_dir, kind_subdir) のリストを返す。
///
/// - `home` が `Some` の場合: personal scope + project scope 両方のエントリを列挙
/// - `home` が `None` の場合: project scope のエントリのみ列挙（personal cleanup スキップ）
///
/// kind_subdir は `"agents"` / `"skills"` / `"prompts"` / `"hooks"` などの
/// コンポーネント種別配下ディレクトリ名。
fn cleanup_specs(
    kind: TargetKind,
    home: Option<&Path>,
    project_root: &Path,
) -> Vec<(PathBuf, &'static str)> {
    let mut specs: Vec<(PathBuf, &'static str)> = Vec::new();

    match kind {
        TargetKind::Codex => {
            if let Some(h) = home {
                specs.push((h.join(".codex"), "agents"));
                specs.push((h.join(".codex"), "skills"));
            }
            specs.push((project_root.join(".codex"), "agents"));
            specs.push((project_root.join(".codex"), "skills"));
        }
        TargetKind::Copilot => {
            if let Some(h) = home {
                // Personal scope: CopilotTarget::can_place は Agent / Hook のみサポート
                specs.push((h.join(".copilot"), "agents"));
                specs.push((h.join(".copilot"), "hooks"));
            }
            // Project scope: 全コンポーネント種別を受け付ける
            specs.push((project_root.join(".github"), "agents"));
            specs.push((project_root.join(".github"), "prompts"));
            specs.push((project_root.join(".github"), "skills"));
            specs.push((project_root.join(".github"), "hooks"));
        }
        TargetKind::Antigravity => {
            if let Some(h) = home {
                specs.push((h.join(".gemini").join("antigravity"), "skills"));
            }
            specs.push((project_root.join(".agent"), "skills"));
        }
        TargetKind::GeminiCli => {
            if let Some(h) = home {
                specs.push((h.join(".gemini"), "skills"));
            }
            specs.push((project_root.join(".gemini"), "skills"));
        }
    }

    specs
}

fn cleanup_one(fs: &dyn FileSystem, base: &Path, kind_subdir: &str, origin: &PluginOrigin) {
    // 防御的検証: 不正な marketplace / plugin セグメントが渡された場合、
    // base の外で remove_dir_all が走ってしまうのを防ぐため cleanup をスキップする。
    if !is_safe_path_segment(&origin.marketplace) || !is_safe_path_segment(&origin.plugin) {
        return;
    }

    let plugin_dir = base
        .join(kind_subdir)
        .join(&origin.marketplace)
        .join(&origin.plugin);
    remove_if_empty(fs, &plugin_dir);

    let marketplace_dir = base.join(kind_subdir).join(&origin.marketplace);
    remove_if_empty(fs, &marketplace_dir);

    let kind_root = base.join(kind_subdir);
    remove_if_empty(fs, &kind_root);
}

/// パスの 1 セグメントとして安全かを判定する。
///
/// `..` / パスセパレータ / 先頭ドット / 絶対パスを拒否し、`base` 外への
/// 書き込みや削除を防ぐ。`plugin/cache.rs::validate_source_path` と同じ方針。
fn is_safe_path_segment(segment: &str) -> bool {
    if segment.is_empty() {
        return false;
    }
    if segment.contains("..") {
        return false;
    }
    if segment.contains('/') || segment.contains('\\') {
        return false;
    }
    if segment.starts_with('.') {
        return false;
    }
    if Path::new(segment).is_absolute() {
        return false;
    }
    true
}

fn remove_if_empty(fs: &dyn FileSystem, path: &Path) {
    if fs.is_dir(path) {
        if let Ok(entries) = fs.read_dir(path) {
            if entries.is_empty() {
                let _ = fs.remove_dir_all(path);
            }
        }
    }
}

#[cfg(test)]
#[path = "loader_test.rs"]
mod tests;
