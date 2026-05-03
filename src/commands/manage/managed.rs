//! プラグイン管理（TUI）コマンド
//!
//! `plm managed` でインタラクティブなプラグイン管理画面を起動する。

use crate::tui::manager;

pub async fn run() -> Result<(), String> {
    manager::run().map_err(|e| e.to_string())
}
