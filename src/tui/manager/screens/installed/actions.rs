//! Installed タブのアクション実行
//!
//! Disable/Uninstall などのプラグイン操作を実行する。
//! Application層のユースケースに委譲する。

use super::model::UpdateStatusDisplay;
use crate::application;
use crate::plugin::{update_plugin, PackageCache, UpdateStatus};
use crate::tui::manager::core::PluginKey;
use crate::tui::output_suppress::OutputSuppressGuard;
use std::env;
use std::path::Path;

/// アクション実行結果
#[derive(Debug)]
pub enum ActionResult {
    /// 成功
    Success,
    /// エラー
    Error(String),
}

impl From<application::OperationResult> for ActionResult {
    fn from(result: application::OperationResult) -> Self {
        if result.success {
            ActionResult::Success
        } else {
            ActionResult::Error(result.error.unwrap_or_else(|| "Unknown error".to_string()))
        }
    }
}

/// キャッシュ初期化ヘルパー
fn new_cache() -> Result<PackageCache, String> {
    PackageCache::new().map_err(|e| format!("Failed to access cache: {}", e))
}

/// プラグインを Disable（デプロイ先から削除、キャッシュは残す）
///
/// # Arguments
///
/// * `plugin_name` - Target plugin id.
/// * `marketplace` - Optional marketplace name the plugin belongs to.
pub fn disable_plugin(plugin_name: &str, marketplace: Option<&str>) -> ActionResult {
    let cache = match new_cache() {
        Ok(c) => c,
        Err(e) => return ActionResult::Error(e),
    };
    let project_root = env::current_dir().unwrap_or_else(|_| ".".into());
    application::disable_plugin(&cache, plugin_name, marketplace, &project_root, None).into()
}

/// プラグインを Uninstall（デプロイ先 + キャッシュ削除）
///
/// # Arguments
///
/// * `plugin_name` - Target plugin id.
/// * `marketplace` - Optional marketplace name the plugin belongs to.
pub fn uninstall_plugin(plugin_name: &str, marketplace: Option<&str>) -> ActionResult {
    let cache = match new_cache() {
        Ok(c) => c,
        Err(e) => return ActionResult::Error(e),
    };
    let project_root = env::current_dir().unwrap_or_else(|_| ".".into());
    application::uninstall_plugin(&cache, plugin_name, marketplace, &project_root).into()
}

/// プラグインを Enable（キャッシュからデプロイ先に配置）
///
/// # Arguments
///
/// * `plugin_name` - Target plugin id.
/// * `marketplace` - Optional marketplace name the plugin belongs to.
pub fn enable_plugin(plugin_name: &str, marketplace: Option<&str>) -> ActionResult {
    let cache = match new_cache() {
        Ok(c) => c,
        Err(e) => return ActionResult::Error(e),
    };
    let project_root = env::current_dir().unwrap_or_else(|_| ".".into());
    application::enable_plugin(&cache, plugin_name, marketplace, &project_root, None).into()
}

/// バッチ更新を実行
///
/// 各プラグインを順次 `update_plugin()` で更新し、結果を返す。
/// stdout/stderr をリダイレクトして TUI 画面の乱れを防ぐ。
///
/// # Arguments
///
/// * `keys` - Plugin keys to update in order.
pub fn batch_update_plugins(keys: &[PluginKey]) -> Vec<(PluginKey, UpdateStatusDisplay)> {
    let project_root = env::current_dir().unwrap_or_else(|_| ".".into());

    // stdout/stderr をリダイレクト
    // Note: ガード作成失敗時は抑制なしで続行する（TUI 画面が乱れる可能性あり）。
    // TUI 代替スクリーン上では eprintln! が表示されないため、ログ出力は行わない。
    let _guard = OutputSuppressGuard::new();

    keys.iter()
        .map(|key| {
            let status = run_update_plugin(key, &project_root);
            (key.clone(), status)
        })
        .collect()
}

/// 単一プラグインの更新を同期的に実行
///
/// # Arguments
///
/// * `key` - Target plugin key (`marketplace` + `cache_id`).
/// * `project_root` - Project root directory used for deployment paths.
fn run_update_plugin(key: &PluginKey, project_root: &Path) -> UpdateStatusDisplay {
    let handle = match tokio::runtime::Handle::try_current() {
        Ok(h) => h,
        Err(_) => {
            return UpdateStatusDisplay::Failed("No Tokio runtime available".to_string());
        }
    };

    let cache = match new_cache() {
        Ok(c) => c,
        Err(msg) => return UpdateStatusDisplay::Failed(msg),
    };

    let result = tokio::task::block_in_place(|| {
        handle.block_on(update_plugin(
            &cache,
            &key.cache_id,
            key.marketplace.as_deref(),
            project_root,
            None,
        ))
    });

    match result.status {
        UpdateStatus::Updated { .. } => UpdateStatusDisplay::Updated,
        UpdateStatus::AlreadyUpToDate => UpdateStatusDisplay::AlreadyUpToDate,
        UpdateStatus::Skipped { reason } => UpdateStatusDisplay::Skipped(reason),
        UpdateStatus::Failed => {
            UpdateStatusDisplay::Failed(result.error.unwrap_or_else(|| "Unknown error".to_string()))
        }
    }
}
