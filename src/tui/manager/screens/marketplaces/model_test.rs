use super::*;
use crate::tui::manager::core::{DataStore, MarketplaceItem};
use crossterm::event::KeyCode;

fn make_marketplace(name: &str) -> MarketplaceItem {
    MarketplaceItem {
        name: name.to_string(),
        source: format!("owner/{}", name),
        source_path: None,
        plugin_count: Some(3),
        last_updated: Some("2026-01-01 12:00".to_string()),
    }
}

fn make_data(names: &[&str]) -> (tempfile::TempDir, DataStore) {
    DataStore::for_test(
        vec![],
        names.iter().map(|n| make_marketplace(n)).collect(),
        None,
    )
}

// ============================================================================
// Model::new テスト
// ============================================================================

#[test]
fn new_with_marketplaces_selects_first() {
    let (_temp_dir, data) = make_data(&["market-a", "market-b"]);
    let model = Model::new(&data);
    if let Model::MarketList { selection, .. } = &model {
        assert_eq!(
            selection.selected_id().map(String::as_str),
            Some("market-a")
        );
        assert_eq!(selection.selected_index(), Some(0));
    } else {
        panic!("Expected MarketList");
    }
}

#[test]
fn new_with_empty_marketplaces() {
    let (_temp_dir, data) = make_data(&[]);
    let model = Model::new(&data);
    if let Model::MarketList { selection, .. } = &model {
        assert_eq!(selection.selected_id(), None);
        assert_eq!(selection.selected_index(), Some(0)); // "+ Add new" is selected
    } else {
        panic!("Expected MarketList");
    }
}

// ============================================================================
// is_top_level テスト
// ============================================================================

#[test]
fn is_top_level_market_list() {
    let (_temp_dir, data) = make_data(&[]);
    let model = Model::new(&data);
    assert!(model.is_top_level());
}

#[test]
fn is_top_level_market_detail() {
    let model = Model::MarketDetail {
        marketplace_name: "test".to_string(),
        state: ListState::default(),
        error_message: None,
        browse_plugins: None,
        browse_selected: None,
    };
    assert!(!model.is_top_level());
}

#[test]
fn is_top_level_add_form() {
    let model = Model::AddForm(AddFormModel::Source {
        source_input: String::new(),
        error_message: None,
    });
    assert!(!model.is_top_level());
}

// ============================================================================
// is_form_active テスト
// ============================================================================

#[test]
fn is_form_active_add_form() {
    let model = Model::AddForm(AddFormModel::Source {
        source_input: String::new(),
        error_message: None,
    });
    assert!(model.is_form_active());
}

#[test]
fn is_form_active_market_list() {
    let (_temp_dir, data) = make_data(&[]);
    let model = Model::new(&data);
    assert!(!model.is_form_active());
}

// ============================================================================
// key_to_msg テスト
// ============================================================================

#[test]
fn key_to_msg_market_list_navigation() {
    let (_temp_dir, data) = make_data(&["market-a"]);
    let model = Model::new(&data);

    assert!(matches!(key_to_msg(KeyCode::Up, &model), Some(Msg::Up)));
    assert!(matches!(key_to_msg(KeyCode::Down, &model), Some(Msg::Down)));
    assert!(matches!(
        key_to_msg(KeyCode::Char('k'), &model),
        Some(Msg::Up)
    ));
    assert!(matches!(
        key_to_msg(KeyCode::Char('j'), &model),
        Some(Msg::Down)
    ));
    assert!(matches!(
        key_to_msg(KeyCode::Enter, &model),
        Some(Msg::Enter)
    ));
    assert!(matches!(key_to_msg(KeyCode::Esc, &model), Some(Msg::Back)));
    assert!(matches!(
        key_to_msg(KeyCode::Char('u'), &model),
        Some(Msg::UpdateMarket)
    ));
    assert!(matches!(
        key_to_msg(KeyCode::Char('U'), &model),
        Some(Msg::UpdateAll)
    ));
}

#[test]
fn key_to_msg_add_form_input() {
    let model = Model::AddForm(AddFormModel::Source {
        source_input: String::new(),
        error_message: None,
    });

    assert!(matches!(
        key_to_msg(KeyCode::Char('a'), &model),
        Some(Msg::FormInput('a'))
    ));
    assert!(matches!(
        key_to_msg(KeyCode::Backspace, &model),
        Some(Msg::FormBackspace)
    ));
    assert!(matches!(
        key_to_msg(KeyCode::Enter, &model),
        Some(Msg::Enter)
    ));
    assert!(matches!(key_to_msg(KeyCode::Esc, &model), Some(Msg::Back)));
    // In form mode, 'q' is treated as form input, not quit
    assert!(matches!(
        key_to_msg(KeyCode::Char('q'), &model),
        Some(Msg::FormInput('q'))
    ));
}

// ============================================================================
// key_to_msg: PluginBrowse テスト
// ============================================================================

fn make_plugin_browse_model() -> Model {
    let mut state = ListState::default();
    state.select(Some(0));
    Model::PluginBrowse {
        marketplace_name: "test".to_string(),
        plugins: vec![BrowsePlugin {
            name: "p1".to_string(),
            description: None,
            version: None,
            source: crate::marketplace::PluginSource::Local("local".to_string()),
            installed: false,
        }],
        selected_plugins: HashSet::new(),
        highlighted_idx: 0,
        state,
    }
}

#[test]
fn key_to_msg_plugin_browse_space_returns_toggle_select() {
    let model = make_plugin_browse_model();
    assert!(matches!(
        key_to_msg(KeyCode::Char(' '), &model),
        Some(Msg::ToggleSelect)
    ));
}

#[test]
fn key_to_msg_plugin_browse_i_returns_start_install() {
    let model = make_plugin_browse_model();
    assert!(matches!(
        key_to_msg(KeyCode::Char('i'), &model),
        Some(Msg::StartInstall)
    ));
}

#[test]
fn key_to_msg_plugin_browse_enter_returns_start_install() {
    let model = make_plugin_browse_model();
    assert!(matches!(
        key_to_msg(KeyCode::Enter, &model),
        Some(Msg::StartInstall)
    ));
}

#[test]
fn key_to_msg_plugin_browse_navigation() {
    let model = make_plugin_browse_model();
    assert!(matches!(
        key_to_msg(KeyCode::Char('j'), &model),
        Some(Msg::Down)
    ));
    assert!(matches!(
        key_to_msg(KeyCode::Char('k'), &model),
        Some(Msg::Up)
    ));
    assert!(matches!(key_to_msg(KeyCode::Down, &model), Some(Msg::Down)));
    assert!(matches!(key_to_msg(KeyCode::Up, &model), Some(Msg::Up)));
    assert!(matches!(key_to_msg(KeyCode::Esc, &model), Some(Msg::Back)));
}

// ============================================================================
// key_to_msg: TargetSelect テスト
// ============================================================================

fn make_target_select_model() -> Model {
    let mut state = ListState::default();
    state.select(Some(0));
    Model::TargetSelect {
        marketplace_name: "test".to_string(),
        plugins: vec![],
        selected_plugins: HashSet::new(),
        targets: vec![
            ("codex".to_string(), "OpenAI Codex".to_string(), false),
            ("copilot".to_string(), "VS Code Copilot".to_string(), false),
        ],
        highlighted_idx: 0,
        state,
    }
}

#[test]
fn key_to_msg_target_select_space_returns_toggle_select() {
    let model = make_target_select_model();
    assert!(matches!(
        key_to_msg(KeyCode::Char(' '), &model),
        Some(Msg::ToggleSelect)
    ));
}

#[test]
fn key_to_msg_target_select_enter_returns_confirm_targets() {
    let model = make_target_select_model();
    assert!(matches!(
        key_to_msg(KeyCode::Enter, &model),
        Some(Msg::ConfirmTargets)
    ));
}

#[test]
fn key_to_msg_target_select_esc_returns_back() {
    let model = make_target_select_model();
    assert!(matches!(key_to_msg(KeyCode::Esc, &model), Some(Msg::Back)));
}

// ============================================================================
// key_to_msg: ScopeSelect テスト
// ============================================================================

fn make_scope_select_model() -> Model {
    let mut state = ListState::default();
    state.select(Some(0));
    Model::ScopeSelect {
        marketplace_name: "test".to_string(),
        plugins: vec![],
        selected_plugins: HashSet::new(),
        target_names: vec!["codex".to_string()],
        highlighted_idx: 0,
        state,
    }
}

#[test]
fn key_to_msg_scope_select_enter_returns_confirm_scope() {
    let model = make_scope_select_model();
    assert!(matches!(
        key_to_msg(KeyCode::Enter, &model),
        Some(Msg::ConfirmScope)
    ));
}

#[test]
fn key_to_msg_scope_select_esc_returns_back() {
    let model = make_scope_select_model();
    assert!(matches!(key_to_msg(KeyCode::Esc, &model), Some(Msg::Back)));
}

// ============================================================================
// key_to_msg: Installing テスト
// ============================================================================

#[test]
fn key_to_msg_installing_returns_none() {
    let model = Model::Installing {
        marketplace_name: "test".to_string(),
        plugins: vec![],
        plugin_names: vec!["p1".to_string()],
        target_names: vec!["codex".to_string()],
        scope: crate::component::Scope::Personal,
        current_idx: 0,
        total: 1,
    };

    assert!(key_to_msg(KeyCode::Enter, &model).is_none());
    assert!(key_to_msg(KeyCode::Esc, &model).is_none());
    assert!(key_to_msg(KeyCode::Up, &model).is_none());
    assert!(key_to_msg(KeyCode::Down, &model).is_none());
    assert!(key_to_msg(KeyCode::Char(' '), &model).is_none());
    assert!(key_to_msg(KeyCode::Char('i'), &model).is_none());
}

// ============================================================================
// key_to_msg: InstallResult テスト
// ============================================================================

#[test]
fn key_to_msg_install_result_enter_returns_back_to_browse() {
    let model = Model::InstallResult {
        marketplace_name: "test".to_string(),
        plugins: vec![],
        summary: super::InstallSummary {
            results: vec![],
            total: 0,
            succeeded: 0,
            failed: 0,
        },
    };

    assert!(matches!(
        key_to_msg(KeyCode::Enter, &model),
        Some(Msg::BackToPluginBrowse)
    ));
}

#[test]
fn key_to_msg_install_result_esc_returns_back_to_browse() {
    let model = Model::InstallResult {
        marketplace_name: "test".to_string(),
        plugins: vec![],
        summary: super::InstallSummary {
            results: vec![],
            total: 0,
            succeeded: 0,
            failed: 0,
        },
    };

    assert!(matches!(
        key_to_msg(KeyCode::Esc, &model),
        Some(Msg::BackToPluginBrowse)
    ));
}

// ============================================================================
// MarketDetail browse state preservation テスト
// ============================================================================

#[test]
fn market_detail_holds_browse_state() {
    let plugins = vec![BrowsePlugin {
        name: "p1".to_string(),
        description: None,
        version: None,
        source: crate::marketplace::PluginSource::Local("local".to_string()),
        installed: false,
    }];
    let mut selected = HashSet::new();
    selected.insert("p1".to_string());

    let model = Model::MarketDetail {
        marketplace_name: "test".to_string(),
        state: ListState::default(),
        error_message: None,
        browse_plugins: Some(plugins),
        browse_selected: Some(selected),
    };

    if let Model::MarketDetail {
        browse_plugins,
        browse_selected,
        ..
    } = &model
    {
        assert!(browse_plugins.is_some());
        assert!(browse_selected.as_ref().unwrap().contains("p1"));
    } else {
        panic!("Expected MarketDetail");
    }
}

// ============================================================================
// from_cache / to_cache テスト
// ============================================================================

#[test]
fn to_cache_and_from_cache_round_trip() {
    let (_temp_dir, data) = make_data(&["market-a", "market-b"]);
    let model = Model::new(&data);
    let cache = model.to_cache();
    assert_eq!(cache.selected_id.as_deref(), Some("market-a"));

    let restored = Model::from_cache(&data, &cache);
    if let Model::MarketList { selection, .. } = &restored {
        assert_eq!(
            selection.selected_id().map(String::as_str),
            Some("market-a")
        );
        assert_eq!(selection.selected_index(), Some(0));
    } else {
        panic!("Expected MarketList");
    }
}

#[test]
fn from_cache_with_stale_id() {
    let (_temp_dir, data) = make_data(&["market-a"]);
    let cache = CacheState {
        selected_id: Some("deleted-market".to_string()),
    };
    let model = Model::from_cache(&data, &cache);
    if let Model::MarketList { selection, .. } = &model {
        assert_eq!(
            selection.selected_id().map(String::as_str),
            Some("market-a")
        );
    } else {
        panic!("Expected MarketList");
    }
}
