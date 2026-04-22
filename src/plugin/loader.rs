//! プラグインのロードとディレクトリクリーンアップ
//!
//! キャッシュからのマニフェスト読み込み (`load_plugin`) と、
//! アンインストール後の空ディレクトリ整理 (`cleanup_plugin_directories`) を提供する。

use crate::fs::{FileSystem, RealFs};
use crate::plugin::{PackageCacheAccess, Plugin};
use crate::target::paths::home_dir;
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

    let origin = match marketplace {
        Some(mp) => PluginOrigin::from_marketplace(mp, plugin_name),
        None => PluginOrigin::from_marketplace("github", plugin_name),
    };

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
pub(crate) fn cleanup_plugin_directories(
    kind: TargetKind,
    origin: &PluginOrigin,
    project_root: &Path,
) {
    let fs = RealFs;
    let home = home_dir();
    cleanup_plugin_directories_impl(&fs, kind, &home, origin, project_root);
}

/// 内部実装 — `home` と `fs` を注入可能にし、テストから直接呼ぶ。
pub(crate) fn cleanup_plugin_directories_impl(
    fs: &dyn FileSystem,
    kind: TargetKind,
    home: &Path,
    origin: &PluginOrigin,
    project_root: &Path,
) {
    for (base, kind_subdir) in cleanup_specs(kind, home, project_root) {
        cleanup_one(fs, &base, kind_subdir, origin);
    }
}

/// TargetKind ごとに (base_dir, kind_subdir) のリストを返す。
///
/// base_dir は Personal / Project それぞれのディレクトリ、
/// kind_subdir は `"agents"` / `"skills"` / `"prompts"` / `"hooks"` などの
/// コンポーネント種別配下ディレクトリ名。
fn cleanup_specs(
    kind: TargetKind,
    home: &Path,
    project_root: &Path,
) -> Vec<(PathBuf, &'static str)> {
    match kind {
        TargetKind::Codex => vec![
            (home.join(".codex"), "agents"),
            (home.join(".codex"), "skills"),
            (project_root.join(".codex"), "agents"),
            (project_root.join(".codex"), "skills"),
        ],
        TargetKind::Copilot => vec![
            // Personal scope: CopilotTarget::can_place は Agent / Hook のみサポート
            (home.join(".copilot"), "agents"),
            (home.join(".copilot"), "hooks"),
            // Project scope: 全コンポーネント種別を受け付ける
            (project_root.join(".github"), "agents"),
            (project_root.join(".github"), "prompts"),
            (project_root.join(".github"), "skills"),
            (project_root.join(".github"), "hooks"),
        ],
        TargetKind::Antigravity => vec![
            (home.join(".gemini").join("antigravity"), "skills"),
            (project_root.join(".agent"), "skills"),
        ],
        TargetKind::GeminiCli => vec![
            (home.join(".gemini"), "skills"),
            (project_root.join(".gemini"), "skills"),
        ],
    }
}

fn cleanup_one(fs: &dyn FileSystem, base: &Path, kind_subdir: &str, origin: &PluginOrigin) {
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
