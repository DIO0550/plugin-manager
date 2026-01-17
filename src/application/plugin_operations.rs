//! プラグイン操作ユースケース
//!
//! Enable/Disable/Uninstall などのプラグインライフサイクル操作を提供する。
//!
//! ## Functional Core / Imperative Shell パターン
//!
//! このモジュールは以下のフローで動作する：
//! 1. Imperative Shell: プラグインのスキャン（I/O）
//! 2. Functional Core: `PluginIntent::expand()` で操作を展開（純粋関数）
//! 3. Imperative Shell: `PluginIntent::apply()` で実行（I/O）

use super::plugin_action::{PluginAction, PluginIntent};
use crate::component::Component;
use crate::fs::{FileSystem, RealFs};
use crate::plugin::{CachedPlugin, PluginCache, PluginManifest};
use crate::target::{all_targets, OperationResult, PluginOrigin};
use std::path::{Path, PathBuf};

/// プラグインを Disable（デプロイ先から削除、キャッシュは残す）
///
/// # Arguments
/// * `plugin_name` - プラグイン名
/// * `marketplace` - マーケットプレイス名（任意）
/// * `project_root` - プロジェクトルートパス
/// * `target_filter` - ターゲットフィルタ（None で全ターゲット）
pub fn disable_plugin(
    plugin_name: &str,
    marketplace: Option<&str>,
    project_root: &Path,
    target_filter: Option<&str>,
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

    // Functional Core: 意図を生成（純粋）
    let intent = PluginIntent::with_target_filter(
        PluginAction::Disable {
            plugin_name: plugin_name.to_string(),
            marketplace: marketplace.map(|s| s.to_string()),
        },
        components,
        project_root.to_path_buf(),
        target_filter,
    );

    // Imperative Shell: 実行（I/O）
    let result = intent.apply();

    // 後処理: 空になったディレクトリをクリーンアップ
    if result.success {
        let targets_to_cleanup: Vec<_> = match target_filter {
            Some(filter) => all_targets()
                .into_iter()
                .filter(|t| t.name() == filter)
                .collect(),
            None => all_targets(),
        };
        for target in &targets_to_cleanup {
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
/// * `target_filter` - ターゲットフィルタ（None で全ターゲット）
pub fn enable_plugin(
    plugin_name: &str,
    marketplace: Option<&str>,
    project_root: &Path,
    target_filter: Option<&str>,
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

    // Functional Core: 意図を生成（純粋）
    let intent = PluginIntent::with_target_filter(
        PluginAction::Enable {
            plugin_name: plugin_name.to_string(),
            marketplace: marketplace.map(|s| s.to_string()),
        },
        components,
        project_root.to_path_buf(),
        target_filter,
    );

    // Imperative Shell: 実行（I/O）
    intent.apply()
}

/// アンインストール前の情報取得
///
/// プラグインの存在確認と、削除対象の情報を取得する。
///
/// # Arguments
/// * `plugin_name` - プラグイン名
/// * `marketplace` - マーケットプレイス名（任意、デフォルト: "github"）
///
/// # Returns
/// * `Ok(UninstallInfo)` - プラグイン情報
/// * `Err(String)` - プラグインが見つからない場合のエラー
pub fn get_uninstall_info(
    plugin_name: &str,
    marketplace: Option<&str>,
) -> Result<UninstallInfo, String> {
    let cache = PluginCache::new().map_err(|e| format!("Failed to access cache: {}", e))?;
    let marketplace_str = marketplace.unwrap_or("github");

    // 存在確認
    if !cache.is_cached(Some(marketplace_str), plugin_name) {
        return Err(format!(
            "Plugin '{}' not found in cache (marketplace: {})",
            plugin_name, marketplace_str
        ));
    }

    // コンポーネント情報取得
    let plugin = load_plugin_deployment(&cache, Some(marketplace_str), plugin_name)?;
    let components = plugin.components();

    // 影響を受けるターゲット
    let affected_targets = all_targets()
        .iter()
        .filter(|t| components.iter().any(|c| t.supports(c.kind)))
        .map(|t| t.name().to_string())
        .collect();

    Ok(UninstallInfo {
        plugin_name: plugin_name.to_string(),
        marketplace: marketplace_str.to_string(),
        components,
        affected_targets,
    })
}

/// アンインストール情報
#[derive(Debug)]
pub struct UninstallInfo {
    /// プラグイン名
    pub plugin_name: String,
    /// マーケットプレイス名
    pub marketplace: String,
    /// コンポーネント一覧
    pub components: Vec<Component>,
    /// 影響を受けるターゲット名一覧
    pub affected_targets: Vec<String>,
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
    // まずデプロイ先から削除（全ターゲット）
    let disable_result = disable_plugin(plugin_name, marketplace, project_root, None);
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

#[cfg(test)]
#[path = "plugin_operations_test.rs"]
mod tests;
