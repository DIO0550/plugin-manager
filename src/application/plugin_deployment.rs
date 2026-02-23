//! プラグインデプロイメント支援
//!
//! デプロイ操作に必要な PluginDeployment DTO と、
//! キャッシュ読み込み・ディレクトリクリーンアップのヘルパー関数。

use crate::component::Component;
use crate::fs::{FileSystem, RealFs};
use crate::plugin::{CachedPlugin, PluginCache, PluginManifest};
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
    ///
    /// CachedPlugin の components() メソッドを再利用する。
    pub fn components(&self) -> Vec<Component> {
        // CachedPlugin を構築してコンポーネントスキャンを委譲
        let cached = CachedPlugin {
            name: self.origin.plugin.clone(),
            marketplace: Some(self.origin.marketplace.clone()),
            path: self.path.clone(),
            manifest: self.manifest.clone(),
            git_ref: String::new(),
            commit_sha: String::new(),
        };
        cached.components()
    }
}

/// キャッシュから PluginDeployment を読み込む
///
/// マニフェストとパス情報を含む PluginDeployment を構築する。
pub(super) fn load_plugin_deployment(
    cache: &PluginCache,
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
pub(super) fn cleanup_plugin_directories(
    target_name: &str,
    origin: &PluginOrigin,
    project_root: &Path,
) {
    let fs = RealFs;

    // ターゲットごとのディレクトリ構造
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
        // プラグインディレクトリのパス: <base>/<kind>/<marketplace>/<plugin>/
        let plugin_dir = project_root
            .join(base_dir)
            .join(kind_dir)
            .join(&origin.marketplace)
            .join(&origin.plugin);

        // ディレクトリが存在して空なら削除
        if fs.is_dir(&plugin_dir) {
            if let Ok(entries) = fs.read_dir(&plugin_dir) {
                if entries.is_empty() {
                    let _ = fs.remove_dir_all(&plugin_dir);
                }
            }
        }

        // マーケットプレイスディレクトリも空なら削除
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

        // kind ディレクトリも空なら削除
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
