//! プラグイン操作ユースケース
//!
//! Enable/Disable/Uninstall などのプラグインライフサイクル操作を提供する。
//!
//! ## Functional Core / Imperative Shell パターン
//!
//! このモジュールは以下のフローで動作する：
//! 1. Imperative Shell: プラグインのスキャン（I/O）
//! 2. Functional Core: `PluginPlan::expand()` で操作を計画（純粋関数）
//! 3. Imperative Shell: `PluginPlan::apply()` で実行（I/O）

use super::plugin_action::{PluginAction, PluginPlan};
use crate::component::Component;
use crate::plugin::{CachedPlugin, PluginCache, PluginManifest};
use crate::target::{all_targets, OperationResult, PluginOrigin};
use std::fs;
use std::path::{Path, PathBuf};

/// プラグインを Disable（デプロイ先から削除、キャッシュは残す）
///
/// # Arguments
/// * `plugin_name` - プラグイン名
/// * `marketplace` - マーケットプレイス名（任意）
/// * `project_root` - プロジェクトルートパス
pub fn disable_plugin(
    plugin_name: &str,
    marketplace: Option<&str>,
    project_root: &Path,
) -> OperationResult {
    let cache = match PluginCache::new() {
        Ok(c) => c,
        Err(e) => return OperationResult::error(format!("Failed to access cache: {}", e)),
    };

    // プラグインがキャッシュに存在するか確認
    if !cache.is_cached(marketplace, plugin_name) {
        return OperationResult::error(format!("Plugin '{}' not found in cache", plugin_name));
    }

    // Imperative Shell: コンポーネントをスキャン（I/O）
    let plugin = match load_plugin_deployment(&cache, marketplace, plugin_name) {
        Ok(p) => p,
        Err(e) => return OperationResult::error(e),
    };
    let components = plugin.components();

    // Functional Core: 計画を生成（純粋）
    let plan = PluginPlan::new(
        PluginAction::Disable {
            plugin_name: plugin_name.to_string(),
            marketplace: marketplace.map(|s| s.to_string()),
        },
        components,
        project_root.to_path_buf(),
    );

    // Imperative Shell: 実行（I/O）
    let result = plan.apply();

    // 後処理: 空になったディレクトリをクリーンアップ
    if result.success {
        for target in &all_targets() {
            cleanup_plugin_directories(target.name(), &plugin.origin, project_root);
        }
    }

    result
}

/// プラグインを Enable（キャッシュからデプロイ先に配置）
///
/// # Arguments
/// * `plugin_name` - プラグイン名
/// * `marketplace` - マーケットプレイス名（任意）
/// * `project_root` - プロジェクトルートパス
pub fn enable_plugin(
    plugin_name: &str,
    marketplace: Option<&str>,
    project_root: &Path,
) -> OperationResult {
    let cache = match PluginCache::new() {
        Ok(c) => c,
        Err(e) => return OperationResult::error(format!("Failed to access cache: {}", e)),
    };

    // プラグインがキャッシュに存在するか確認
    if !cache.is_cached(marketplace, plugin_name) {
        return OperationResult::error(format!("Plugin '{}' not found in cache", plugin_name));
    }

    // Imperative Shell: コンポーネントをスキャン（I/O）
    let plugin = match load_plugin_deployment(&cache, marketplace, plugin_name) {
        Ok(p) => p,
        Err(e) => return OperationResult::error(e),
    };
    let components = plugin.components();

    // Functional Core: 計画を生成（純粋）
    let plan = PluginPlan::new(
        PluginAction::Enable {
            plugin_name: plugin_name.to_string(),
            marketplace: marketplace.map(|s| s.to_string()),
        },
        components,
        project_root.to_path_buf(),
    );

    // Imperative Shell: 実行（I/O）
    plan.apply()
}

/// プラグインを Uninstall（デプロイ先 + キャッシュ削除）
///
/// # Arguments
/// * `plugin_name` - プラグイン名
/// * `marketplace` - マーケットプレイス名（任意）
/// * `project_root` - プロジェクトルートパス
pub fn uninstall_plugin(
    plugin_name: &str,
    marketplace: Option<&str>,
    project_root: &Path,
) -> OperationResult {
    // まずデプロイ先から削除
    let disable_result = disable_plugin(plugin_name, marketplace, project_root);
    if !disable_result.success {
        return disable_result;
    }

    // キャッシュから削除
    let cache = match PluginCache::new() {
        Ok(c) => c,
        Err(e) => return OperationResult::error(format!("Failed to access cache: {}", e)),
    };

    if let Err(e) = cache.remove(marketplace, plugin_name) {
        return OperationResult::error(format!("Failed to remove from cache: {}", e));
    }

    disable_result
}

/// プラグインディレクトリをクリーンアップ
///
/// コンポーネント削除後に空になったプラグインディレクトリを削除する。
fn cleanup_plugin_directories(target_name: &str, origin: &PluginOrigin, project_root: &Path) {
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
        if plugin_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&plugin_dir) {
                if entries.count() == 0 {
                    let _ = fs::remove_dir(&plugin_dir);
                }
            }
        }

        // マーケットプレイスディレクトリも空なら削除
        let marketplace_dir = project_root
            .join(base_dir)
            .join(kind_dir)
            .join(&origin.marketplace);

        if marketplace_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&marketplace_dir) {
                if entries.count() == 0 {
                    let _ = fs::remove_dir(&marketplace_dir);
                }
            }
        }

        // kind ディレクトリも空なら削除
        let kind_dir_path = project_root.join(base_dir).join(kind_dir);

        if kind_dir_path.is_dir() {
            if let Ok(entries) = fs::read_dir(&kind_dir_path) {
                if entries.count() == 0 {
                    let _ = fs::remove_dir(&kind_dir_path);
                }
            }
        }
    }
}

/// キャッシュから PluginDeployment を読み込む
///
/// マニフェストとパス情報を含む PluginDeployment を構築する。
fn load_plugin_deployment(
    cache: &PluginCache,
    marketplace: Option<&str>,
    plugin_name: &str,
) -> Result<PluginDeployment, String> {
    let manifest = cache
        .load_manifest(marketplace, plugin_name)
        .map_err(|e| format!("Failed to load manifest: {}", e))?;

    let origin = match marketplace {
        Some(mp) => PluginOrigin::from_marketplace(mp, &manifest.name),
        None => PluginOrigin::from_marketplace("github", &manifest.name),
    };

    let plugin_path = cache.plugin_path(marketplace, plugin_name);

    Ok(PluginDeployment {
        origin,
        manifest,
        path: plugin_path,
    })
}

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
