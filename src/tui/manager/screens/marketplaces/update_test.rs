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

fn make_data(names: &[&str]) -> (tempfile::TempDir, DataStore) {
    DataStore::for_test(
        vec![],
        names.iter().map(|n| make_marketplace(n)).collect(),
        None,
    )
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
    let (_temp_dir, mut data) = make_data(&["mp-a", "mp-b"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a", "mp-b"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
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
    let (_temp_dir, mut data) = make_data(&[]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
    let mut model = Model::new(&data);

    // Enter -> MarketDetail
    update(&mut model, Msg::Enter, &mut data);

    // Move to Back (index 4: Update, Remove, ShowPlugins, BrowsePlugins, Back)
    update(&mut model, Msg::Down, &mut data);
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
// PluginBrowse navigation (Up/Down)
// ============================================================================

fn make_plugin_browse(name: &str, plugin_count: usize) -> Model {
    use crate::marketplace::PluginSource;
    use crate::tui::manager::screens::marketplaces::model::BrowsePlugin;
    use std::collections::HashSet;

    let plugins: Vec<BrowsePlugin> = (0..plugin_count)
        .map(|i| BrowsePlugin {
            name: format!("plugin-{}", i),
            description: None,
            version: None,
            source: PluginSource::Local("local".to_string()),
            installed: false,
        })
        .collect();
    let mut state = ListState::default();
    if !plugins.is_empty() {
        state.select(Some(0));
    }
    Model::PluginBrowse {
        marketplace_name: name.to_string(),
        plugins,
        selected_plugins: HashSet::new(),
        highlighted_idx: 0,
        state,
    }
}

#[test]
fn down_in_plugin_browse_moves_highlight() {
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
    let mut model = make_plugin_browse("mp-a", 3);

    update(&mut model, Msg::Down, &mut data);

    if let Model::PluginBrowse {
        highlighted_idx,
        state,
        ..
    } = &model
    {
        assert_eq!(*highlighted_idx, 1);
        assert_eq!(state.selected(), Some(1));
    } else {
        panic!("Expected PluginBrowse");
    }
}

#[test]
fn up_in_plugin_browse_does_not_go_below_zero() {
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
    let mut model = make_plugin_browse("mp-a", 3);

    update(&mut model, Msg::Up, &mut data);

    if let Model::PluginBrowse {
        highlighted_idx, ..
    } = &model
    {
        assert_eq!(*highlighted_idx, 0);
    } else {
        panic!("Expected PluginBrowse");
    }
}

#[test]
fn down_in_plugin_browse_does_not_exceed_len() {
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
    let mut model = make_plugin_browse("mp-a", 2);

    // Move to last
    update(&mut model, Msg::Down, &mut data);
    // Try to go beyond
    update(&mut model, Msg::Down, &mut data);

    if let Model::PluginBrowse {
        highlighted_idx, ..
    } = &model
    {
        assert_eq!(*highlighted_idx, 1);
    } else {
        panic!("Expected PluginBrowse");
    }
}

// ============================================================================
// TargetSelect / ScopeSelect navigation
// ============================================================================

fn make_target_select(name: &str) -> Model {
    use std::collections::HashSet;

    let mut state = ListState::default();
    state.select(Some(0));
    Model::TargetSelect {
        marketplace_name: name.to_string(),
        plugins: vec![],
        selected_plugins: HashSet::new(),
        targets: vec![
            ("codex".to_string(), "OpenAI Codex".to_string(), false),
            ("copilot".to_string(), "VS Code Copilot".to_string(), false),
            (
                "antigravity".to_string(),
                "Google Antigravity".to_string(),
                false,
            ),
            ("gemini-cli".to_string(), "Gemini CLI".to_string(), false),
        ],
        highlighted_idx: 0,
        state,
    }
}

fn make_scope_select(name: &str) -> Model {
    use std::collections::HashSet;

    let mut state = ListState::default();
    state.select(Some(0));
    Model::ScopeSelect {
        marketplace_name: name.to_string(),
        plugins: vec![],
        selected_plugins: HashSet::new(),
        target_names: vec!["codex".to_string()],
        highlighted_idx: 0,
        state,
    }
}

#[test]
fn down_in_target_select_moves_highlight() {
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
    let mut model = make_target_select("mp-a");

    update(&mut model, Msg::Down, &mut data);

    if let Model::TargetSelect {
        highlighted_idx,
        state,
        ..
    } = &model
    {
        assert_eq!(*highlighted_idx, 1);
        assert_eq!(state.selected(), Some(1));
    } else {
        panic!("Expected TargetSelect");
    }
}

#[test]
fn up_down_in_scope_select_moves_highlight() {
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
    let mut model = make_scope_select("mp-a");

    // Down from 0 -> 1
    update(&mut model, Msg::Down, &mut data);
    if let Model::ScopeSelect {
        highlighted_idx, ..
    } = &model
    {
        assert_eq!(*highlighted_idx, 1);
    }

    // Down again -> still 1 (clamped, only 2 options)
    update(&mut model, Msg::Down, &mut data);
    if let Model::ScopeSelect {
        highlighted_idx, ..
    } = &model
    {
        assert_eq!(*highlighted_idx, 1, "Should clamp at 1");
    }

    // Up from 1 -> 0
    update(&mut model, Msg::Up, &mut data);
    if let Model::ScopeSelect {
        highlighted_idx, ..
    } = &model
    {
        assert_eq!(*highlighted_idx, 0);
    }
}

// ============================================================================
// ConfirmTargets (TargetSelect -> ScopeSelect)
// ============================================================================

#[test]
fn confirm_targets_transitions_to_scope_select() {
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
    let mut model = make_target_select("mp-a");

    // Select first target
    if let Model::TargetSelect { targets, .. } = &mut model {
        targets[0].2 = true;
    }

    update(&mut model, Msg::ConfirmTargets, &mut data);

    assert_eq!(
        model_variant(&model),
        "ScopeSelect",
        "Should transition to ScopeSelect"
    );

    if let Model::ScopeSelect {
        target_names,
        highlighted_idx,
        state,
        ..
    } = &model
    {
        assert!(target_names.contains(&"codex".to_string()));
        assert_eq!(*highlighted_idx, 0, "Should default to Personal (0)");
        assert_eq!(state.selected(), Some(0));
    }
}

#[test]
fn confirm_targets_ignored_when_no_target_selected() {
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
    let mut model = make_target_select("mp-a");
    // All targets unselected

    update(&mut model, Msg::ConfirmTargets, &mut data);

    assert_eq!(
        model_variant(&model),
        "TargetSelect",
        "Should stay TargetSelect when no target selected"
    );
}

// ============================================================================
// TargetSelect toggle
// ============================================================================

#[test]
fn toggle_select_in_target_select_toggles_target() {
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
    let mut model = make_target_select("mp-a");

    // Toggle on
    update(&mut model, Msg::ToggleSelect, &mut data);
    if let Model::TargetSelect { targets, .. } = &model {
        assert!(targets[0].2, "Target 0 should be selected after toggle");
    } else {
        panic!("Expected TargetSelect");
    }

    // Toggle off
    update(&mut model, Msg::ToggleSelect, &mut data);
    if let Model::TargetSelect { targets, .. } = &model {
        assert!(
            !targets[0].2,
            "Target 0 should be deselected after second toggle"
        );
    } else {
        panic!("Expected TargetSelect");
    }
}

// ============================================================================
// StartInstall (PluginBrowse -> TargetSelect)
// ============================================================================

#[test]
fn start_install_transitions_to_target_select() {
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
    let mut model = make_plugin_browse("mp-a", 3);

    // Select plugin-0
    if let Model::PluginBrowse {
        selected_plugins, ..
    } = &mut model
    {
        selected_plugins.insert("plugin-0".to_string());
    }

    update(&mut model, Msg::StartInstall, &mut data);

    assert_eq!(
        model_variant(&model),
        "TargetSelect",
        "Should transition to TargetSelect"
    );

    if let Model::TargetSelect {
        marketplace_name,
        targets,
        selected_plugins,
        highlighted_idx,
        state,
        ..
    } = &model
    {
        assert_eq!(marketplace_name, "mp-a");
        assert!(
            !targets.is_empty(),
            "Should have targets from all_targets()"
        );
        assert!(
            targets.iter().all(|(_, _, sel)| !sel),
            "All targets should be unselected"
        );
        assert!(selected_plugins.contains("plugin-0"));
        assert_eq!(*highlighted_idx, 0);
        assert_eq!(state.selected(), Some(0));
    }
}

#[test]
fn start_install_ignored_when_no_selection() {
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
    let mut model = make_plugin_browse("mp-a", 3);
    // selected_plugins is empty

    update(&mut model, Msg::StartInstall, &mut data);

    assert_eq!(
        model_variant(&model),
        "PluginBrowse",
        "Should stay PluginBrowse when no selection"
    );
}

// ============================================================================
// PluginBrowse toggle selection
// ============================================================================

#[test]
fn toggle_select_in_plugin_browse_adds_to_selected() {
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
    let mut model = make_plugin_browse("mp-a", 3);

    update(&mut model, Msg::ToggleSelect, &mut data);

    if let Model::PluginBrowse {
        selected_plugins, ..
    } = &model
    {
        assert!(
            selected_plugins.contains("plugin-0"),
            "Should add plugin-0 to selected"
        );
    } else {
        panic!("Expected PluginBrowse");
    }
}

#[test]
fn toggle_select_in_plugin_browse_removes_from_selected() {
    use std::collections::HashSet;

    let (_temp_dir, mut data) = make_data(&["mp-a"]);
    let mut model = make_plugin_browse("mp-a", 3);

    // Pre-add plugin-0 to selected
    if let Model::PluginBrowse {
        selected_plugins, ..
    } = &mut model
    {
        selected_plugins.insert("plugin-0".to_string());
    }

    update(&mut model, Msg::ToggleSelect, &mut data);

    if let Model::PluginBrowse {
        selected_plugins, ..
    } = &model
    {
        assert!(
            !selected_plugins.contains("plugin-0"),
            "Should remove plugin-0 from selected"
        );
    } else {
        panic!("Expected PluginBrowse");
    }
}

// ============================================================================
// BrowsePlugins (Enter from MarketDetail)
// ============================================================================

#[test]
fn enter_browse_plugins_transitions_to_plugin_browse() {
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
    let mut model = Model::new(&data);

    // Enter -> MarketDetail
    update(&mut model, Msg::Enter, &mut data);

    // Move to BrowsePlugins (index 3: Update=0, Remove=1, ShowPlugins=2, BrowsePlugins=3)
    update(&mut model, Msg::Down, &mut data);
    update(&mut model, Msg::Down, &mut data);
    update(&mut model, Msg::Down, &mut data);
    update(&mut model, Msg::Enter, &mut data);

    assert_eq!(
        model_variant(&model),
        "PluginBrowse",
        "Should transition to PluginBrowse"
    );

    if let Model::PluginBrowse {
        marketplace_name,
        highlighted_idx,
        selected_plugins,
        ..
    } = &model
    {
        assert_eq!(marketplace_name, "mp-a");
        assert_eq!(*highlighted_idx, 0);
        assert!(selected_plugins.is_empty());
    }
}

#[test]
fn enter_browse_plugins_noop_when_no_cache() {
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
    // Set plugin_count to None (no cache)
    data.marketplaces[0].plugin_count = None;

    let mut model = Model::new(&data);

    // Enter -> MarketDetail
    update(&mut model, Msg::Enter, &mut data);

    // Move to BrowsePlugins (index 3)
    update(&mut model, Msg::Down, &mut data);
    update(&mut model, Msg::Down, &mut data);
    update(&mut model, Msg::Down, &mut data);
    update(&mut model, Msg::Enter, &mut data);

    assert_eq!(
        model_variant(&model),
        "MarketDetail",
        "Should stay at MarketDetail when no cache"
    );
}

// ============================================================================
// BackToPluginBrowse (InstallResult -> PluginBrowse)
// ============================================================================

#[test]
fn back_to_plugin_browse_from_result_refreshes_plugins() {
    use crate::marketplace::PluginSource;
    use crate::tui::manager::screens::marketplaces::model::BrowsePlugin;

    let (_temp_dir, mut data) = make_data(&["mp-a"]);

    let plugins = vec![BrowsePlugin {
        name: "p1".to_string(),
        description: None,
        version: None,
        source: PluginSource::Local("local".to_string()),
        installed: false,
    }];

    let mut model = Model::InstallResult {
        marketplace_name: "mp-a".to_string(),
        plugins,
        summary: InstallSummary {
            results: vec![PluginInstallResult {
                plugin_name: "p1".to_string(),
                success: true,
                error: None,
            }],
            total: 1,
            succeeded: 1,
            failed: 0,
        },
    };

    update(&mut model, Msg::BackToPluginBrowse, &mut data);

    assert_eq!(model_variant(&model), "PluginBrowse");

    if let Model::PluginBrowse {
        marketplace_name,
        selected_plugins,
        highlighted_idx,
        ..
    } = &model
    {
        assert_eq!(marketplace_name, "mp-a");
        assert!(selected_plugins.is_empty(), "Selection should be cleared");
        assert_eq!(*highlighted_idx, 0);
    }
}

// ============================================================================
// ExecuteInstall (Installing -> InstallResult)
// ============================================================================

use crate::tui::manager::screens::marketplaces::model::{InstallSummary, PluginInstallResult};

fn make_installing(name: &str, plugin_names: &[&str]) -> Model {
    use crate::marketplace::PluginSource;
    use crate::tui::manager::screens::marketplaces::model::BrowsePlugin;

    let plugins: Vec<BrowsePlugin> = plugin_names
        .iter()
        .map(|n| BrowsePlugin {
            name: n.to_string(),
            description: None,
            version: None,
            source: PluginSource::Local("local".to_string()),
            installed: false,
        })
        .collect();

    Model::Installing {
        marketplace_name: name.to_string(),
        plugins,
        plugin_names: plugin_names.iter().map(|n| n.to_string()).collect(),
        target_names: vec!["codex".to_string()],
        scope: crate::component::Scope::Personal,
        current_idx: 0,
        total: plugin_names.len(),
    }
}

#[test]
fn execute_install_transitions_to_install_result() {
    use super::execute_install_with;

    let (_temp_dir, mut data) = make_data(&["mp-a"]);
    let mut model = make_installing("mp-a", &["p1", "p2"]);

    let mut reload_called = false;

    execute_install_with(
        &mut model,
        &mut data,
        |_mp, _plugins, _targets, _scope| InstallSummary {
            results: vec![
                PluginInstallResult {
                    plugin_name: "p1".to_string(),
                    success: true,
                    error: None,
                },
                PluginInstallResult {
                    plugin_name: "p2".to_string(),
                    success: true,
                    error: None,
                },
            ],
            total: 2,
            succeeded: 2,
            failed: 0,
        },
        |_d| {
            reload_called = true;
            Ok(())
        },
    );

    assert!(reload_called, "reload should be called after install");
    assert_eq!(model_variant(&model), "InstallResult");

    if let Model::InstallResult { summary, .. } = &model {
        assert_eq!(summary.succeeded, 2);
        assert_eq!(summary.failed, 0);
    }
}

#[test]
fn execute_install_with_failure_shows_in_summary() {
    use super::execute_install_with;

    let (_temp_dir, mut data) = make_data(&["mp-a"]);
    let mut model = make_installing("mp-a", &["p1", "p2"]);

    execute_install_with(
        &mut model,
        &mut data,
        |_mp, _plugins, _targets, _scope| InstallSummary {
            results: vec![
                PluginInstallResult {
                    plugin_name: "p1".to_string(),
                    success: true,
                    error: None,
                },
                PluginInstallResult {
                    plugin_name: "p2".to_string(),
                    success: false,
                    error: Some("install failed".to_string()),
                },
            ],
            total: 2,
            succeeded: 1,
            failed: 1,
        },
        |_d| Ok(()),
    );

    assert_eq!(model_variant(&model), "InstallResult");

    if let Model::InstallResult { summary, .. } = &model {
        assert_eq!(summary.succeeded, 1);
        assert_eq!(summary.failed, 1);
    }
}

// ============================================================================
// ConfirmScope (ScopeSelect -> Installing)
// ============================================================================

fn make_scope_select_with_plugins(name: &str, plugin_names: &[&str]) -> Model {
    use crate::marketplace::PluginSource;
    use crate::tui::manager::screens::marketplaces::model::BrowsePlugin;
    use std::collections::HashSet;

    let plugins: Vec<BrowsePlugin> = plugin_names
        .iter()
        .map(|n| BrowsePlugin {
            name: n.to_string(),
            description: None,
            version: None,
            source: PluginSource::Local("local".to_string()),
            installed: false,
        })
        .collect();
    let selected_plugins: HashSet<String> = plugin_names.iter().map(|n| n.to_string()).collect();

    let mut state = ListState::default();
    state.select(Some(0));

    Model::ScopeSelect {
        marketplace_name: name.to_string(),
        plugins,
        selected_plugins,
        target_names: vec!["codex".to_string()],
        highlighted_idx: 0,
        state,
    }
}

#[test]
fn confirm_scope_transitions_to_installing_with_phase2() {
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
    let mut model = make_scope_select_with_plugins("mp-a", &["p1", "p2"]);

    let effect = update(&mut model, Msg::ConfirmScope, &mut data);

    assert_eq!(model_variant(&model), "Installing");
    assert!(
        effect.phase2_msg.is_some(),
        "Should return Phase2 with ExecuteInstall"
    );

    if let Model::Installing {
        marketplace_name,
        plugin_names,
        target_names,
        scope,
        total,
        ..
    } = &model
    {
        assert_eq!(marketplace_name, "mp-a");
        assert_eq!(plugin_names.len(), 2);
        assert!(target_names.contains(&"codex".to_string()));
        assert_eq!(*scope, crate::component::Scope::Personal);
        assert_eq!(*total, 2);
    }
}

#[test]
fn confirm_scope_personal_sets_scope_personal() {
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
    let mut model = make_scope_select_with_plugins("mp-a", &["p1"]);

    // highlighted_idx = 0 -> Personal
    update(&mut model, Msg::ConfirmScope, &mut data);

    if let Model::Installing { scope, .. } = &model {
        assert_eq!(*scope, crate::component::Scope::Personal);
    } else {
        panic!("Expected Installing");
    }
}

#[test]
fn confirm_scope_project_sets_scope_project() {
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
    let mut model = make_scope_select_with_plugins("mp-a", &["p1"]);

    // Move to Project (index 1)
    if let Model::ScopeSelect {
        highlighted_idx,
        state,
        ..
    } = &mut model
    {
        *highlighted_idx = 1;
        state.select(Some(1));
    }

    update(&mut model, Msg::ConfirmScope, &mut data);

    if let Model::Installing { scope, .. } = &model {
        assert_eq!(*scope, crate::component::Scope::Project);
    } else {
        panic!("Expected Installing");
    }
}

// ============================================================================
// Back from PluginBrowse
// ============================================================================

#[test]
fn back_from_plugin_browse_returns_to_detail() {
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
    let mut model = make_plugin_browse("mp-a", 3);

    // Select a plugin before going back
    if let Model::PluginBrowse {
        selected_plugins, ..
    } = &mut model
    {
        selected_plugins.insert("plugin-0".to_string());
    }

    update(&mut model, Msg::Back, &mut data);

    assert_eq!(
        model_variant(&model),
        "MarketDetail",
        "Should return to MarketDetail"
    );

    if let Model::MarketDetail {
        marketplace_name,
        state,
        browse_plugins,
        browse_selected,
        ..
    } = &model
    {
        assert_eq!(marketplace_name, "mp-a");
        assert_eq!(state.selected(), Some(0));
        assert!(browse_plugins.is_some(), "Should preserve browse plugins");
        assert!(
            browse_selected.as_ref().unwrap().contains("plugin-0"),
            "Should preserve selection"
        );
    }
}

// ============================================================================
// Back from TargetSelect / ScopeSelect
// ============================================================================

#[test]
fn back_from_target_select_returns_to_plugin_browse() {
    use std::collections::HashSet;

    let (_temp_dir, mut data) = make_data(&["mp-a"]);
    let mut model = make_target_select("mp-a");

    // Set selected_plugins
    if let Model::TargetSelect {
        selected_plugins, ..
    } = &mut model
    {
        selected_plugins.insert("p1".to_string());
    }

    update(&mut model, Msg::Back, &mut data);

    assert_eq!(model_variant(&model), "PluginBrowse");

    if let Model::PluginBrowse {
        selected_plugins, ..
    } = &model
    {
        assert!(
            selected_plugins.contains("p1"),
            "Should preserve selected plugins"
        );
    }
}

#[test]
fn back_from_scope_select_returns_to_target_select() {
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
    let mut model = make_scope_select("mp-a");

    update(&mut model, Msg::Back, &mut data);

    assert_eq!(model_variant(&model), "TargetSelect");

    if let Model::TargetSelect { targets, .. } = &model {
        // codex should be selected (was in target_names)
        let codex = targets.iter().find(|(name, _, _)| name == "codex");
        assert!(codex.is_some());
        assert!(codex.unwrap().2, "codex should be marked as selected");
    }
}

#[test]
fn back_from_scope_select_preserves_target_selection() {
    use std::collections::HashSet;

    let (_temp_dir, mut data) = make_data(&["mp-a"]);

    let mut state = ListState::default();
    state.select(Some(0));
    let mut model = Model::ScopeSelect {
        marketplace_name: "mp-a".to_string(),
        plugins: vec![],
        selected_plugins: HashSet::new(),
        target_names: vec!["codex".to_string(), "copilot".to_string()],
        highlighted_idx: 0,
        state,
    };

    update(&mut model, Msg::Back, &mut data);

    assert_eq!(model_variant(&model), "TargetSelect");

    if let Model::TargetSelect { targets, .. } = &model {
        for (name, _, selected) in targets {
            if name == "codex" || name == "copilot" {
                assert!(selected, "{} should be selected", name);
            } else {
                assert!(!selected, "{} should not be selected", name);
            }
        }
    }
}

// ============================================================================
// BrowsePlugins re-entry state preservation
// ============================================================================

#[test]
fn enter_browse_plugins_restores_selection() {
    use crate::marketplace::PluginSource;
    use crate::tui::manager::screens::marketplaces::model::BrowsePlugin;
    use std::collections::HashSet;

    let (_temp_dir, mut data) = make_data(&["mp-a"]);

    let preserved_plugins = vec![
        BrowsePlugin {
            name: "p1".to_string(),
            description: None,
            version: None,
            source: PluginSource::Local("local".to_string()),
            installed: false,
        },
        BrowsePlugin {
            name: "p2".to_string(),
            description: None,
            version: None,
            source: PluginSource::Local("local".to_string()),
            installed: false,
        },
    ];
    let mut preserved_selected = HashSet::new();
    preserved_selected.insert("p1".to_string());

    let mut state = ListState::default();
    state.select(Some(3)); // BrowsePlugins index
    let mut model = Model::MarketDetail {
        marketplace_name: "mp-a".to_string(),
        state,
        error_message: None,
        browse_plugins: Some(preserved_plugins),
        browse_selected: Some(preserved_selected),
    };

    update(&mut model, Msg::Enter, &mut data);

    assert_eq!(model_variant(&model), "PluginBrowse");
    if let Model::PluginBrowse {
        selected_plugins,
        plugins,
        ..
    } = &model
    {
        assert!(
            selected_plugins.contains("p1"),
            "Should restore selected p1"
        );
        assert_eq!(plugins.len(), 2, "Should restore 2 plugins");
    }
}

// ============================================================================
// Back ナビゲーション
// ============================================================================

#[test]
fn back_from_detail_returns_to_market_list() {
    let (_temp_dir, mut data) = make_data(&["mp-a", "mp-b"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
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
    let (_temp_dir, mut data) = make_data(&[]);
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
    let (_temp_dir, mut data) = make_data(&[]);
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
    let (_temp_dir, mut data) = make_data(&[]);
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
    let (_temp_dir, mut data) = make_data(&[]);
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
    let (_temp_dir, mut data) = make_data(&[]);
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
    let (_temp_dir, mut data) = make_data(&[]);
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
    let (_temp_dir, mut data) = make_data(&[]);
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
    let (_temp_dir, mut data) = make_data(&[]);
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
    let (_temp_dir, mut data) = make_data(&["existing"]);
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
    let (_temp_dir, mut data) = make_data(&[]);
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
    let (_temp_dir, mut data) = make_data(&[]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a", "mp-b"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a", "mp-b"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a", "mp-b"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
    let mut model = Model::new(&data);

    // Move to Add new
    update(&mut model, Msg::Down, &mut data);
    let effect = update(&mut model, Msg::UpdateMarket, &mut data);

    assert!(effect.phase2_msg.is_none(), "Should not trigger on Add new");
}

#[test]
fn update_market_ignored_when_operation_in_progress() {
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a", "mp-b"]);
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
    let (_temp_dir, mut data) = make_data(&[]);
    let mut model = Model::new(&data);

    let effect = update(&mut model, Msg::UpdateAll, &mut data);

    assert!(effect.phase2_msg.is_none());
}

#[test]
fn update_all_ignored_when_operation_in_progress() {
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a", "mp-b"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a", "mp-b"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
    let mut model = Model::MarketDetail {
        marketplace_name: "mp-a".to_string(),
        state: {
            let mut s = ListState::default();
            s.select(Some(0));
            s
        },
        error_message: Some("some error".to_string()),
        browse_plugins: None,
        browse_selected: None,
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
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a"]);
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
    let (_temp_dir, mut data) = make_data(&["mp-a", "mp-b"]);
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
        Model::PluginBrowse { .. } => "PluginBrowse",
        Model::TargetSelect { .. } => "TargetSelect",
        Model::ScopeSelect { .. } => "ScopeSelect",
        Model::Installing { .. } => "Installing",
        Model::InstallResult { .. } => "InstallResult",
    }
}
