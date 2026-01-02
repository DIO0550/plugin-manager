//! プラグイン管理 TUI の描画処理
//!
//! 各画面のレンダリング。
//!
//! ## モジュール構成
//!
//! - `common`: 共通ユーティリティ
//! - `list`: プラグイン一覧画面
//! - `component_types`: コンポーネント種別選択画面
//! - `component_list`: コンポーネント一覧画面

mod common;
mod component_list;
mod component_types;
mod list;

use super::state::{ManagerApp, ManagerScreen};
use ratatui::prelude::*;
use ratatui::widgets::Clear;

/// UI をレンダリング
pub(super) fn draw(f: &mut Frame, app: &mut ManagerApp) {
    // 背景をクリア
    f.render_widget(Clear, f.area());

    match &app.screen {
        ManagerScreen::PluginList => list::render_list_screen(f, app),
        ManagerScreen::ComponentTypes(plugin_idx) => {
            component_types::render_component_types_screen(f, app, *plugin_idx)
        }
        ManagerScreen::ComponentList(plugin_idx, type_idx) => {
            component_list::render_component_list_screen(f, app, *plugin_idx, *type_idx)
        }
    }
}
