//! Installed タブの update（状態更新）
//!
//! メッセージに応じた画面状態の更新ロジック。

use super::actions;
use super::model::{CacheState, DetailAction, Model, Msg};
use crate::tui::manager::core::DataStore;
use ratatui::widgets::ListState;

/// メッセージに応じて状態を更新
pub fn update(model: &mut Model, msg: Msg, data: &mut DataStore) {
    match msg {
        Msg::Up => select_prev(model, data),
        Msg::Down => select_next(model, data),
        Msg::Enter => enter(model, data),
        Msg::Back => back(model),
    }
}

/// 選択を上に移動
fn select_prev(model: &mut Model, data: &DataStore) {
    let len = list_len(model, data);
    if len == 0 {
        return;
    }
    let state = model.current_state_mut();
    let current = state.selected().unwrap_or(0);
    let prev = current.saturating_sub(1);
    state.select(Some(prev));

    // selected_id を更新
    update_selected_id(model, data);
}

/// 選択を下に移動
fn select_next(model: &mut Model, data: &DataStore) {
    let len = list_len(model, data);
    if len == 0 {
        return;
    }
    let state = model.current_state_mut();
    let current = state.selected().unwrap_or(0);
    let next = (current + 1).min(len.saturating_sub(1));
    state.select(Some(next));

    // selected_id を更新
    update_selected_id(model, data);
}

/// 次の階層へ遷移
fn enter(model: &mut Model, data: &mut DataStore) {
    match model {
        Model::PluginList { selected_id, .. } => {
            // PluginList → PluginDetail へ遷移
            if let Some(plugin_id) = selected_id.clone() {
                if data.find_plugin(&plugin_id).is_some() {
                    let mut new_state = ListState::default();
                    new_state.select(Some(0));
                    *model = Model::PluginDetail {
                        plugin_id,
                        state: new_state,
                    };
                }
            }
        }
        Model::PluginDetail { plugin_id, state } => {
            // プラグイン情報を取得
            let plugin = data.find_plugin(plugin_id).cloned();
            let marketplace = plugin.as_ref().and_then(|p| p.marketplace.clone());
            let enabled = plugin.as_ref().is_some_and(|p| p.enabled);

            // enabled 状態に応じたアクション一覧を取得
            let detail_actions = DetailAction::for_plugin(enabled);
            let selected = state.selected().unwrap_or(0);

            match detail_actions.get(selected) {
                Some(DetailAction::DisablePlugin) => {
                    // Disable: デプロイ先から削除、キャッシュは残す
                    let result = actions::disable_plugin(plugin_id, marketplace.as_deref());
                    match result {
                        actions::ActionResult::Success => {
                            // 成功 - プラグインを disabled 状態に更新
                            data.set_plugin_enabled(plugin_id, false);
                        }
                        actions::ActionResult::Error(e) => {
                            data.last_error = Some(e);
                        }
                    }
                }
                Some(DetailAction::EnablePlugin) => {
                    // Enable: キャッシュからデプロイ先に配置
                    let result = actions::enable_plugin(plugin_id, marketplace.as_deref());
                    match result {
                        actions::ActionResult::Success => {
                            // 成功 - プラグインを enabled 状態に更新
                            data.set_plugin_enabled(plugin_id, true);
                        }
                        actions::ActionResult::Error(e) => {
                            data.last_error = Some(e);
                        }
                    }
                }
                Some(DetailAction::Uninstall) => {
                    // Uninstall: デプロイ先 + キャッシュ削除
                    let result = actions::uninstall_plugin(plugin_id, marketplace.as_deref());
                    match result {
                        actions::ActionResult::Success => {
                            // 成功 - プラグインを一覧から削除して PluginList に戻る
                            data.remove_plugin(plugin_id);
                            let mut new_state = ListState::default();
                            if !data.plugins.is_empty() {
                                new_state.select(Some(0));
                            }
                            *model = Model::PluginList {
                                selected_id: data.plugins.first().map(|p| p.name.clone()),
                                state: new_state,
                            };
                        }
                        actions::ActionResult::Error(e) => {
                            data.last_error = Some(e);
                        }
                    }
                }
                Some(DetailAction::ViewComponents) => {
                    // ComponentTypes に遷移
                    let mut new_state = ListState::default();
                    new_state.select(Some(0));
                    *model = Model::ComponentTypes {
                        plugin_id: plugin_id.clone(),
                        selected_kind_idx: 0,
                        state: new_state,
                    };
                }
                Some(DetailAction::Back) => {
                    // PluginList に戻る
                    let id = plugin_id.clone();
                    let mut new_state = ListState::default();
                    new_state.select(Some(0));
                    *model = Model::PluginList {
                        selected_id: Some(id),
                        state: new_state,
                    };
                }
                _ => {
                    // 他のアクションは現時点では何もしない（UI表示のみ）
                }
            }
        }
        Model::ComponentTypes {
            plugin_id, state, ..
        } => {
            if let Some(plugin) = data.find_plugin(plugin_id) {
                let counts = data.available_component_kinds(plugin);
                let selected_idx = state.selected().unwrap_or(0);
                if let Some(count) = counts.get(selected_idx) {
                    let kind = count.kind;
                    let components = data.component_names(plugin, kind);
                    if !components.is_empty() {
                        let mut new_state = ListState::default();
                        new_state.select(Some(0));
                        *model = Model::ComponentList {
                            plugin_id: plugin_id.clone(),
                            kind,
                            selected_idx: 0,
                            state: new_state,
                        };
                    }
                }
            }
        }
        Model::ComponentList { .. } => {
            // 最下層なので何もしない
        }
    }
}

/// 前の階層へ戻る
fn back(model: &mut Model) {
    match model {
        Model::PluginList { .. } => {
            // PluginList での Back は app.rs で Quit 処理される
        }
        Model::PluginDetail { plugin_id, .. } => {
            // PluginDetail → PluginList へ戻る
            let id = plugin_id.clone();
            let mut new_state = ListState::default();
            new_state.select(Some(0));
            *model = Model::PluginList {
                selected_id: Some(id),
                state: new_state,
            };
        }
        Model::ComponentTypes { plugin_id, .. } => {
            // ComponentTypes → PluginDetail へ戻る
            let plugin_id = plugin_id.clone();
            let mut new_state = ListState::default();
            new_state.select(Some(0));
            *model = Model::PluginDetail {
                plugin_id,
                state: new_state,
            };
        }
        Model::ComponentList {
            plugin_id, kind: _, ..
        } => {
            let plugin_id = plugin_id.clone();
            let mut new_state = ListState::default();
            new_state.select(Some(0));
            *model = Model::ComponentTypes {
                plugin_id,
                selected_kind_idx: 0,
                state: new_state,
            };
        }
    }
}

/// 現在の画面のリスト長を取得
fn list_len(model: &Model, data: &DataStore) -> usize {
    match model {
        Model::PluginList { .. } => data.plugins.len(),
        Model::PluginDetail { plugin_id, .. } => {
            let enabled = data.find_plugin(plugin_id).is_some_and(|p| p.enabled);
            DetailAction::for_plugin(enabled).len()
        }
        Model::ComponentTypes { plugin_id, .. } => {
            if let Some(plugin) = data.find_plugin(plugin_id) {
                data.available_component_kinds(plugin).len().max(1)
            } else {
                0
            }
        }
        Model::ComponentList {
            plugin_id, kind, ..
        } => {
            if let Some(plugin) = data.find_plugin(plugin_id) {
                data.component_names(plugin, *kind).len()
            } else {
                0
            }
        }
    }
}

/// selected_id を現在のインデックスから更新
fn update_selected_id(model: &mut Model, data: &DataStore) {
    if let Model::PluginList { selected_id, state } = model {
        if let Some(idx) = state.selected() {
            *selected_id = data.plugins.get(idx).map(|p| p.name.clone());
        }
    }
}
