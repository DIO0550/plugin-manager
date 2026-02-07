//! Installed タブの update（状態更新）
//!
//! メッセージに応じた画面状態の更新ロジック。

use super::actions;
use super::model::{DetailAction, Model, Msg, UpdateStatusDisplay};
use crate::tui::manager::core::{filter_plugins, DataStore};
use ratatui::widgets::ListState;
use std::collections::HashMap;

/// update() の戻り値
///
/// フィルタフォーカス移動とバッチ更新実行の2つの副作用を伝える。
pub struct UpdateEffect {
    /// フィルタ入力欄へフォーカス移動すべき
    pub should_focus_filter: bool,
    /// 描画後にバッチ更新を実行すべき（2段階方式の Phase 1 完了後）
    pub needs_execute_batch: bool,
}

impl UpdateEffect {
    fn none() -> Self {
        Self {
            should_focus_filter: false,
            needs_execute_batch: false,
        }
    }

    fn focus_filter() -> Self {
        Self {
            should_focus_filter: true,
            needs_execute_batch: false,
        }
    }

    fn execute_batch() -> Self {
        Self {
            should_focus_filter: false,
            needs_execute_batch: true,
        }
    }
}

/// メッセージに応じて状態を更新
pub fn update(
    model: &mut Model,
    msg: Msg,
    data: &mut DataStore,
    filter_text: &str,
) -> UpdateEffect {
    match msg {
        Msg::Up => {
            if select_prev(model, data, filter_text) {
                UpdateEffect::focus_filter()
            } else {
                UpdateEffect::none()
            }
        }
        Msg::Down => {
            select_next(model, data, filter_text);
            UpdateEffect::none()
        }
        Msg::Enter => enter(model, data, filter_text),
        Msg::Back => {
            back(model, filter_text, data);
            UpdateEffect::none()
        }
        Msg::ToggleMark => {
            toggle_mark(model);
            UpdateEffect::none()
        }
        Msg::ToggleAllMarks => {
            toggle_all_marks(model, data, filter_text);
            UpdateEffect::none()
        }
        Msg::BatchUpdate => batch_update(model),
        Msg::UpdateAll => update_all(model, data),
        Msg::ExecuteBatch => {
            execute_batch(model, data, filter_text);
            UpdateEffect::none()
        }
    }
}

/// 個別プラグインのマークをトグル
fn toggle_mark(model: &mut Model) {
    if let Model::PluginList {
        selected_id,
        marked_ids,
        ..
    } = model
    {
        if let Some(id) = selected_id.as_ref() {
            if marked_ids.contains(id) {
                marked_ids.remove(id);
            } else {
                marked_ids.insert(id.clone());
            }
        }
    }
}

/// フィルタ済みプラグインの一括マークトグル
fn toggle_all_marks(model: &mut Model, data: &DataStore, filter_text: &str) {
    if let Model::PluginList { marked_ids, .. } = model {
        let filtered = filter_plugins(&data.plugins, filter_text);

        // フィルタ済み全プラグインが既にマーク済みかチェック
        let all_marked = !filtered.is_empty()
            && filtered
                .iter()
                .all(|plugin| marked_ids.contains(&plugin.name));

        if all_marked {
            // フィルタ済み分のみ解除
            for plugin in &filtered {
                marked_ids.remove(&plugin.name);
            }
        } else {
            // フィルタ済みプラグインを全マーク
            for plugin in &filtered {
                marked_ids.insert(plugin.name.clone());
            }
        }
    }
}

/// Phase 1: マーク済みプラグインのステータスを Updating にセット
fn batch_update(model: &mut Model) -> UpdateEffect {
    if let Model::PluginList {
        marked_ids,
        update_statuses,
        ..
    } = model
    {
        if marked_ids.is_empty() {
            return UpdateEffect::none();
        }

        // 前回バッチの古いステータスをクリアしてから新バッチの Updating をセット
        update_statuses.clear();
        for id in marked_ids.iter() {
            update_statuses.insert(id.clone(), UpdateStatusDisplay::Updating);
        }

        return UpdateEffect::execute_batch();
    }

    UpdateEffect::none()
}

/// Phase 1: 全プラグインのステータスを Updating にセット
fn update_all(model: &mut Model, data: &DataStore) -> UpdateEffect {
    if let Model::PluginList {
        update_statuses, ..
    } = model
    {
        if data.plugins.is_empty() {
            return UpdateEffect::none();
        }

        // 前回の古いステータスをクリアしてから全プラグインに Updating をセット
        update_statuses.clear();
        for plugin in &data.plugins {
            update_statuses.insert(plugin.name.clone(), UpdateStatusDisplay::Updating);
        }

        return UpdateEffect::execute_batch();
    }

    UpdateEffect::none()
}

/// Phase 2: 実際のバッチ更新処理を実行
fn execute_batch(model: &mut Model, data: &mut DataStore, filter_text: &str) {
    execute_batch_with(
        model,
        data,
        filter_text,
        |names| actions::batch_update_plugins(names),
        |d| d.reload(),
    );
}

/// Phase 2 の実装本体（依存関数を注入可能）
///
/// テストでは `run_updates` と `reload` にスタブを注入して
/// ファイルシステムアクセスなしで密閉的にテストできる。
fn execute_batch_with(
    model: &mut Model,
    data: &mut DataStore,
    filter_text: &str,
    run_updates: impl FnOnce(&[String]) -> Vec<(String, UpdateStatusDisplay)>,
    reload: impl FnOnce(&mut DataStore) -> std::io::Result<()>,
) {
    if let Model::PluginList {
        marked_ids,
        update_statuses,
        selected_id,
        state,
    } = model
    {
        // update_statuses から Updating のプラグイン名を収集
        let plugin_names: Vec<String> = update_statuses
            .iter()
            .filter(|(_, status)| matches!(status, UpdateStatusDisplay::Updating))
            .map(|(name, _)| name.clone())
            .filter(|name| data.find_plugin(name).is_some())
            .collect();

        // バッチ更新実行
        let results = run_updates(&plugin_names);

        // 結果を update_statuses に反映
        let mut new_statuses = HashMap::new();
        let mut batch_errors: Vec<String> = Vec::new();
        for (name, status) in results {
            // Failed の詳細を収集
            if let UpdateStatusDisplay::Failed(ref reason) = status {
                batch_errors.push(format!("Update failed for {}: {}", name, reason));
            }
            new_statuses.insert(name, status);
        }

        // 収集したエラーを last_error に集約
        if !batch_errors.is_empty() {
            let aggregated = if batch_errors.len() == 1 {
                batch_errors.into_iter().next().unwrap()
            } else {
                format!(
                    "{} plugins failed during batch update:\n{}",
                    batch_errors.len(),
                    batch_errors.join("\n")
                )
            };
            data.last_error = Some(aggregated);
        }
        *update_statuses = new_statuses;

        // DataStore を全体リロード
        if let Err(e) = reload(data) {
            let reload_msg = format!("Failed to reload plugins: {}", e);
            data.last_error = Some(match data.last_error.take() {
                Some(prev) => format!("{prev}\n{reload_msg}"),
                None => reload_msg,
            });
        }

        // reload 後に存在しなくなったプラグインをマークから除去
        marked_ids.retain(|id| data.find_plugin(id).is_some());

        // マーク済みIDの条件付きクリア:
        // marked_ids 内の全プラグインが update_statuses に含まれている場合のみクリア
        if marked_ids
            .iter()
            .all(|id| update_statuses.contains_key(id))
        {
            marked_ids.clear();
        }

        // reload 後にフィルタ済みリストに対して選択状態を再同期
        let filtered = filter_plugins(&data.plugins, filter_text);
        let current_selected = selected_id.as_ref();
        let new_idx = current_selected
            .and_then(|id| filtered.iter().position(|p| &p.name == id))
            .or(if filtered.is_empty() { None } else { Some(0) });

        state.select(new_idx);
        *selected_id = new_idx.and_then(|idx| filtered.get(idx).map(|p| p.name.clone()));
    }
}

/// 選択を上に移動
///
/// 戻り値: `true` ならリスト先頭で↑が押され、フィルタへフォーカス移動すべき
fn select_prev(model: &mut Model, data: &DataStore, filter_text: &str) -> bool {
    let len = list_len(model, data, filter_text);
    if len == 0 {
        // リストが空の場合もフィルタへフォーカス移動
        if matches!(model, Model::PluginList { .. }) {
            return true;
        }
        return false;
    }
    let state = model.current_state_mut();
    let current = state.selected().unwrap_or(0);
    if current == 0 {
        // PluginList の先頭ならフィルタへフォーカス移動
        if matches!(model, Model::PluginList { .. }) {
            return true;
        }
        return false;
    }
    let prev = current.saturating_sub(1);
    state.select(Some(prev));

    // selected_id を更新
    update_selected_id(model, data, filter_text);
    false
}

/// 選択を下に移動
fn select_next(model: &mut Model, data: &DataStore, filter_text: &str) {
    let len = list_len(model, data, filter_text);
    if len == 0 {
        return;
    }
    let state = model.current_state_mut();
    let current = state.selected().unwrap_or(0);
    let next = (current + 1).min(len.saturating_sub(1));
    state.select(Some(next));

    // selected_id を更新
    update_selected_id(model, data, filter_text);
}

/// 次の階層へ遷移
fn enter(model: &mut Model, data: &mut DataStore, filter_text: &str) -> UpdateEffect {
    match model {
        Model::PluginList {
            selected_id,
            marked_ids,
            update_statuses,
            ..
        } => {
            // PluginList → PluginDetail へ遷移（マーク状態を保存）
            if let Some(plugin_id) = selected_id.clone() {
                if data.find_plugin(&plugin_id).is_some() {
                    let saved_marked = std::mem::take(marked_ids);
                    let saved_statuses = std::mem::take(update_statuses);
                    let mut new_state = ListState::default();
                    new_state.select(Some(0));
                    *model = Model::PluginDetail {
                        plugin_id,
                        state: new_state,
                        saved_marked_ids: saved_marked,
                        saved_update_statuses: saved_statuses,
                    };
                }
            }
            UpdateEffect::none()
        }
        Model::PluginDetail {
            plugin_id,
            state,
            saved_marked_ids,
            saved_update_statuses,
        } => {
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
                            let uninstalled_id = plugin_id.clone();
                            let mut restored_marks = std::mem::take(saved_marked_ids);
                            let mut restored_statuses = std::mem::take(saved_update_statuses);
                            restored_marks.remove(&uninstalled_id);
                            restored_statuses.remove(&uninstalled_id);
                            data.remove_plugin(&uninstalled_id);
                            // フィルタ済みリストに対して選択を同期
                            let filtered = filter_plugins(&data.plugins, filter_text);
                            let mut new_state = ListState::default();
                            let selected_id = if !filtered.is_empty() {
                                new_state.select(Some(0));
                                Some(filtered[0].name.clone())
                            } else {
                                None
                            };
                            *model = Model::PluginList {
                                selected_id,
                                state: new_state,
                                marked_ids: restored_marks,
                                update_statuses: restored_statuses,
                            };
                        }
                        actions::ActionResult::Error(e) => {
                            data.last_error = Some(e);
                        }
                    }
                }
                Some(DetailAction::ViewComponents) => {
                    // ComponentTypes に遷移（マーク状態を引き継ぐ）
                    let restored_marks = std::mem::take(saved_marked_ids);
                    let restored_statuses = std::mem::take(saved_update_statuses);
                    let mut new_state = ListState::default();
                    new_state.select(Some(0));
                    *model = Model::ComponentTypes {
                        plugin_id: plugin_id.clone(),
                        selected_kind_idx: 0,
                        state: new_state,
                        saved_marked_ids: restored_marks,
                        saved_update_statuses: restored_statuses,
                    };
                }
                Some(DetailAction::Back) => {
                    // PluginList に戻る（マーク状態を復元、選択をフィルタ済みリストと同期）
                    let id = plugin_id.clone();
                    let restored_marks = std::mem::take(saved_marked_ids);
                    let restored_statuses = std::mem::take(saved_update_statuses);
                    let filtered = filter_plugins(&data.plugins, filter_text);
                    let mut new_state = ListState::default();
                    let idx = filtered.iter().position(|p| p.name == id);
                    let selected_id = if let Some(idx) = idx {
                        new_state.select(Some(idx));
                        Some(id)
                    } else if !filtered.is_empty() {
                        new_state.select(Some(0));
                        Some(filtered[0].name.clone())
                    } else {
                        None
                    };
                    *model = Model::PluginList {
                        selected_id,
                        state: new_state,
                        marked_ids: restored_marks,
                        update_statuses: restored_statuses,
                    };
                }
                Some(DetailAction::UpdateNow) => {
                    // UpdateNow: 単一プラグインを即時更新
                    let target_id = plugin_id.clone();
                    let restored_marks = std::mem::take(saved_marked_ids);
                    let mut restored_statuses = std::mem::take(saved_update_statuses);
                    // 古いステータスをクリアして対象プラグインに Updating をセット
                    restored_statuses.clear();
                    restored_statuses
                        .insert(target_id.clone(), UpdateStatusDisplay::Updating);
                    // PluginList に遷移（フィルタ済みリストで選択位置を同期）
                    let filtered = filter_plugins(&data.plugins, filter_text);
                    let mut new_state = ListState::default();
                    let idx = filtered.iter().position(|p| p.name == target_id);
                    let selected_id = if let Some(idx) = idx {
                        new_state.select(Some(idx));
                        Some(target_id)
                    } else if !filtered.is_empty() {
                        new_state.select(Some(0));
                        Some(filtered[0].name.clone())
                    } else {
                        None
                    };
                    *model = Model::PluginList {
                        selected_id,
                        state: new_state,
                        marked_ids: restored_marks,
                        update_statuses: restored_statuses,
                    };
                    return UpdateEffect::execute_batch();
                }
                _ => {
                    // MarkForUpdate は UI 表示のみ（BatchUpdate 経由で処理）
                }
            }
            UpdateEffect::none()
        }
        Model::ComponentTypes {
            plugin_id,
            state,
            saved_marked_ids,
            saved_update_statuses,
            ..
        } => {
            if let Some(plugin) = data.find_plugin(plugin_id) {
                let counts = data.available_component_kinds(plugin);
                let selected_idx = state.selected().unwrap_or(0);
                if let Some(count) = counts.get(selected_idx) {
                    let kind = count.kind;
                    let components = data.component_names(plugin, kind);
                    if !components.is_empty() {
                        let restored_marks = std::mem::take(saved_marked_ids);
                        let restored_statuses = std::mem::take(saved_update_statuses);
                        let mut new_state = ListState::default();
                        new_state.select(Some(0));
                        *model = Model::ComponentList {
                            plugin_id: plugin_id.clone(),
                            kind,
                            selected_idx: 0,
                            state: new_state,
                            saved_marked_ids: restored_marks,
                            saved_update_statuses: restored_statuses,
                        };
                    }
                }
            }
            UpdateEffect::none()
        }
        Model::ComponentList { .. } => {
            // 最下層なので何もしない
            UpdateEffect::none()
        }
    }
}

/// 前の階層へ戻る
fn back(model: &mut Model, filter_text: &str, data: &DataStore) {
    match model {
        Model::PluginList { .. } => {
            // PluginList での Back は app.rs で Quit 処理される
        }
        Model::PluginDetail {
            plugin_id,
            saved_marked_ids,
            saved_update_statuses,
            ..
        } => {
            // PluginDetail → PluginList へ戻る（マーク状態を復元）
            let id = plugin_id.clone();
            let restored_marks = std::mem::take(saved_marked_ids);
            let restored_statuses = std::mem::take(saved_update_statuses);
            let filtered = filter_plugins(&data.plugins, filter_text);
            let mut new_state = ListState::default();
            let idx = filtered.iter().position(|p| p.name == id);
            let selected_id = if let Some(idx) = idx {
                new_state.select(Some(idx));
                Some(id)
            } else if !filtered.is_empty() {
                new_state.select(Some(0));
                Some(filtered[0].name.clone())
            } else {
                None
            };
            *model = Model::PluginList {
                selected_id,
                state: new_state,
                marked_ids: restored_marks,
                update_statuses: restored_statuses,
            };
        }
        Model::ComponentTypes {
            plugin_id,
            saved_marked_ids,
            saved_update_statuses,
            ..
        } => {
            // ComponentTypes → PluginDetail へ戻る（マーク状態を復元）
            let plugin_id = plugin_id.clone();
            let restored_marks = std::mem::take(saved_marked_ids);
            let restored_statuses = std::mem::take(saved_update_statuses);
            let mut new_state = ListState::default();
            new_state.select(Some(0));
            *model = Model::PluginDetail {
                plugin_id,
                state: new_state,
                saved_marked_ids: restored_marks,
                saved_update_statuses: restored_statuses,
            };
        }
        Model::ComponentList {
            plugin_id,
            saved_marked_ids,
            saved_update_statuses,
            ..
        } => {
            // ComponentList → ComponentTypes へ戻る（マーク状態を復元）
            let plugin_id = plugin_id.clone();
            let restored_marks = std::mem::take(saved_marked_ids);
            let restored_statuses = std::mem::take(saved_update_statuses);
            let mut new_state = ListState::default();
            new_state.select(Some(0));
            *model = Model::ComponentTypes {
                plugin_id,
                selected_kind_idx: 0,
                state: new_state,
                saved_marked_ids: restored_marks,
                saved_update_statuses: restored_statuses,
            };
        }
    }
}

/// 現在の画面のリスト長を取得
fn list_len(model: &Model, data: &DataStore, filter_text: &str) -> usize {
    match model {
        Model::PluginList { .. } => filter_plugins(&data.plugins, filter_text).len(),
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
fn update_selected_id(model: &mut Model, data: &DataStore, filter_text: &str) {
    if let Model::PluginList {
        selected_id, state, ..
    } = model
    {
        if let Some(idx) = state.selected() {
            let filtered = filter_plugins(&data.plugins, filter_text);
            *selected_id = filtered.get(idx).map(|p| p.name.clone());
        }
    }
}

#[cfg(test)]
#[path = "update_test.rs"]
mod update_test;
