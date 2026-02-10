//! Marketplaces タブの update（状態遷移ロジック）

use super::actions;
use super::model::{AddFormModel, DetailAction, Model, Msg, OperationStatus};
use crate::marketplace::normalize_name;
use crate::repo;
use crate::tui::manager::core::{DataStore, MarketplaceItem};
use ratatui::widgets::ListState;

/// update() の戻り値
pub struct UpdateEffect {
    pub should_focus_filter: bool,
    pub phase2_msg: Option<Msg>,
}

impl UpdateEffect {
    pub fn none() -> Self {
        Self {
            should_focus_filter: false,
            phase2_msg: None,
        }
    }

    fn phase2(msg: Msg) -> Self {
        Self {
            should_focus_filter: false,
            phase2_msg: Some(msg),
        }
    }
}

/// メッセージに応じて状態を更新
pub fn update(model: &mut Model, msg: Msg, data: &mut DataStore) -> UpdateEffect {
    match msg {
        Msg::Up => {
            clear_error(model);
            select_prev(model, data);
            UpdateEffect::none()
        }
        Msg::Down => {
            clear_error(model);
            select_next(model, data);
            UpdateEffect::none()
        }
        Msg::Enter => {
            clear_error(model);
            enter(model, data)
        }
        Msg::Back => {
            clear_error(model);
            back(model, data);
            UpdateEffect::none()
        }
        Msg::FormInput(c) => {
            form_input(model, c);
            UpdateEffect::none()
        }
        Msg::FormBackspace => {
            form_backspace(model);
            UpdateEffect::none()
        }
        Msg::UpdateMarket => update_market(model),
        Msg::UpdateAll => update_all(model, data),
        Msg::ExecuteUpdate => execute_update(model, data),
        Msg::ExecuteRemove => execute_remove(model, data),
    }
}

/// error_message をクリア
fn clear_error(model: &mut Model) {
    match model {
        Model::MarketList { error_message, .. } => *error_message = None,
        Model::MarketDetail { error_message, .. } => *error_message = None,
        _ => {}
    }
}

/// マーケットプレイスリストの長さ（+ Add new を含む）
fn market_list_len(data: &DataStore) -> usize {
    data.marketplaces.len() + 1
}

/// 選択を上に移動
fn select_prev(model: &mut Model, data: &DataStore) {
    match model {
        Model::MarketList {
            state, selected_id, ..
        } => {
            let current = state.selected().unwrap_or(0);
            if current == 0 {
                return;
            }
            let prev = current - 1;
            state.select(Some(prev));
            *selected_id = data.marketplaces.get(prev).map(|m| m.name.clone());
        }
        Model::MarketDetail { state, .. } => {
            let current = state.selected().unwrap_or(0);
            if current > 0 {
                state.select(Some(current - 1));
            }
        }
        Model::PluginList {
            selected_idx,
            state,
            plugins,
            ..
        } => {
            if plugins.is_empty() {
                return;
            }
            if *selected_idx > 0 {
                *selected_idx -= 1;
                state.select(Some(*selected_idx));
            }
        }
        Model::AddForm(_) => {}
    }
}

/// 選択を下に移動
fn select_next(model: &mut Model, data: &DataStore) {
    match model {
        Model::MarketList {
            state, selected_id, ..
        } => {
            let len = market_list_len(data);
            let current = state.selected().unwrap_or(0);
            let next = (current + 1).min(len - 1);
            state.select(Some(next));
            *selected_id = data.marketplaces.get(next).map(|m| m.name.clone());
        }
        Model::MarketDetail { state, .. } => {
            let len = DetailAction::all().len();
            let current = state.selected().unwrap_or(0);
            let next = (current + 1).min(len - 1);
            state.select(Some(next));
        }
        Model::PluginList {
            selected_idx,
            state,
            plugins,
            ..
        } => {
            if plugins.is_empty() {
                return;
            }
            let next = (*selected_idx + 1).min(plugins.len() - 1);
            *selected_idx = next;
            state.select(Some(next));
        }
        Model::AddForm(_) => {}
    }
}

/// Enter 処理
fn enter(model: &mut Model, data: &mut DataStore) -> UpdateEffect {
    match model {
        Model::MarketList {
            selected_id,
            operation_status,
            ..
        } => {
            // 二重実行防止
            if operation_status.is_some() {
                return UpdateEffect::none();
            }

            if let Some(name) = selected_id.clone() {
                // 通常項目 -> MarketDetail
                let mut state = ListState::default();
                state.select(Some(0));
                *model = Model::MarketDetail {
                    marketplace_name: name,
                    state,
                    error_message: None,
                };
            } else {
                // "+ Add new" -> AddForm
                *model = Model::AddForm(AddFormModel::Source {
                    source_input: String::new(),
                    error_message: None,
                });
            }
            UpdateEffect::none()
        }
        Model::MarketDetail {
            marketplace_name,
            state,
            ..
        } => {
            let selected = state.selected().unwrap_or(0);
            let actions_list = DetailAction::all();
            match actions_list.get(selected) {
                Some(DetailAction::Update) => {
                    let name = marketplace_name.clone();
                    let mut new_state = ListState::default();
                    new_state.select(Some(data.marketplace_index(&name).unwrap_or(0)));
                    *model = Model::MarketList {
                        selected_id: Some(name.clone()),
                        state: new_state,
                        operation_status: Some(OperationStatus::Updating(name)),
                        error_message: None,
                    };
                    UpdateEffect::phase2(Msg::ExecuteUpdate)
                }
                Some(DetailAction::Remove) => {
                    let name = marketplace_name.clone();
                    let mut new_state = ListState::default();
                    new_state.select(Some(data.marketplace_index(&name).unwrap_or(0)));
                    *model = Model::MarketList {
                        selected_id: Some(name.clone()),
                        state: new_state,
                        operation_status: Some(OperationStatus::Removing(name)),
                        error_message: None,
                    };
                    UpdateEffect::phase2(Msg::ExecuteRemove)
                }
                Some(DetailAction::ShowPlugins) => {
                    let name = marketplace_name.clone();
                    let plugins = actions::get_marketplace_plugins(&name);
                    let mut new_state = ListState::default();
                    if !plugins.is_empty() {
                        new_state.select(Some(0));
                    }
                    *model = Model::PluginList {
                        marketplace_name: name,
                        selected_idx: 0,
                        state: new_state,
                        plugins,
                    };
                    UpdateEffect::none()
                }
                Some(DetailAction::Back) => {
                    let name = marketplace_name.clone();
                    back_to_market_list(model, data, name);
                    UpdateEffect::none()
                }
                None => UpdateEffect::none(),
            }
        }
        Model::PluginList { .. } => {
            // プラグインリストでは Enter は何もしない
            UpdateEffect::none()
        }
        Model::AddForm(_) => enter_form(model, data),
    }
}

/// AddForm の Enter 処理
fn enter_form(model: &mut Model, data: &mut DataStore) -> UpdateEffect {
    match model {
        Model::AddForm(AddFormModel::Source {
            source_input,
            error_message,
        }) => {
            if source_input.is_empty() {
                *error_message = Some("Source is required".to_string());
                return UpdateEffect::none();
            }
            match repo::from_url(source_input) {
                Ok(parsed) => {
                    let source = parsed.full_name();
                    let default_name = match normalize_name(parsed.name()) {
                        Ok(name) => name,
                        Err(e) => {
                            *error_message = Some(format!("Invalid repository name: {}", e));
                            return UpdateEffect::none();
                        }
                    };
                    *model = Model::AddForm(AddFormModel::Name {
                        source,
                        name_input: String::new(),
                        default_name,
                        error_message: None,
                    });
                }
                Err(e) => {
                    *error_message = Some(format!(
                        "Invalid source. Use owner/repo or GitHub URL. ({})",
                        e
                    ));
                }
            }
            UpdateEffect::none()
        }
        Model::AddForm(AddFormModel::Name {
            source,
            name_input,
            default_name,
            error_message,
        }) => {
            let name = if name_input.is_empty() {
                default_name.clone()
            } else {
                match normalize_name(name_input) {
                    Ok(n) => n,
                    Err(e) => {
                        *error_message = Some(e);
                        return UpdateEffect::none();
                    }
                }
            };

            // 重複チェック
            if data.find_marketplace(&name).is_some() {
                *error_message = Some(format!("Marketplace '{}' already exists", name));
                return UpdateEffect::none();
            }

            let source = source.clone();
            *model = Model::AddForm(AddFormModel::Confirm {
                source,
                name,
                error_message: None,
            });
            UpdateEffect::none()
        }
        Model::AddForm(AddFormModel::Confirm { source, name, .. }) => {
            // Add は直接実行（source 情報を保持するため 2段階方式を使わない）
            let source = source.clone();
            let name = name.clone();
            execute_add_with(
                &source,
                &name,
                model,
                data,
                |s, n| actions::add_marketplace(s, n, None),
                |d| d.reload_marketplaces(),
            )
        }
        _ => UpdateEffect::none(),
    }
}

/// ExecuteAdd の実装本体（依存関数注入パターン）
fn execute_add_with(
    source: &str,
    name: &str,
    model: &mut Model,
    data: &mut DataStore,
    run_add: impl FnOnce(&str, &str) -> Result<actions::AddResult, String>,
    reload: impl FnOnce(&mut DataStore),
) -> UpdateEffect {
    let source_owned = source.to_string();
    let name_owned = name.to_string();

    match run_add(&source_owned, &name_owned) {
        Ok(_result) => {
            reload(data);
            let idx = data.marketplace_index(&name_owned).unwrap_or(0);
            let mut state = ListState::default();
            state.select(Some(idx));
            *model = Model::MarketList {
                selected_id: Some(name_owned),
                state,
                operation_status: None,
                error_message: None,
            };
        }
        Err(e) => {
            // AddForm Confirm に戻してエラーを表示
            *model = Model::AddForm(AddFormModel::Confirm {
                source: source_owned,
                name: name_owned,
                error_message: Some(e),
            });
        }
    }
    UpdateEffect::none()
}

/// Back 処理
fn back(model: &mut Model, data: &DataStore) {
    match model {
        Model::MarketList { .. } => {
            // MarketList での Back は app.rs で処理
        }
        Model::MarketDetail { .. } => {
            let name = match model {
                Model::MarketDetail {
                    marketplace_name, ..
                } => marketplace_name.clone(),
                _ => unreachable!(),
            };
            back_to_market_list(model, data, name);
        }
        Model::PluginList {
            marketplace_name, ..
        } => {
            let name = marketplace_name.clone();
            let mut state = ListState::default();
            state.select(Some(0));
            *model = Model::MarketDetail {
                marketplace_name: name,
                state,
                error_message: None,
            };
        }
        Model::AddForm(_) => {
            // AddForm -> MarketList
            let selected_id = data.marketplaces.first().map(|m| m.name.clone());
            let mut state = ListState::default();
            state.select(Some(0));
            *model = Model::MarketList {
                selected_id,
                state,
                operation_status: None,
                error_message: None,
            };
        }
    }
}

/// MarketList に戻る
fn back_to_market_list(model: &mut Model, data: &DataStore, marketplace_name: String) {
    let idx = data.marketplace_index(&marketplace_name).unwrap_or(0);
    let mut state = ListState::default();
    state.select(Some(idx));
    *model = Model::MarketList {
        selected_id: Some(marketplace_name),
        state,
        operation_status: None,
        error_message: None,
    };
}

/// FormInput 処理
fn form_input(model: &mut Model, c: char) {
    match model {
        Model::AddForm(AddFormModel::Source {
            source_input,
            error_message,
        }) => {
            *error_message = None;
            source_input.push(c);
        }
        Model::AddForm(AddFormModel::Name {
            name_input,
            error_message,
            ..
        }) => {
            *error_message = None;
            name_input.push(c);
        }
        _ => {}
    }
}

/// FormBackspace 処理
fn form_backspace(model: &mut Model) {
    match model {
        Model::AddForm(AddFormModel::Source { source_input, .. }) => {
            source_input.pop();
        }
        Model::AddForm(AddFormModel::Name { name_input, .. }) => {
            name_input.pop();
        }
        _ => {}
    }
}

/// 'u' キー: 選択中のマーケットプレイスを更新
fn update_market(model: &mut Model) -> UpdateEffect {
    if let Model::MarketList {
        selected_id,
        operation_status,
        error_message,
        ..
    } = model
    {
        if operation_status.is_some() {
            return UpdateEffect::none();
        }
        if let Some(name) = selected_id.clone() {
            *error_message = None;
            *operation_status = Some(OperationStatus::Updating(name));
            return UpdateEffect::phase2(Msg::ExecuteUpdate);
        }
    }
    UpdateEffect::none()
}

/// 'U' キー: 全マーケットプレイスを更新
fn update_all(model: &mut Model, data: &DataStore) -> UpdateEffect {
    if let Model::MarketList {
        operation_status,
        error_message,
        ..
    } = model
    {
        if operation_status.is_some() {
            return UpdateEffect::none();
        }
        if data.marketplaces.is_empty() {
            return UpdateEffect::none();
        }
        *error_message = None;
        *operation_status = Some(OperationStatus::UpdatingAll);
        return UpdateEffect::phase2(Msg::ExecuteUpdate);
    }
    UpdateEffect::none()
}

/// Phase 2: ExecuteUpdate
fn execute_update(model: &mut Model, data: &mut DataStore) -> UpdateEffect {
    execute_update_with(
        model,
        data,
        |name| actions::update_marketplace(name),
        |_name| actions::update_all_marketplaces(),
        |d| d.reload_marketplaces(),
    )
}

/// ExecuteUpdate の実装本体
fn execute_update_with(
    model: &mut Model,
    data: &mut DataStore,
    run_update: impl FnOnce(&str) -> Result<MarketplaceItem, String>,
    run_update_all: impl FnOnce(&str) -> Vec<(String, Result<MarketplaceItem, String>)>,
    reload: impl FnOnce(&mut DataStore),
) -> UpdateEffect {
    if let Model::MarketList {
        operation_status,
        error_message,
        selected_id,
        state,
    } = model
    {
        match operation_status.take() {
            Some(OperationStatus::Updating(name)) => match run_update(&name) {
                Ok(_) => {
                    reload(data);
                    let idx = data.marketplace_index(&name).unwrap_or(0);
                    state.select(Some(idx));
                    *selected_id = Some(name);
                    *error_message = None;
                }
                Err(e) => {
                    *error_message = Some(format!("Failed to update '{}': {}", name, e));
                }
            },
            Some(OperationStatus::UpdatingAll) => {
                let results = run_update_all("");
                let mut errors = Vec::new();
                for (name, result) in &results {
                    if let Err(e) = result {
                        errors.push(format!("{}: {}", name, e));
                    }
                }
                reload(data);
                if errors.is_empty() {
                    *error_message = None;
                } else {
                    *error_message = Some(format!("Failed to update: {}", errors.join(", ")));
                }
                // 選択状態を維持
                if let Some(id) = selected_id.as_ref() {
                    let idx = data.marketplace_index(id).unwrap_or(0);
                    state.select(Some(idx));
                }
            }
            other => {
                // 元に戻す
                *operation_status = other;
            }
        }
    }
    UpdateEffect::none()
}

/// Phase 2: ExecuteRemove
fn execute_remove(model: &mut Model, data: &mut DataStore) -> UpdateEffect {
    execute_remove_with(
        model,
        data,
        |name| actions::remove_marketplace(name),
        |d| d.reload_marketplaces(),
    )
}

/// ExecuteRemove の実装本体
fn execute_remove_with(
    model: &mut Model,
    data: &mut DataStore,
    run_remove: impl FnOnce(&str) -> Result<(), String>,
    reload: impl FnOnce(&mut DataStore),
) -> UpdateEffect {
    if let Model::MarketList {
        operation_status,
        error_message,
        selected_id,
        state,
    } = model
    {
        match operation_status.take() {
            Some(OperationStatus::Removing(name)) => {
                match run_remove(&name) {
                    Ok(()) => {
                        reload(data);
                        // 先頭にクランプ
                        let new_selected = data.marketplaces.first().map(|m| m.name.clone());
                        *selected_id = new_selected;
                        state.select(Some(0));
                        *error_message = None;
                    }
                    Err(e) => {
                        *error_message = Some(format!("Failed to remove '{}': {}", name, e));
                    }
                }
            }
            other => {
                // Removing 以外の状態だった場合は元に戻す
                *operation_status = other;
            }
        }
    }
    UpdateEffect::none()
}

#[cfg(test)]
#[path = "update_test.rs"]
mod update_test;
