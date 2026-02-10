use super::{execute_add_with, execute_remove_with, execute_update_with, update};
use crate::tui::manager::core::{DataStore, MarketplaceItem};
use crate::tui::manager::screens::marketplaces::actions::AddResult;
use crate::tui::manager::screens::marketplaces::model::{
    AddFormModel, DetailAction, Model, Msg, OperationStatus,
};
use ratatui::widgets::ListState;

// ============================================================================
// ヘルパー関数
// ============================================================================

fn make_marketplace(name: &str) -> MarketplaceItem {
    MarketplaceItem {
        name: name.to_string(),
        source: format!("owner/{}", name),
        source_path: None,
        plugin_count: Some(3),
        last_updated: Some("2026-01-01 00:00".to_string()),
    }
}

fn make_data(names: &[&str]) -> DataStore {
    DataStore {
        plugins: vec![],
        marketplaces: names.iter().map(|n| make_marketplace(n)).collect(),
        last_error: None,
    }
}

fn make_add_result(name: &str) -> AddResult {
    AddResult {
        marketplace: make_marketplace(name),
    }
}

// ============================================================================
// Up/Down ナビゲーション
// ============================================================================

#[test]
fn down_moves_selection_in_market_list() {
    let mut data = make_data(&["mp-a", "mp-b"]);
    let mut model = Model::new(&data);

    // 初期状態: index 0 (mp-a), selected_id = Some("mp-a")
    if let Model::MarketList {
        selected_id, state, ..
    } = &model
    {
        assert_eq!(state.selected(), Some(0));
        assert_eq!(selected_id.as_deref(), Some("mp-a"));
    }

    update(&mut model, Msg::Down, &mut data);

    if let Model::MarketList {
        selected_id, state, ..
    } = &model
    {
        assert_eq!(state.selected(), Some(1));
        assert_eq!(selected_id.as_deref(), Some("mp-b"));
    } else {
        panic!("Expected MarketList");
    }
}

#[test]
fn down_past_last_marketplace_selects_add_new() {
    let mut data = make_data(&["mp-a"]);
    let mut model = Model::new(&data);

    // Move past mp-a to "+ Add new" (index 1)
    update(&mut model, Msg::Down, &mut data);

    if let Model::MarketList {
        selected_id, state, ..
    } = &model
    {
        assert_eq!(state.selected(), Some(1));
        assert_eq!(
            selected_id, &None,
            "At '+ Add new', selected_id should be None"
        );
    } else {
        panic!("Expected MarketList");
    }
}

#[test]
fn down_does_not_go_past_add_new() {
    let mut data = make_data(&["mp-a"]);
    let mut model = Model::new(&data);

    // list len = 2 (mp-a + Add new)
    update(&mut model, Msg::Down, &mut data); // index 1 (Add new)
    update(&mut model, Msg::Down, &mut data); // should stay at 1

    if let Model::MarketList { state, .. } = &model {
        assert_eq!(state.selected(), Some(1));
    } else {
        panic!("Expected MarketList");
    }
}

#[test]
fn up_does_not_go_past_zero() {
    let mut data = make_data(&["mp-a"]);
    let mut model = Model::new(&data);

    update(&mut model, Msg::Up, &mut data);

    if let Model::MarketList { state, .. } = &model {
        assert_eq!(state.selected(), Some(0));
    } else {
        panic!("Expected MarketList");
    }
}

#[test]
fn up_from_add_new_returns_to_last_marketplace() {
    let mut data = make_data(&["mp-a", "mp-b"]);
    let mut model = Model::new(&data);

    // Move to Add new (index 2)
    update(&mut model, Msg::Down, &mut data);
    update(&mut model, Msg::Down, &mut data);

    // Move back up
    update(&mut model, Msg::Up, &mut data);

    if let Model::MarketList {
        selected_id, state, ..
    } = &model
    {
        assert_eq!(state.selected(), Some(1));
        assert_eq!(selected_id.as_deref(), Some("mp-b"));
    } else {
        panic!("Expected MarketList");
    }
}

#[test]
fn down_in_detail_moves_action_selection() {
    let mut data = make_data(&["mp-a"]);
    let mut model = Model::new(&data);

    // Enter -> MarketDetail
    update(&mut model, Msg::Enter, &mut data);

    if let Model::MarketDetail { state, .. } = &model {
        assert_eq!(state.selected(), Some(0));
    }

    update(&mut model, Msg::Down, &mut data);

    if let Model::MarketDetail { state, .. } = &model {
        assert_eq!(state.selected(), Some(1));
    } else {
        panic!("Expected MarketDetail");
    }
}

#[test]
fn down_in_detail_does_not_exceed_action_count() {
    let mut data = make_data(&["mp-a"]);
    let mut model = Model::new(&data);

    // Enter -> MarketDetail
    update(&mut model, Msg::Enter, &mut data);

    let action_count = DetailAction::all().len();
    for _ in 0..action_count + 5 {
        update(&mut model, Msg::Down, &mut data);
    }

    if let Model::MarketDetail { state, .. } = &model {
        assert_eq!(state.selected(), Some(action_count - 1));
    } else {
        panic!("Expected MarketDetail");
    }
}

// ============================================================================
// Enter ナビゲーション
// ============================================================================

#[test]
fn enter_on_marketplace_transitions_to_detail() {
    let mut data = make_data(&["mp-a"]);
    let mut model = Model::new(&data);

    update(&mut model, Msg::Enter, &mut data);

    if let Model::MarketDetail {
        marketplace_name,
        state,
        ..
    } = &model
    {
        assert_eq!(marketplace_name, "mp-a");
        assert_eq!(state.selected(), Some(0));
    } else {
        panic!("Expected MarketDetail, got {:?}", model_variant(&model));
    }
}

#[test]
fn enter_on_add_new_transitions_to_add_form() {
    let mut data = make_data(&["mp-a"]);
    let mut model = Model::new(&data);

    // Move to Add new
    update(&mut model, Msg::Down, &mut data);
    update(&mut model, Msg::Enter, &mut data);

    assert!(
        matches!(model, Model::AddForm(AddFormModel::Source { .. })),
        "Expected AddForm Source"
    );
}

#[test]
fn enter_on_empty_list_transitions_to_add_form() {
    let mut data = make_data(&[]);
    let mut model = Model::new(&data);

    // With 0 marketplaces, index 0 is Add new (selected_id = None)
    update(&mut model, Msg::Enter, &mut data);

    assert!(
        matches!(model, Model::AddForm(AddFormModel::Source { .. })),
        "Expected AddForm Source on empty list"
    );
}

#[test]
fn enter_ignored_when_operation_in_progress() {
    let mut data = make_data(&["mp-a"]);
    let mut model = Model::new(&data);

    // Set operation_status
    if let Model::MarketList {
        operation_status, ..
    } = &mut model
    {
        *operation_status = Some(OperationStatus::Updating("mp-a".to_string()));
    }

    update(&mut model, Msg::Enter, &mut data);

    // Should still be MarketList (not transitioned)
    assert!(
        matches!(model, Model::MarketList { .. }),
        "Enter should be ignored during operation"
    );
}

// ============================================================================
// Detail アクション
// ============================================================================

#[test]
fn detail_update_action_returns_execute_batch() {
    let mut data = make_data(&["mp-a"]);
    let mut model = Model::new(&data);

    // Enter -> MarketDetail
    update(&mut model, Msg::Enter, &mut data);

    // First action is Update (index 0)
    let effect = update(&mut model, Msg::Enter, &mut data);

    assert!(
        effect.phase2_msg.is_some(),
        "Update action should trigger phase2"
    );

    if let Model::MarketList {
        operation_status, ..
    } = &model
    {
        assert!(
            matches!(operation_status, Some(OperationStatus::Updating(name)) if name == "mp-a"),
            "Should set Updating status"
        );
    } else {
        panic!("Expected MarketList");
    }
}

#[test]
fn detail_remove_action_returns_execute_batch() {
    let mut data = make_data(&["mp-a"]);
    let mut model = Model::new(&data);

    // Enter -> MarketDetail
    update(&mut model, Msg::Enter, &mut data);

    // Move to Remove (index 1)
    update(&mut model, Msg::Down, &mut data);
    let effect = update(&mut model, Msg::Enter, &mut data);

    assert!(
        effect.phase2_msg.is_some(),
        "Remove action should trigger phase2"
    );

    if let Model::MarketList {
        operation_status, ..
    } = &model
    {
        assert!(
            matches!(operation_status, Some(OperationStatus::Removing(name)) if name == "mp-a"),
            "Should set Removing status"
        );
    } else {
        panic!("Expected MarketList");
    }
}

#[test]
fn detail_show_plugins_transitions_to_plugin_list() {
    let mut data = make_data(&["mp-a"]);
    let mut model = Model::new(&data);

    // Enter -> MarketDetail
    update(&mut model, Msg::Enter, &mut data);

    // Move to ShowPlugins (index 2)
    update(&mut model, Msg::Down, &mut data);
    update(&mut model, Msg::Down, &mut data);
    let effect = update(&mut model, Msg::Enter, &mut data);

    assert!(effect.phase2_msg.is_none());

    if let Model::PluginList {
        marketplace_name, ..
    } = &model
    {
        assert_eq!(marketplace_name, "mp-a");
    } else {
        panic!("Expected PluginList");
    }
}

#[test]
fn detail_back_action_returns_to_market_list() {
    let mut data = make_data(&["mp-a"]);
    let mut model = Model::new(&data);

    // Enter -> MarketDetail
    update(&mut model, Msg::Enter, &mut data);

    // Move to Back (index 3)
    update(&mut model, Msg::Down, &mut data);
    update(&mut model, Msg::Down, &mut data);
    update(&mut model, Msg::Down, &mut data);
    update(&mut model, Msg::Enter, &mut data);

    if let Model::MarketList { selected_id, .. } = &model {
        assert_eq!(selected_id.as_deref(), Some("mp-a"));
    } else {
        panic!("Expected MarketList");
    }
}

// ============================================================================
// Back ナビゲーション
// ============================================================================

#[test]
fn back_from_detail_returns_to_market_list() {
    let mut data = make_data(&["mp-a", "mp-b"]);
    let mut model = Model::new(&data);

    // Enter -> MarketDetail
    update(&mut model, Msg::Enter, &mut data);
    assert!(matches!(model, Model::MarketDetail { .. }));

    // Back -> MarketList
    update(&mut model, Msg::Back, &mut data);

    if let Model::MarketList { selected_id, .. } = &model {
        assert_eq!(selected_id.as_deref(), Some("mp-a"));
    } else {
        panic!("Expected MarketList");
    }
}

#[test]
fn back_from_plugin_list_returns_to_detail() {
    let mut data = make_data(&["mp-a"]);
    let mut model = Model::new(&data);

    // Enter -> MarketDetail
    update(&mut model, Msg::Enter, &mut data);

    // ShowPlugins (index 2)
    update(&mut model, Msg::Down, &mut data);
    update(&mut model, Msg::Down, &mut data);
    update(&mut model, Msg::Enter, &mut data);
    assert!(matches!(model, Model::PluginList { .. }));

    // Back -> MarketDetail
    update(&mut model, Msg::Back, &mut data);

    if let Model::MarketDetail {
        marketplace_name, ..
    } = &model
    {
        assert_eq!(marketplace_name, "mp-a");
    } else {
        panic!("Expected MarketDetail");
    }
}

#[test]
fn back_from_add_form_returns_to_market_list() {
    let mut data = make_data(&["mp-a"]);
    let mut model = Model::new(&data);

    // Move to Add new and enter
    update(&mut model, Msg::Down, &mut data);
    update(&mut model, Msg::Enter, &mut data);
    assert!(matches!(model, Model::AddForm(_)));

    // Back -> MarketList
    update(&mut model, Msg::Back, &mut data);

    assert!(
        matches!(model, Model::MarketList { .. }),
        "Back from AddForm should return to MarketList"
    );
}

// ============================================================================
// FormInput / FormBackspace
// ============================================================================

#[test]
fn form_input_appends_to_source() {
    let mut data = make_data(&[]);
    let mut model = Model::new(&data);

    // Enter on Add new
    update(&mut model, Msg::Enter, &mut data);

    update(&mut model, Msg::FormInput('a'), &mut data);
    update(&mut model, Msg::FormInput('b'), &mut data);

    if let Model::AddForm(AddFormModel::Source { source_input, .. }) = &model {
        assert_eq!(source_input, "ab");
    } else {
        panic!("Expected AddForm Source");
    }
}

#[test]
fn form_backspace_removes_from_source() {
    let mut data = make_data(&[]);
    let mut model = Model::new(&data);

    update(&mut model, Msg::Enter, &mut data);
    update(&mut model, Msg::FormInput('a'), &mut data);
    update(&mut model, Msg::FormInput('b'), &mut data);
    update(&mut model, Msg::FormBackspace, &mut data);

    if let Model::AddForm(AddFormModel::Source { source_input, .. }) = &model {
        assert_eq!(source_input, "a");
    } else {
        panic!("Expected AddForm Source");
    }
}

#[test]
fn form_input_clears_error_message() {
    let mut data = make_data(&[]);
    let mut model = Model::new(&data);

    update(&mut model, Msg::Enter, &mut data);

    // Set error manually
    if let Model::AddForm(AddFormModel::Source { error_message, .. }) = &mut model {
        *error_message = Some("test error".to_string());
    }

    update(&mut model, Msg::FormInput('x'), &mut data);

    if let Model::AddForm(AddFormModel::Source { error_message, .. }) = &model {
        assert!(
            error_message.is_none(),
            "FormInput should clear error_message"
        );
    } else {
        panic!("Expected AddForm Source");
    }
}

// ============================================================================
// AddForm Enter (Source -> Name -> Confirm)
// ============================================================================

#[test]
fn enter_empty_source_shows_error() {
    let mut data = make_data(&[]);
    let mut model = Model::new(&data);

    update(&mut model, Msg::Enter, &mut data);
    // Enter with empty input
    update(&mut model, Msg::Enter, &mut data);

    if let Model::AddForm(AddFormModel::Source { error_message, .. }) = &model {
        assert!(
            error_message.is_some(),
            "Should show error for empty source"
        );
    } else {
        panic!("Expected AddForm Source");
    }
}

#[test]
fn enter_invalid_source_shows_error() {
    let mut data = make_data(&[]);
    let mut model = Model::new(&data);

    update(&mut model, Msg::Enter, &mut data);

    // Type invalid source (no slash)
    for c in "invalid".chars() {
        update(&mut model, Msg::FormInput(c), &mut data);
    }
    update(&mut model, Msg::Enter, &mut data);

    if let Model::AddForm(AddFormModel::Source { error_message, .. }) = &model {
        assert!(
            error_message.is_some(),
            "Should show error for invalid source"
        );
        assert!(
            error_message.as_ref().unwrap().contains("Invalid source"),
            "Error should mention invalid source"
        );
    } else {
        panic!("Expected AddForm Source");
    }
}

#[test]
fn enter_valid_source_transitions_to_name_step() {
    let mut data = make_data(&[]);
    let mut model = Model::new(&data);

    update(&mut model, Msg::Enter, &mut data);

    // Type valid source
    for c in "owner/repo".chars() {
        update(&mut model, Msg::FormInput(c), &mut data);
    }
    update(&mut model, Msg::Enter, &mut data);

    if let Model::AddForm(AddFormModel::Name {
        source,
        default_name,
        ..
    }) = &model
    {
        assert_eq!(source, "owner/repo");
        assert_eq!(default_name, "repo");
    } else {
        panic!("Expected AddForm Name");
    }
}

#[test]
fn enter_name_step_with_empty_input_uses_default() {
    let mut data = make_data(&[]);
    let mut model = Model::new(&data);

    // Source step
    update(&mut model, Msg::Enter, &mut data);
    for c in "owner/my-repo".chars() {
        update(&mut model, Msg::FormInput(c), &mut data);
    }
    update(&mut model, Msg::Enter, &mut data);

    // Name step - enter with empty input
    update(&mut model, Msg::Enter, &mut data);

    if let Model::AddForm(AddFormModel::Confirm { source, name, .. }) = &model {
        assert_eq!(source, "owner/my-repo");
        assert_eq!(name, "my-repo");
    } else {
        panic!("Expected AddForm Confirm");
    }
}

#[test]
fn enter_name_step_with_custom_name() {
    let mut data = make_data(&[]);
    let mut model = Model::new(&data);

    // Source step
    update(&mut model, Msg::Enter, &mut data);
    for c in "owner/repo".chars() {
        update(&mut model, Msg::FormInput(c), &mut data);
    }
    update(&mut model, Msg::Enter, &mut data);

    // Name step - type custom name
    for c in "custom-name".chars() {
        update(&mut model, Msg::FormInput(c), &mut data);
    }
    update(&mut model, Msg::Enter, &mut data);

    if let Model::AddForm(AddFormModel::Confirm { name, .. }) = &model {
        assert_eq!(name, "custom-name");
    } else {
        panic!("Expected AddForm Confirm");
    }
}

#[test]
fn enter_name_step_duplicate_shows_error() {
    let mut data = make_data(&["existing"]);
    let mut model = Model::new(&data);

    // Navigate to Add new
    update(&mut model, Msg::Down, &mut data);
    update(&mut model, Msg::Enter, &mut data);

    // Source step
    for c in "owner/repo".chars() {
        update(&mut model, Msg::FormInput(c), &mut data);
    }
    update(&mut model, Msg::Enter, &mut data);

    // Name step - type duplicate name
    for c in "existing".chars() {
        update(&mut model, Msg::FormInput(c), &mut data);
    }
    update(&mut model, Msg::Enter, &mut data);

    if let Model::AddForm(AddFormModel::Name { error_message, .. }) = &model {
        assert!(
            error_message.is_some(),
            "Should show error for duplicate name"
        );
        assert!(
            error_message.as_ref().unwrap().contains("already exists"),
            "Error should mention already exists"
        );
    } else {
        panic!("Expected AddForm Name with error");
    }
}

// ============================================================================
// execute_add_with (依存関数注入パターン)
// ============================================================================

#[test]
fn execute_add_success_transitions_to_market_list() {
    let mut data = make_data(&[]);
    let mut model = Model::AddForm(AddFormModel::Confirm {
        source: "owner/repo".to_string(),
        name: "my-repo".to_string(),
        error_message: None,
    });

    execute_add_with(
        "owner/repo",
        "my-repo",
        &mut model,
        &mut data,
        |_source, name| Ok(make_add_result(name)),
        |d| {
            d.marketplaces.push(make_marketplace("my-repo"));
        },
    );

    if let Model::MarketList {
        selected_id,
        operation_status,
        ..
    } = &model
    {
        assert_eq!(selected_id.as_deref(), Some("my-repo"));
        assert!(operation_status.is_none());
    } else {
        panic!("Expected MarketList after successful add");
    }
}

#[test]
fn execute_add_failure_returns_to_confirm_with_error() {
    let mut data = make_data(&[]);
    let mut model = Model::AddForm(AddFormModel::Confirm {
        source: "owner/repo".to_string(),
        name: "my-repo".to_string(),
        error_message: None,
    });

    execute_add_with(
        "owner/repo",
        "my-repo",
        &mut model,
        &mut data,
        |_source, _name| Err("network error".to_string()),
        |_d| {},
    );

    if let Model::AddForm(AddFormModel::Confirm {
        error_message,
        source,
        name,
        ..
    }) = &model
    {
        assert_eq!(source, "owner/repo");
        assert_eq!(name, "my-repo");
        assert!(error_message.is_some(), "Should set error on failure");
        assert!(
            error_message.as_ref().unwrap().contains("network error"),
            "Error should contain failure reason"
        );
    } else {
        panic!("Expected AddForm Confirm with error");
    }
}

// ============================================================================
// ExecuteUpdate (依存関数注入パターン)
// ============================================================================

#[test]
fn execute_update_success_clears_status() {
    let mut data = make_data(&["mp-a"]);
    let mut state = ListState::default();
    state.select(Some(0));
    let mut model = Model::MarketList {
        selected_id: Some("mp-a".to_string()),
        state,
        operation_status: Some(OperationStatus::Updating("mp-a".to_string())),
        error_message: None,
    };

    execute_update_with(
        &mut model,
        &mut data,
        |_name| Ok(make_marketplace("mp-a")),
        || vec![],
        |_d| {},
    );

    if let Model::MarketList {
        operation_status,
        error_message,
        selected_id,
        ..
    } = &model
    {
        assert!(operation_status.is_none(), "Should clear operation_status");
        assert!(error_message.is_none(), "Should have no error");
        assert_eq!(selected_id.as_deref(), Some("mp-a"));
    } else {
        panic!("Expected MarketList");
    }
}

#[test]
fn execute_update_failure_sets_error_message() {
    let mut data = make_data(&["mp-a"]);
    let mut state = ListState::default();
    state.select(Some(0));
    let mut model = Model::MarketList {
        selected_id: Some("mp-a".to_string()),
        state,
        operation_status: Some(OperationStatus::Updating("mp-a".to_string())),
        error_message: None,
    };

    execute_update_with(
        &mut model,
        &mut data,
        |_name| Err("network error".to_string()),
        || vec![],
        |_d| {},
    );

    if let Model::MarketList {
        operation_status,
        error_message,
        ..
    } = &model
    {
        assert!(operation_status.is_none(), "Should clear operation_status");
        assert!(error_message.is_some(), "Should set error message");
        assert!(
            error_message.as_ref().unwrap().contains("network error"),
            "Error should contain failure reason"
        );
    } else {
        panic!("Expected MarketList");
    }
}

#[test]
fn execute_update_all_success() {
    let mut data = make_data(&["mp-a", "mp-b"]);
    let mut state = ListState::default();
    state.select(Some(0));
    let mut model = Model::MarketList {
        selected_id: Some("mp-a".to_string()),
        state,
        operation_status: Some(OperationStatus::UpdatingAll),
        error_message: None,
    };

    execute_update_with(
        &mut model,
        &mut data,
        |_name| Ok(make_marketplace("unused")),
        || {
            vec![
                ("mp-a".to_string(), Ok(make_marketplace("mp-a"))),
                ("mp-b".to_string(), Ok(make_marketplace("mp-b"))),
            ]
        },
        |_d| {},
    );

    if let Model::MarketList {
        operation_status,
        error_message,
        ..
    } = &model
    {
        assert!(operation_status.is_none());
        assert!(error_message.is_none(), "No errors on full success");
    } else {
        panic!("Expected MarketList");
    }
}

#[test]
fn execute_update_all_partial_failure_sets_error() {
    let mut data = make_data(&["mp-a", "mp-b"]);
    let mut state = ListState::default();
    state.select(Some(0));
    let mut model = Model::MarketList {
        selected_id: Some("mp-a".to_string()),
        state,
        operation_status: Some(OperationStatus::UpdatingAll),
        error_message: None,
    };

    execute_update_with(
        &mut model,
        &mut data,
        |_name| Ok(make_marketplace("unused")),
        || {
            vec![
                ("mp-a".to_string(), Ok(make_marketplace("mp-a"))),
                ("mp-b".to_string(), Err("fetch failed".to_string())),
            ]
        },
        |_d| {},
    );

    if let Model::MarketList { error_message, .. } = &model {
        assert!(
            error_message.is_some(),
            "Should set error on partial failure"
        );
        assert!(
            error_message.as_ref().unwrap().contains("mp-b"),
            "Error should mention failed marketplace"
        );
        assert!(
            error_message.as_ref().unwrap().contains("fetch failed"),
            "Error should contain failure reason"
        );
    } else {
        panic!("Expected MarketList");
    }
}

// ============================================================================
// ExecuteRemove (依存関数注入パターン)
// ============================================================================

#[test]
fn execute_remove_success_reloads_and_clamps() {
    let mut data = make_data(&["mp-a", "mp-b"]);
    let mut state = ListState::default();
    state.select(Some(0));
    let mut model = Model::MarketList {
        selected_id: Some("mp-a".to_string()),
        state,
        operation_status: Some(OperationStatus::Removing("mp-a".to_string())),
        error_message: None,
    };

    execute_remove_with(
        &mut model,
        &mut data,
        |_name| Ok(()),
        |d| {
            d.marketplaces.retain(|m| m.name != "mp-a");
        },
    );

    if let Model::MarketList {
        selected_id,
        operation_status,
        state,
        ..
    } = &model
    {
        assert!(operation_status.is_none());
        assert_eq!(selected_id.as_deref(), Some("mp-b"));
        assert_eq!(state.selected(), Some(0));
    } else {
        panic!("Expected MarketList");
    }
}

#[test]
fn execute_remove_failure_sets_error() {
    let mut data = make_data(&["mp-a"]);
    let mut state = ListState::default();
    state.select(Some(0));
    let mut model = Model::MarketList {
        selected_id: Some("mp-a".to_string()),
        state,
        operation_status: Some(OperationStatus::Removing("mp-a".to_string())),
        error_message: None,
    };

    execute_remove_with(
        &mut model,
        &mut data,
        |_name| Err("permission denied".to_string()),
        |_d| {},
    );

    if let Model::MarketList {
        error_message,
        operation_status,
        ..
    } = &model
    {
        assert!(operation_status.is_none());
        assert!(error_message.is_some(), "Should set error on failure");
        assert!(
            error_message
                .as_ref()
                .unwrap()
                .contains("permission denied"),
            "Error should contain failure reason"
        );
    } else {
        panic!("Expected MarketList");
    }
}

// ============================================================================
// UpdateMarket / UpdateAll
// ============================================================================

#[test]
fn update_market_sets_updating_and_returns_execute_batch() {
    let mut data = make_data(&["mp-a"]);
    let mut model = Model::new(&data);

    let effect = update(&mut model, Msg::UpdateMarket, &mut data);

    assert!(effect.phase2_msg.is_some());

    if let Model::MarketList {
        operation_status, ..
    } = &model
    {
        assert!(
            matches!(operation_status, Some(OperationStatus::Updating(name)) if name == "mp-a")
        );
    } else {
        panic!("Expected MarketList");
    }
}

#[test]
fn update_market_ignored_on_add_new() {
    let mut data = make_data(&["mp-a"]);
    let mut model = Model::new(&data);

    // Move to Add new
    update(&mut model, Msg::Down, &mut data);
    let effect = update(&mut model, Msg::UpdateMarket, &mut data);

    assert!(effect.phase2_msg.is_none(), "Should not trigger on Add new");
}

#[test]
fn update_market_ignored_when_operation_in_progress() {
    let mut data = make_data(&["mp-a"]);
    let mut model = Model::new(&data);

    if let Model::MarketList {
        operation_status, ..
    } = &mut model
    {
        *operation_status = Some(OperationStatus::Updating("mp-a".to_string()));
    }

    let effect = update(&mut model, Msg::UpdateMarket, &mut data);

    assert!(effect.phase2_msg.is_none(), "Should not double-trigger");
}

#[test]
fn update_all_sets_updating_all_and_returns_execute_batch() {
    let mut data = make_data(&["mp-a", "mp-b"]);
    let mut model = Model::new(&data);

    let effect = update(&mut model, Msg::UpdateAll, &mut data);

    assert!(effect.phase2_msg.is_some());

    if let Model::MarketList {
        operation_status, ..
    } = &model
    {
        assert!(matches!(
            operation_status,
            Some(OperationStatus::UpdatingAll)
        ));
    } else {
        panic!("Expected MarketList");
    }
}

#[test]
fn update_all_on_empty_list_does_nothing() {
    let mut data = make_data(&[]);
    let mut model = Model::new(&data);

    let effect = update(&mut model, Msg::UpdateAll, &mut data);

    assert!(effect.phase2_msg.is_none());
}

#[test]
fn update_all_ignored_when_operation_in_progress() {
    let mut data = make_data(&["mp-a"]);
    let mut model = Model::new(&data);

    if let Model::MarketList {
        operation_status, ..
    } = &mut model
    {
        *operation_status = Some(OperationStatus::Updating("mp-a".to_string()));
    }

    let effect = update(&mut model, Msg::UpdateAll, &mut data);

    assert!(effect.phase2_msg.is_none(), "Should not double-trigger");
}

// ============================================================================
// error_message ライフサイクル
// ============================================================================

#[test]
fn error_message_cleared_on_up() {
    let mut data = make_data(&["mp-a", "mp-b"]);
    let mut state = ListState::default();
    state.select(Some(1));
    let mut model = Model::MarketList {
        selected_id: Some("mp-b".to_string()),
        state,
        operation_status: None,
        error_message: Some("some error".to_string()),
    };

    update(&mut model, Msg::Up, &mut data);

    if let Model::MarketList { error_message, .. } = &model {
        assert!(error_message.is_none(), "Error should be cleared on Up");
    }
}

#[test]
fn error_message_cleared_on_down() {
    let mut data = make_data(&["mp-a", "mp-b"]);
    let mut model = Model::MarketList {
        selected_id: Some("mp-a".to_string()),
        state: {
            let mut s = ListState::default();
            s.select(Some(0));
            s
        },
        operation_status: None,
        error_message: Some("some error".to_string()),
    };

    update(&mut model, Msg::Down, &mut data);

    if let Model::MarketList { error_message, .. } = &model {
        assert!(error_message.is_none(), "Error should be cleared on Down");
    }
}

#[test]
fn error_message_cleared_on_enter() {
    let mut data = make_data(&["mp-a"]);
    let mut model = Model::MarketList {
        selected_id: Some("mp-a".to_string()),
        state: {
            let mut s = ListState::default();
            s.select(Some(0));
            s
        },
        operation_status: None,
        error_message: Some("some error".to_string()),
    };

    update(&mut model, Msg::Enter, &mut data);

    // Should transition to MarketDetail (error was cleared)
    assert!(matches!(
        model,
        Model::MarketDetail {
            error_message: None,
            ..
        }
    ));
}

#[test]
fn error_message_cleared_on_back() {
    let mut data = make_data(&["mp-a"]);
    let mut model = Model::MarketDetail {
        marketplace_name: "mp-a".to_string(),
        state: {
            let mut s = ListState::default();
            s.select(Some(0));
            s
        },
        error_message: Some("some error".to_string()),
    };

    update(&mut model, Msg::Back, &mut data);

    if let Model::MarketList { error_message, .. } = &model {
        assert!(
            error_message.is_none(),
            "Error should be cleared after back"
        );
    }
}

#[test]
fn stale_error_cleared_on_update_market_start() {
    let mut data = make_data(&["mp-a"]);
    let mut model = Model::MarketList {
        selected_id: Some("mp-a".to_string()),
        state: {
            let mut s = ListState::default();
            s.select(Some(0));
            s
        },
        operation_status: None,
        error_message: Some("previous error".to_string()),
    };

    let effect = update(&mut model, Msg::UpdateMarket, &mut data);

    assert!(effect.phase2_msg.is_some());
    if let Model::MarketList { error_message, .. } = &model {
        assert!(
            error_message.is_none(),
            "Stale error should be cleared when starting update"
        );
    }
}

#[test]
fn stale_error_cleared_on_update_all_start() {
    let mut data = make_data(&["mp-a"]);
    let mut model = Model::MarketList {
        selected_id: Some("mp-a".to_string()),
        state: {
            let mut s = ListState::default();
            s.select(Some(0));
            s
        },
        operation_status: None,
        error_message: Some("previous error".to_string()),
    };

    let effect = update(&mut model, Msg::UpdateAll, &mut data);

    assert!(effect.phase2_msg.is_some());
    if let Model::MarketList { error_message, .. } = &model {
        assert!(
            error_message.is_none(),
            "Stale error should be cleared when starting update all"
        );
    }
}

#[test]
fn stale_error_cleared_on_successful_update_retry() {
    let mut data = make_data(&["mp-a"]);
    let mut state = ListState::default();
    state.select(Some(0));
    let mut model = Model::MarketList {
        selected_id: Some("mp-a".to_string()),
        state,
        operation_status: Some(OperationStatus::Updating("mp-a".to_string())),
        error_message: Some("previous failure".to_string()),
    };

    execute_update_with(
        &mut model,
        &mut data,
        |_name| Ok(make_marketplace("mp-a")),
        || vec![],
        |_d| {},
    );

    if let Model::MarketList { error_message, .. } = &model {
        assert!(
            error_message.is_none(),
            "Stale error should be cleared after successful update"
        );
    } else {
        panic!("Expected MarketList");
    }
}

#[test]
fn stale_error_cleared_on_successful_remove_retry() {
    let mut data = make_data(&["mp-a"]);
    let mut state = ListState::default();
    state.select(Some(0));
    let mut model = Model::MarketList {
        selected_id: Some("mp-a".to_string()),
        state,
        operation_status: Some(OperationStatus::Removing("mp-a".to_string())),
        error_message: Some("previous failure".to_string()),
    };

    execute_remove_with(
        &mut model,
        &mut data,
        |_name| Ok(()),
        |d| {
            d.marketplaces.clear();
        },
    );

    if let Model::MarketList { error_message, .. } = &model {
        assert!(
            error_message.is_none(),
            "Stale error should be cleared after successful remove"
        );
    } else {
        panic!("Expected MarketList");
    }
}

#[test]
fn stale_error_cleared_on_successful_update_all_retry() {
    let mut data = make_data(&["mp-a", "mp-b"]);
    let mut state = ListState::default();
    state.select(Some(0));
    let mut model = Model::MarketList {
        selected_id: Some("mp-a".to_string()),
        state,
        operation_status: Some(OperationStatus::UpdatingAll),
        error_message: Some("previous failure".to_string()),
    };

    execute_update_with(
        &mut model,
        &mut data,
        |_name| Ok(make_marketplace("unused")),
        || {
            vec![
                ("mp-a".to_string(), Ok(make_marketplace("mp-a"))),
                ("mp-b".to_string(), Ok(make_marketplace("mp-b"))),
            ]
        },
        |_d| {},
    );

    if let Model::MarketList { error_message, .. } = &model {
        assert!(
            error_message.is_none(),
            "Stale error should be cleared after successful update all"
        );
    } else {
        panic!("Expected MarketList");
    }
}

// ============================================================================
// Helper
// ============================================================================

fn model_variant(model: &Model) -> &'static str {
    match model {
        Model::MarketList { .. } => "MarketList",
        Model::MarketDetail { .. } => "MarketDetail",
        Model::PluginList { .. } => "PluginList",
        Model::AddForm(_) => "AddForm",
    }
}
