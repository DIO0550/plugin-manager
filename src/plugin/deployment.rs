//! プラグインデプロイメント支援
//!
//! デプロイ操作に必要な PluginDeployment DTO と、
//! キャッシュ読み込み・ディレクトリクリーンアップのヘルパー関数。

use crate::component::Component;
use crate::fs::{FileSystem, RealFs};
use crate::plugin::{PackageCacheAccess, Plugin, PluginManifest};
use crate::target::PluginOrigin;
use std::path::{Path, PathBuf};

/// デプロイ用プラグイン情報（Application層DTO）
///
/// デプロイ操作に必要な origin, manifest, path を保持。
pub struct PluginDeployment {
    pub origin: PluginOrigin,
    pub manifest: PluginManifest,
    pub path: PathBuf,
}

impl PluginDeployment {
    /// プラグイン内のコンポーネントを取得
    pub fn components(&self) -> Vec<Component> {
        let plugin = Plugin::new(self.manifest.clone(), self.path.clone());
        plugin.components().to_vec()
    }
}

/// キャッシュから PluginDeployment を読み込む
///
/// マニフェストとパス情報を含む PluginDeployment を構築する。
///
/// # Arguments
/// * `cache` - package cache access used to read the manifest and path
/// * `marketplace` - marketplace name (`None` defaults to `"github"`)
/// * `plugin_name` - plugin name or repository identifier
pub(crate) fn load_plugin_deployment(
    cache: &dyn PackageCacheAccess,
    marketplace: Option<&str>,
    plugin_name: &str,
) -> Result<PluginDeployment, String> {
    let manifest = cache
        .load_manifest(marketplace, plugin_name)
        .map_err(|e| format!("Failed to load manifest: {}", e))?;

    let origin = match marketplace {
        Some(mp) => PluginOrigin::from_marketplace(mp, plugin_name),
        None => PluginOrigin::from_marketplace("github", plugin_name),
    };

    let plugin_path = cache.plugin_path(marketplace, plugin_name);

    Ok(PluginDeployment {
        origin,
        manifest,
        path: plugin_path,
    })
}

/// プラグインディレクトリをクリーンアップ
///
/// コンポーネント削除後に空になったプラグインディレクトリを削除する。
///
/// # Arguments
/// * `target_name` - target environment identifier (e.g. `"codex"`, `"copilot"`)
/// * `origin` - plugin origin providing marketplace and plugin segments
/// * `project_root` - project root under which deploy directories live
pub(crate) fn cleanup_plugin_directories(
    target_name: &str,
    origin: &PluginOrigin,
    project_root: &Path,
) {
    let fs = RealFs;

    let dirs_to_check: Vec<(&str, &str)> = match target_name {
        "codex" => vec![("agents", ".codex"), ("skills", ".codex")],
        "copilot" => vec![
            ("agents", ".github"),
            ("prompts", ".github"),
            ("skills", ".github"),
        ],
        _ => vec![],
    };

    for (kind_dir, base_dir) in dirs_to_check {
        let plugin_dir = project_root
            .join(base_dir)
            .join(kind_dir)
            .join(&origin.marketplace)
            .join(&origin.plugin);

        if fs.is_dir(&plugin_dir) {
            if let Ok(entries) = fs.read_dir(&plugin_dir) {
                if entries.is_empty() {
                    let _ = fs.remove_dir_all(&plugin_dir);
                }
            }
        }

        let marketplace_dir = project_root
            .join(base_dir)
            .join(kind_dir)
            .join(&origin.marketplace);

        if fs.is_dir(&marketplace_dir) {
            if let Ok(entries) = fs.read_dir(&marketplace_dir) {
                if entries.is_empty() {
                    let _ = fs.remove_dir_all(&marketplace_dir);
                }
            }
        }

        let kind_dir_path = project_root.join(base_dir).join(kind_dir);

        if fs.is_dir(&kind_dir_path) {
            if let Ok(entries) = fs.read_dir(&kind_dir_path) {
                if entries.is_empty() {
                    let _ = fs.remove_dir_all(&kind_dir_path);
                }
            }
        }
    }
}
