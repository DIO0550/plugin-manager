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

fn make_data(names: &[&str]) -> DataStore {
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
    let data = make_data(&["market-a", "market-b"]);
    let model = Model::new(&data);
    if let Model::MarketList {
        selected_id, state, ..
    } = &model
    {
        assert_eq!(selected_id.as_deref(), Some("market-a"));
        assert_eq!(state.selected(), Some(0));
    } else {
        panic!("Expected MarketList");
    }
}

#[test]
fn new_with_empty_marketplaces() {
    let data = make_data(&[]);
    let model = Model::new(&data);
    if let Model::MarketList {
        selected_id, state, ..
    } = &model
    {
        assert_eq!(selected_id, &None);
        assert_eq!(state.selected(), Some(0)); // "+ Add new" is selected
    } else {
        panic!("Expected MarketList");
    }
}

// ============================================================================
// is_top_level テスト
// ============================================================================

#[test]
fn is_top_level_market_list() {
    let data = make_data(&[]);
    let model = Model::new(&data);
    assert!(model.is_top_level());
}

#[test]
fn is_top_level_market_detail() {
    let model = Model::MarketDetail {
        marketplace_name: "test".to_string(),
        state: ListState::default(),
        error_message: None,
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
    let data = make_data(&[]);
    let model = Model::new(&data);
    assert!(!model.is_form_active());
}

// ============================================================================
// key_to_msg テスト
// ============================================================================

#[test]
fn key_to_msg_market_list_navigation() {
    let data = make_data(&["market-a"]);
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
// from_cache / to_cache テスト
// ============================================================================

#[test]
fn to_cache_and_from_cache_round_trip() {
    let data = make_data(&["market-a", "market-b"]);
    let model = Model::new(&data);
    let cache = model.to_cache();
    assert_eq!(cache.selected_id.as_deref(), Some("market-a"));

    let restored = Model::from_cache(&data, &cache);
    if let Model::MarketList {
        selected_id, state, ..
    } = &restored
    {
        assert_eq!(selected_id.as_deref(), Some("market-a"));
        assert_eq!(state.selected(), Some(0));
    } else {
        panic!("Expected MarketList");
    }
}

#[test]
fn from_cache_with_stale_id() {
    let data = make_data(&["market-a"]);
    let cache = CacheState {
        selected_id: Some("deleted-market".to_string()),
    };
    let model = Model::from_cache(&data, &cache);
    if let Model::MarketList { selected_id, .. } = &model {
        assert_eq!(selected_id.as_deref(), Some("market-a"));
    } else {
        panic!("Expected MarketList");
    }
}
