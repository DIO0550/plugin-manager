//! プラグイン管理（TUI）コマンド
//!
//! `plm managed` でインタラクティブなプラグイン管理画面を起動する。

use crate::tui::plugin_manager;

pub async fn run() -> Result<(), String> {
    plugin_manager::run().map_err(|e| e.to_string())
}
