//! Installed タブのアクション実行
//!
//! Disable/Uninstall などのプラグイン操作を実行する。
//! Application層のユースケースに委譲する。

use super::model::UpdateStatusDisplay;
use crate::application;
use crate::plugin::{update_plugin, PluginCache, UpdateStatus};
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
fn new_cache() -> Result<PluginCache, String> {
    PluginCache::new().map_err(|e| format!("Failed to access cache: {}", e))
}

/// プラグインを Disable（デプロイ先から削除、キャッシュは残す）
pub fn disable_plugin(plugin_name: &str, marketplace: Option<&str>) -> ActionResult {
    let cache = match new_cache() {
        Ok(c) => c,
        Err(e) => return ActionResult::Error(e),
    };
    let project_root = env::current_dir().unwrap_or_else(|_| ".".into());
    application::disable_plugin(&cache, plugin_name, marketplace, &project_root, None).into()
}

/// プラグインを Uninstall（デプロイ先 + キャッシュ削除）
pub fn uninstall_plugin(plugin_name: &str, marketplace: Option<&str>) -> ActionResult {
    let cache = match new_cache() {
        Ok(c) => c,
        Err(e) => return ActionResult::Error(e),
    };
    let project_root = env::current_dir().unwrap_or_else(|_| ".".into());
    application::uninstall_plugin(&cache, plugin_name, marketplace, &project_root).into()
}

/// プラグインを Enable（キャッシュからデプロイ先に配置）
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
pub fn batch_update_plugins(plugin_names: &[String]) -> Vec<(String, UpdateStatusDisplay)> {
    let project_root = env::current_dir().unwrap_or_else(|_| ".".into());

    // stdout/stderr をリダイレクト
    // Note: ガード作成失敗時は抑制なしで続行する（TUI 画面が乱れる可能性あり）。
    // TUI 代替スクリーン上では eprintln! が表示されないため、ログ出力は行わない。
    let _guard = OutputSuppressGuard::new();

    plugin_names
        .iter()
        .map(|name| {
            let status = run_update_plugin(name, &project_root);
            (name.clone(), status)
        })
        .collect()
}

/// 単一プラグインの更新を同期的に実行
fn run_update_plugin(plugin_name: &str, project_root: &Path) -> UpdateStatusDisplay {
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
        handle.block_on(update_plugin(&cache, plugin_name, project_root, None))
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
