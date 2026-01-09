//! Installed タブのアクション実行
//!
//! Disable/Uninstall などのプラグイン操作を実行する。
//! Application層のユースケースに委譲する。

use crate::application;
use std::env;

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

/// プラグインを Disable（デプロイ先から削除、キャッシュは残す）
pub fn disable_plugin(plugin_name: &str, marketplace: Option<&str>) -> ActionResult {
    let project_root = env::current_dir().unwrap_or_else(|_| ".".into());
    application::disable_plugin(plugin_name, marketplace, &project_root).into()
}

/// プラグインを Uninstall（デプロイ先 + キャッシュ削除）
pub fn uninstall_plugin(plugin_name: &str, marketplace: Option<&str>) -> ActionResult {
    let project_root = env::current_dir().unwrap_or_else(|_| ".".into());
    application::uninstall_plugin(plugin_name, marketplace, &project_root).into()
}

/// プラグインを Enable（キャッシュからデプロイ先に配置）
pub fn enable_plugin(plugin_name: &str, marketplace: Option<&str>) -> ActionResult {
    let project_root = env::current_dir().unwrap_or_else(|_| ".".into());
    application::enable_plugin(plugin_name, marketplace, &project_root).into()
}
