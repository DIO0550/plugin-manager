//! Marketplaces タブの update（状態遷移ロジック）

use super::actions;
use super::model::{
    AddFormModel, BrowsePlugin, DetailAction, MarketplacesScreenModel, Msg, OperationStatus,
};
use crate::marketplace::normalize_name;
use crate::repo;
use crate::tui::manager::core::{DataStore, MarketplaceItem, SelectionState};
use ratatui::widgets::ListState;
use std::collections::HashSet;

/// BrowsePlugins guard でキャッシュ無しの際に表示するメッセージ。
const NO_CACHE_MESSAGE: &str = "No cached data — try update";

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
///
/// # Arguments
///
/// * `model` - Marketplaces tab model to mutate.
/// * `msg` - Incoming message to apply.
/// * `data` - Shared data store for marketplaces and plugins.
pub fn update(model: &mut MarketplacesScreenModel, msg: Msg, data: &mut DataStore) -> UpdateEffect {
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
        Msg::ExecuteAdd => execute_add(model, data),
        Msg::ToggleSelect => toggle_select(model),
        Msg::StartInstall => start_install(model),
        Msg::ConfirmTargets => confirm_targets(model),
        Msg::ConfirmScope => confirm_scope(model),
        Msg::ExecuteInstall => execute_install(model, data),
        Msg::BackToPluginBrowse => back_to_plugin_browse(model, data),
    }
}

/// error_message をクリア
fn clear_error(model: &mut MarketplacesScreenModel) {
    match model {
        MarketplacesScreenModel::MarketList { error_message, .. } => *error_message = None,
        MarketplacesScreenModel::MarketDetail { error_message, .. } => *error_message = None,
        _ => {}
    }
}

/// マーケットプレイスリストの長さ（+ Add new を含む）
fn market_list_len(data: &DataStore) -> usize {
    data.marketplaces.len() + 1
}

fn market_selection(
    selected_id: Option<String>,
    selected_index: Option<usize>,
) -> SelectionState<String> {
    SelectionState::new(selected_id, selected_index)
}

/// 選択を上に移動
fn select_prev(model: &mut MarketplacesScreenModel, data: &DataStore) {
    match model {
        MarketplacesScreenModel::MarketList { selection, .. } => {
            let current = selection.selected_index().unwrap_or(0);
            if current == 0 {
                return;
            }
            let prev = current - 1;
            selection.set(
                data.marketplaces.get(prev).map(|m| m.name.clone()),
                Some(prev),
            );
        }
        MarketplacesScreenModel::MarketDetail { state, .. } => {
            let current = state.selected().unwrap_or(0);
            if current > 0 {
                state.select(Some(current - 1));
            }
        }
        MarketplacesScreenModel::PluginList {
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
        MarketplacesScreenModel::PluginBrowse {
            highlighted_idx,
            state,
            plugins,
            ..
        } => {
            if plugins.is_empty() {
                return;
            }
            if *highlighted_idx > 0 {
                *highlighted_idx -= 1;
                state.select(Some(*highlighted_idx));
            }
        }
        MarketplacesScreenModel::TargetSelect {
            highlighted_idx,
            state,
            targets,
            ..
        } => {
            if targets.is_empty() {
                return;
            }
            if *highlighted_idx > 0 {
                *highlighted_idx -= 1;
                state.select(Some(*highlighted_idx));
            }
        }
        MarketplacesScreenModel::ScopeSelect {
            highlighted_idx,
            state,
            ..
        } if *highlighted_idx > 0 => {
            *highlighted_idx -= 1;
            state.select(Some(*highlighted_idx));
        }
        _ => {}
    }
}

/// 選択を下に移動
fn select_next(model: &mut MarketplacesScreenModel, data: &DataStore) {
    match model {
        MarketplacesScreenModel::MarketList { selection, .. } => {
            let len = market_list_len(data);
            let current = selection.selected_index().unwrap_or(0);
            let next = (current + 1).min(len - 1);
            selection.set(
                data.marketplaces.get(next).map(|m| m.name.clone()),
                Some(next),
            );
        }
        MarketplacesScreenModel::MarketDetail { state, .. } => {
            let len = DetailAction::all().len();
            let current = state.selected().unwrap_or(0);
            let next = (current + 1).min(len - 1);
            state.select(Some(next));
        }
        MarketplacesScreenModel::PluginList {
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
        MarketplacesScreenModel::PluginBrowse {
            highlighted_idx,
            state,
            plugins,
            ..
        } => {
            if plugins.is_empty() {
                return;
            }
            let next = (*highlighted_idx + 1).min(plugins.len() - 1);
            *highlighted_idx = next;
            state.select(Some(next));
        }
        MarketplacesScreenModel::TargetSelect {
            highlighted_idx,
            state,
            targets,
            ..
        } => {
            if targets.is_empty() {
                return;
            }
            let next = (*highlighted_idx + 1).min(targets.len() - 1);
            *highlighted_idx = next;
            state.select(Some(next));
        }
        MarketplacesScreenModel::ScopeSelect {
            highlighted_idx,
            state,
            ..
        } => {
            // Only 2 options: Personal(0), Project(1)
            let next = (*highlighted_idx + 1).min(1);
            *highlighted_idx = next;
            state.select(Some(next));
        }
        _ => {}
    }
}

/// Enter 処理
fn enter(model: &mut MarketplacesScreenModel, data: &mut DataStore) -> UpdateEffect {
    match model {
        MarketplacesScreenModel::MarketList {
            selection,
            operation_status,
            ..
        } => {
            // 二重実行防止
            if operation_status.is_some() {
                return UpdateEffect::none();
            }

            if let Some(name) = selection.selected_id().cloned() {
                // 通常項目 -> MarketDetail
                let mut state = ListState::default();
                state.select(Some(0));
                *model = MarketplacesScreenModel::MarketDetail {
                    marketplace_name: name,
                    state,
                    error_message: None,
                    browse_plugins: None,
                    browse_selected: None,
                };
            } else {
                // "+ Add new" -> AddForm
                *model = MarketplacesScreenModel::AddForm(AddFormModel::Source {
                    source_input: String::new(),
                    error_message: None,
                });
            }
            UpdateEffect::none()
        }
        MarketplacesScreenModel::MarketDetail {
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
                    *model = MarketplacesScreenModel::MarketList {
                        selection: market_selection(Some(name.clone()), new_state.selected()),
                        operation_status: Some(OperationStatus::Updating(name)),
                        error_message: None,
                        pending_add_source: None,
                    };
                    UpdateEffect::phase2(Msg::ExecuteUpdate)
                }
                Some(DetailAction::Remove) => {
                    let name = marketplace_name.clone();
                    let mut new_state = ListState::default();
                    new_state.select(Some(data.marketplace_index(&name).unwrap_or(0)));
                    *model = MarketplacesScreenModel::MarketList {
                        selection: market_selection(Some(name.clone()), new_state.selected()),
                        operation_status: Some(OperationStatus::Removing(name)),
                        error_message: None,
                        pending_add_source: None,
                    };
                    UpdateEffect::phase2(Msg::ExecuteRemove)
                }
                Some(DetailAction::BrowsePlugins) => {
                    let name = marketplace_name.clone();
                    // Guard: cache must exist (plugin_count.is_some())
                    let has_cache = data
                        .find_marketplace(&name)
                        .map(|m| m.plugin_count.is_some())
                        .unwrap_or(false);
                    if !has_cache {
                        if let MarketplacesScreenModel::MarketDetail { error_message, .. } = model {
                            *error_message = Some(NO_CACHE_MESSAGE.to_string());
                        }
                        return UpdateEffect::none();
                    }

                    // Extract preserved browse state from MarketDetail
                    let (preserved_plugins, preserved_selected) = match model {
                        MarketplacesScreenModel::MarketDetail {
                            browse_plugins,
                            browse_selected,
                            ..
                        } => (browse_plugins.take(), browse_selected.take()),
                        _ => (None, None),
                    };

                    let (plugins, selected_plugins) =
                        if let (Some(bp), Some(bs)) = (preserved_plugins, preserved_selected) {
                            (bp, bs)
                        } else {
                            let bp = actions::get_browse_plugins(&name, &data.plugins);
                            (bp, HashSet::new())
                        };

                    let mut new_state = ListState::default();
                    if !plugins.is_empty() {
                        new_state.select(Some(0));
                    }
                    *model = MarketplacesScreenModel::PluginBrowse {
                        marketplace_name: name,
                        plugins,
                        selected_plugins,
                        highlighted_idx: 0,
                        state: new_state,
                    };
                    UpdateEffect::none()
                }
                Some(DetailAction::ShowPlugins) => {
                    let name = marketplace_name.clone();
                    let plugins = actions::get_marketplace_plugins(&name);
                    let mut new_state = ListState::default();
                    if !plugins.is_empty() {
                        new_state.select(Some(0));
                    }
                    *model = MarketplacesScreenModel::PluginList {
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
        MarketplacesScreenModel::PluginList { .. } => {
            // プラグインリストでは Enter は何もしない
            UpdateEffect::none()
        }
        MarketplacesScreenModel::AddForm(_) => enter_form(model, data),
        _ => UpdateEffect::none(),
    }
}

/// AddForm の Enter 処理
fn enter_form(model: &mut MarketplacesScreenModel, data: &mut DataStore) -> UpdateEffect {
    match model {
        MarketplacesScreenModel::AddForm(AddFormModel::Source {
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
                    *model = MarketplacesScreenModel::AddForm(AddFormModel::Name {
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
        MarketplacesScreenModel::AddForm(AddFormModel::Name {
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
            *model = MarketplacesScreenModel::AddForm(AddFormModel::Confirm {
                source,
                name,
                error_message: None,
            });
            UpdateEffect::none()
        }
        MarketplacesScreenModel::AddForm(AddFormModel::Confirm { source, name, .. }) => {
            let source = source.clone();
            let name = name.clone();
            execute_add_phase1(model, data, source, name)
        }
        _ => UpdateEffect::none(),
    }
}

/// Phase 1: AddForm::Confirm → MarketList(Adding) への即時遷移。
///
/// 副作用は伴わない。実 fetch は phase2 (`execute_add`) で行う。
fn execute_add_phase1(
    model: &mut MarketplacesScreenModel,
    data: &DataStore,
    source: String,
    name: String,
) -> UpdateEffect {
    let idx = data
        .marketplace_index(&name)
        .unwrap_or(data.marketplaces.len());
    *model = MarketplacesScreenModel::MarketList {
        selection: market_selection(Some(name.clone()), Some(idx)),
        operation_status: Some(OperationStatus::Adding(name)),
        error_message: None,
        pending_add_source: Some(source),
    };
    UpdateEffect::phase2(Msg::ExecuteAdd)
}

/// Phase 2: ExecuteAdd dispatcher（本番の依存を注入）
fn execute_add(model: &mut MarketplacesScreenModel, data: &mut DataStore) -> UpdateEffect {
    execute_add_with(
        model,
        data,
        |source, name| actions::add_marketplace(source, name, None),
        |d| d.reload_marketplaces(),
        |d, name| actions::get_browse_plugins(name, &d.plugins),
    )
}

/// Phase 2 本体（依存関数注入パターン）。
///
/// MarketList の operation_status が `Adding(name)` で `pending_add_source` が
/// `Some` であることを前提とする。それ以外の状態では no-op。
///
/// 成功時はそのままプラグインインストールフローに進めるよう
/// `PluginBrowse` に遷移する。失敗時は MarketList に留まり error_message を
/// セットする。
pub(super) fn execute_add_with(
    model: &mut MarketplacesScreenModel,
    data: &mut DataStore,
    run_add: impl FnOnce(&str, &str) -> Result<actions::MarketplaceAddOutcome, String>,
    reload: impl FnOnce(&mut DataStore),
    fetch_browse_plugins: impl FnOnce(&DataStore, &str) -> Vec<BrowsePlugin>,
) -> UpdateEffect {
    let (name, source) = match take_add_request(model) {
        Some(pair) => pair,
        None => return UpdateEffect::none(),
    };

    match run_add(&source, &name) {
        Ok(_) => {
            reload(data);
            let plugins = fetch_browse_plugins(data, &name);
            let mut state = ListState::default();
            if !plugins.is_empty() {
                state.select(Some(0));
            }
            *model = MarketplacesScreenModel::PluginBrowse {
                marketplace_name: name,
                plugins,
                selected_plugins: HashSet::new(),
                highlighted_idx: 0,
                state,
            };
        }
        Err(e) => {
            if let MarketplacesScreenModel::MarketList {
                selection,
                error_message,
                ..
            } = model
            {
                let idx = data.marketplace_index(&name);
                selection.set(idx.map(|_| name.clone()), idx);
                *error_message = Some(e);
            }
        }
    }
    UpdateEffect::none()
}

/// MarketList(Adding) から (name, source) を取り出す。Adding 以外なら状態を戻し
/// `None` を返す。pending_add_source が欠落していた場合は error_message を
/// セットして `None` を返す。
fn take_add_request(model: &mut MarketplacesScreenModel) -> Option<(String, String)> {
    let MarketplacesScreenModel::MarketList {
        operation_status,
        error_message,
        pending_add_source,
        ..
    } = model
    else {
        return None;
    };

    let name = match operation_status.take() {
        Some(OperationStatus::Adding(name)) => name,
        other => {
            *operation_status = other;
            return None;
        }
    };
    let Some(source) = pending_add_source.take() else {
        *error_message = Some("Internal error: missing pending source".to_string());
        return None;
    };
    Some((name, source))
}

/// Back 処理
fn back(model: &mut MarketplacesScreenModel, data: &DataStore) {
    match model {
        MarketplacesScreenModel::MarketList { .. } => {
            // MarketList での Back は app.rs で処理
        }
        MarketplacesScreenModel::MarketDetail { .. } => {
            let name = match model {
                MarketplacesScreenModel::MarketDetail {
                    marketplace_name, ..
                } => marketplace_name.clone(),
                _ => unreachable!(),
            };
            back_to_market_list(model, data, name);
        }
        MarketplacesScreenModel::PluginList {
            marketplace_name, ..
        } => {
            let name = marketplace_name.clone();
            let mut state = ListState::default();
            state.select(Some(0));
            *model = MarketplacesScreenModel::MarketDetail {
                marketplace_name: name,
                state,
                error_message: None,
                browse_plugins: None,
                browse_selected: None,
            };
        }
        MarketplacesScreenModel::AddForm(_) => {
            // AddForm -> MarketList
            let selected_id = data.marketplaces.first().map(|m| m.name.clone());
            let mut state = ListState::default();
            state.select(Some(0));
            *model = MarketplacesScreenModel::MarketList {
                selection: market_selection(selected_id, state.selected()),
                operation_status: None,
                error_message: None,
                pending_add_source: None,
            };
        }
        MarketplacesScreenModel::PluginBrowse { .. } => {
            // Take ownership via replace
            let old = std::mem::replace(
                model,
                MarketplacesScreenModel::MarketList {
                    selection: SelectionState::default(),
                    operation_status: None,
                    error_message: None,
                    pending_add_source: None,
                },
            );
            if let MarketplacesScreenModel::PluginBrowse {
                marketplace_name,
                plugins,
                selected_plugins,
                ..
            } = old
            {
                let mut state = ListState::default();
                state.select(Some(0));
                *model = MarketplacesScreenModel::MarketDetail {
                    marketplace_name,
                    state,
                    error_message: None,
                    browse_plugins: Some(plugins),
                    browse_selected: Some(selected_plugins),
                };
            }
        }
        MarketplacesScreenModel::TargetSelect { .. } => {
            // Take ownership via replace
            let old = std::mem::replace(
                model,
                MarketplacesScreenModel::MarketList {
                    selection: SelectionState::default(),
                    operation_status: None,
                    error_message: None,
                    pending_add_source: None,
                },
            );
            if let MarketplacesScreenModel::TargetSelect {
                marketplace_name,
                plugins,
                selected_plugins,
                ..
            } = old
            {
                let mut state = ListState::default();
                if !plugins.is_empty() {
                    state.select(Some(0));
                }
                *model = MarketplacesScreenModel::PluginBrowse {
                    marketplace_name,
                    plugins,
                    selected_plugins,
                    highlighted_idx: 0,
                    state,
                };
            }
        }
        MarketplacesScreenModel::ScopeSelect { .. } => {
            // Take ownership via replace
            let old = std::mem::replace(
                model,
                MarketplacesScreenModel::MarketList {
                    selection: SelectionState::default(),
                    operation_status: None,
                    error_message: None,
                    pending_add_source: None,
                },
            );
            if let MarketplacesScreenModel::ScopeSelect {
                marketplace_name,
                plugins,
                selected_plugins,
                target_names,
                ..
            } = old
            {
                // Rebuild targets from all_targets(), marking those in target_names as selected
                let targets: Vec<(String, String, bool)> = crate::target::all_targets()
                    .iter()
                    .map(|t| {
                        let name = t.name().to_string();
                        let selected = target_names.contains(&name);
                        (name, t.display_name().to_string(), selected)
                    })
                    .collect();

                let mut state = ListState::default();
                if !targets.is_empty() {
                    state.select(Some(0));
                }

                *model = MarketplacesScreenModel::TargetSelect {
                    marketplace_name,
                    plugins,
                    selected_plugins,
                    targets,
                    highlighted_idx: 0,
                    state,
                };
            }
        }
        _ => {}
    }
}

/// MarketList に戻る
fn back_to_market_list(
    model: &mut MarketplacesScreenModel,
    data: &DataStore,
    marketplace_name: String,
) {
    let idx = data.marketplace_index(&marketplace_name).unwrap_or(0);
    let selected_id = data.marketplaces.get(idx).map(|m| m.name.clone());
    let mut state = ListState::default();
    state.select(Some(idx));
    *model = MarketplacesScreenModel::MarketList {
        selection: market_selection(selected_id, state.selected()),
        operation_status: None,
        error_message: None,
        pending_add_source: None,
    };
}

/// StartInstall: PluginBrowse -> TargetSelect
fn start_install(model: &mut MarketplacesScreenModel) -> UpdateEffect {
    if let MarketplacesScreenModel::PluginBrowse {
        plugins,
        selected_plugins,
        highlighted_idx,
        ..
    } = model
    {
        if plugins.is_empty() {
            return UpdateEffect::none();
        }
        if selected_plugins.is_empty() {
            let idx = (*highlighted_idx).min(plugins.len() - 1);
            match plugins.get(idx) {
                Some(plugin) => {
                    selected_plugins.insert(plugin.name.clone());
                }
                None => return UpdateEffect::none(),
            }
        }
    } else {
        return UpdateEffect::none();
    }

    // Take ownership of fields
    let (marketplace_name, plugins, selected_plugins) = match std::mem::replace(
        model,
        MarketplacesScreenModel::MarketList {
            selection: SelectionState::default(),
            operation_status: None,
            error_message: None,
            pending_add_source: None,
        },
    ) {
        MarketplacesScreenModel::PluginBrowse {
            marketplace_name,
            plugins,
            selected_plugins,
            ..
        } => (marketplace_name, plugins, selected_plugins),
        _ => unreachable!(),
    };

    let targets: Vec<(String, String, bool)> = crate::target::all_targets()
        .iter()
        .map(|t| (t.name().to_string(), t.display_name().to_string(), false))
        .collect();

    let mut state = ListState::default();
    if !targets.is_empty() {
        state.select(Some(0));
    }

    *model = MarketplacesScreenModel::TargetSelect {
        marketplace_name,
        plugins,
        selected_plugins,
        targets,
        highlighted_idx: 0,
        state,
    };
    UpdateEffect::none()
}

/// ConfirmScope: ScopeSelect -> Installing + Phase2
fn confirm_scope(model: &mut MarketplacesScreenModel) -> UpdateEffect {
    if !matches!(model, MarketplacesScreenModel::ScopeSelect { .. }) {
        return UpdateEffect::none();
    }

    let old = std::mem::replace(
        model,
        MarketplacesScreenModel::MarketList {
            selection: SelectionState::default(),
            operation_status: None,
            error_message: None,
            pending_add_source: None,
        },
    );

    if let MarketplacesScreenModel::ScopeSelect {
        marketplace_name,
        plugins,
        selected_plugins,
        target_names,
        highlighted_idx,
        ..
    } = old
    {
        let scope = if highlighted_idx == 0 {
            crate::component::Scope::Personal
        } else {
            crate::component::Scope::Project
        };

        // Collect plugin_names in plugins order for deterministic ordering
        let plugin_names: Vec<String> = plugins
            .iter()
            .filter(|p| selected_plugins.contains(&p.name))
            .map(|p| p.name.clone())
            .collect();

        let total = plugin_names.len();

        *model = MarketplacesScreenModel::Installing {
            marketplace_name,
            plugins,
            plugin_names,
            target_names,
            scope,
            current_idx: 0,
            total,
        };

        return UpdateEffect::phase2(Msg::ExecuteInstall);
    }

    UpdateEffect::none()
}

/// BackToPluginBrowse: InstallOutcome -> PluginBrowse (refresh)
fn back_to_plugin_browse(model: &mut MarketplacesScreenModel, data: &DataStore) -> UpdateEffect {
    if !matches!(model, MarketplacesScreenModel::InstallOutcome { .. }) {
        return UpdateEffect::none();
    }

    let old = std::mem::replace(
        model,
        MarketplacesScreenModel::MarketList {
            selection: SelectionState::default(),
            operation_status: None,
            error_message: None,
            pending_add_source: None,
        },
    );

    if let MarketplacesScreenModel::InstallOutcome {
        marketplace_name, ..
    } = old
    {
        let plugins = actions::get_browse_plugins(&marketplace_name, &data.plugins);
        let mut state = ListState::default();
        if !plugins.is_empty() {
            state.select(Some(0));
        }

        *model = MarketplacesScreenModel::PluginBrowse {
            marketplace_name,
            plugins,
            selected_plugins: HashSet::new(),
            highlighted_idx: 0,
            state,
        };
    }

    UpdateEffect::none()
}

/// Phase 2: ExecuteInstall (real dependencies)
fn execute_install(model: &mut MarketplacesScreenModel, data: &mut DataStore) -> UpdateEffect {
    execute_install_with(model, data, actions::install_plugins, |d| d.reload())
}

/// ExecuteInstall の実装本体（依存関数注入パターン）
///
/// # Arguments
///
/// * `model` - Marketplaces tab model to mutate.
/// * `data` - Shared data store for plugins and marketplaces.
/// * `run_install` - Injected install routine (real or test double).
/// * `reload` - Injected data store reload routine.
pub(super) fn execute_install_with(
    model: &mut MarketplacesScreenModel,
    data: &mut DataStore,
    run_install: impl FnOnce(
        &str,
        &[String],
        &[String],
        crate::component::Scope,
    ) -> super::model::InstallSummary,
    reload: impl FnOnce(&mut DataStore) -> std::io::Result<()>,
) -> UpdateEffect {
    if !matches!(model, MarketplacesScreenModel::Installing { .. }) {
        return UpdateEffect::none();
    }

    let old = std::mem::replace(
        model,
        MarketplacesScreenModel::MarketList {
            selection: SelectionState::default(),
            operation_status: None,
            error_message: None,
            pending_add_source: None,
        },
    );

    if let MarketplacesScreenModel::Installing {
        marketplace_name,
        plugins,
        plugin_names,
        target_names,
        scope,
        ..
    } = old
    {
        let summary = run_install(&marketplace_name, &plugin_names, &target_names, scope);

        if let Err(e) = reload(data) {
            data.last_error = Some(format!("Failed to reload: {}", e));
        }

        *model = MarketplacesScreenModel::InstallOutcome {
            marketplace_name,
            plugins,
            summary,
        };
    }

    UpdateEffect::none()
}

/// ConfirmTargets: TargetSelect -> ScopeSelect
fn confirm_targets(model: &mut MarketplacesScreenModel) -> UpdateEffect {
    if let MarketplacesScreenModel::TargetSelect {
        targets,
        highlighted_idx,
        ..
    } = model
    {
        if targets.is_empty() {
            return UpdateEffect::none();
        }
        if !targets.iter().any(|(_, _, sel)| *sel) {
            let idx = (*highlighted_idx).min(targets.len() - 1);
            match targets.get_mut(idx) {
                Some(target) => target.2 = true,
                None => return UpdateEffect::none(),
            }
        }
    } else {
        return UpdateEffect::none();
    }

    let old = std::mem::replace(
        model,
        MarketplacesScreenModel::MarketList {
            selection: SelectionState::default(),
            operation_status: None,
            error_message: None,
            pending_add_source: None,
        },
    );

    if let MarketplacesScreenModel::TargetSelect {
        marketplace_name,
        plugins,
        selected_plugins,
        targets,
        ..
    } = old
    {
        let target_names: Vec<String> = targets
            .iter()
            .filter(|(_, _, sel)| *sel)
            .map(|(name, _, _)| name.clone())
            .collect();

        let mut state = ListState::default();
        state.select(Some(0));

        *model = MarketplacesScreenModel::ScopeSelect {
            marketplace_name,
            plugins,
            selected_plugins,
            target_names,
            highlighted_idx: 0,
            state,
        };
    }
    UpdateEffect::none()
}

/// ToggleSelect: プラグインまたはターゲットの選択をトグル
fn toggle_select(model: &mut MarketplacesScreenModel) -> UpdateEffect {
    match model {
        MarketplacesScreenModel::PluginBrowse {
            plugins,
            selected_plugins,
            highlighted_idx,
            ..
        } => {
            if plugins.is_empty() {
                return UpdateEffect::none();
            }
            if let Some(plugin) = plugins.get(*highlighted_idx) {
                let name = plugin.name.clone();
                if selected_plugins.contains(&name) {
                    selected_plugins.remove(&name);
                } else {
                    selected_plugins.insert(name);
                }
            }
        }
        MarketplacesScreenModel::TargetSelect {
            targets,
            highlighted_idx,
            ..
        } => {
            if targets.is_empty() {
                return UpdateEffect::none();
            }
            if let Some(target) = targets.get_mut(*highlighted_idx) {
                target.2 = !target.2;
            }
        }
        _ => {}
    }
    UpdateEffect::none()
}

/// FormInput 処理
fn form_input(model: &mut MarketplacesScreenModel, c: char) {
    match model {
        MarketplacesScreenModel::AddForm(AddFormModel::Source {
            source_input,
            error_message,
        }) => {
            *error_message = None;
            source_input.push(c);
        }
        MarketplacesScreenModel::AddForm(AddFormModel::Name {
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
fn form_backspace(model: &mut MarketplacesScreenModel) {
    match model {
        MarketplacesScreenModel::AddForm(AddFormModel::Source { source_input, .. }) => {
            source_input.pop();
        }
        MarketplacesScreenModel::AddForm(AddFormModel::Name { name_input, .. }) => {
            name_input.pop();
        }
        _ => {}
    }
}

/// 'u' キー: 選択中のマーケットプレイスを更新
fn update_market(model: &mut MarketplacesScreenModel) -> UpdateEffect {
    if let MarketplacesScreenModel::MarketList {
        selection,
        operation_status,
        error_message,
        ..
    } = model
    {
        if operation_status.is_some() {
            return UpdateEffect::none();
        }
        if let Some(name) = selection.selected_id().cloned() {
            *error_message = None;
            *operation_status = Some(OperationStatus::Updating(name));
            return UpdateEffect::phase2(Msg::ExecuteUpdate);
        }
    }
    UpdateEffect::none()
}

/// 'U' キー: 全マーケットプレイスを更新
fn update_all(model: &mut MarketplacesScreenModel, data: &DataStore) -> UpdateEffect {
    if let MarketplacesScreenModel::MarketList {
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
fn execute_update(model: &mut MarketplacesScreenModel, data: &mut DataStore) -> UpdateEffect {
    execute_update_with(
        model,
        data,
        actions::update_marketplace,
        actions::update_all_marketplaces,
        |d| d.reload_marketplaces(),
    )
}

/// ExecuteUpdate の実装本体
fn execute_update_with(
    model: &mut MarketplacesScreenModel,
    data: &mut DataStore,
    run_update: impl FnOnce(&str) -> Result<MarketplaceItem, String>,
    run_update_all: impl FnOnce() -> Vec<(String, Result<MarketplaceItem, String>)>,
    reload: impl FnOnce(&mut DataStore),
) -> UpdateEffect {
    if let MarketplacesScreenModel::MarketList {
        operation_status,
        error_message,
        selection,
        ..
    } = model
    {
        match operation_status.take() {
            Some(OperationStatus::Updating(name)) => match run_update(&name) {
                Ok(_) => {
                    reload(data);
                    let idx = data.marketplace_index(&name).unwrap_or(0);
                    selection.set(Some(name), Some(idx));
                    *error_message = None;
                }
                Err(e) => {
                    *error_message = Some(format!("Failed to update '{}': {}", name, e));
                }
            },
            Some(OperationStatus::UpdatingAll) => {
                let results = run_update_all();
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
                if let Some(id) = selection.selected_id() {
                    let idx = data.marketplace_index(id).unwrap_or(0);
                    selection.select_index(Some(idx));
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
fn execute_remove(model: &mut MarketplacesScreenModel, data: &mut DataStore) -> UpdateEffect {
    execute_remove_with(model, data, actions::remove_marketplace, |d| {
        d.reload_marketplaces()
    })
}

/// ExecuteRemove の実装本体
fn execute_remove_with(
    model: &mut MarketplacesScreenModel,
    data: &mut DataStore,
    run_remove: impl FnOnce(&str) -> Result<(), String>,
    reload: impl FnOnce(&mut DataStore),
) -> UpdateEffect {
    if let MarketplacesScreenModel::MarketList {
        operation_status,
        error_message,
        selection,
        ..
    } = model
    {
        match operation_status.take() {
            Some(OperationStatus::Removing(name)) => {
                match run_remove(&name) {
                    Ok(()) => {
                        reload(data);
                        // 先頭にクランプ
                        let new_selected = data.marketplaces.first().map(|m| m.name.clone());
                        selection.set(new_selected, Some(0));
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
