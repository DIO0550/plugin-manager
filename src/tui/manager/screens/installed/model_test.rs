use crossterm::event::KeyCode;

use super::{key_to_msg, CacheState, Model, Msg};
use crate::application::PluginSummary;
use crate::tui::manager::core::DataStore;
use std::collections::HashSet;

fn make_plugin(name: &str) -> PluginSummary {
    PluginSummary {
        name: name.to_string(),
        marketplace: Some("github".to_string()),
        version: "1.0.0".to_string(),
        skills: vec![],
        agents: vec![],
        commands: vec![],
        instructions: vec![],
        hooks: vec![],
        enabled: true,
    }
}

fn make_data(names: &[&str]) -> DataStore {
    DataStore {
        plugins: names.iter().map(|n| make_plugin(n)).collect(),
        last_error: None,
    }
}

// ============================================================================
// key_to_msg テスト
// ============================================================================

#[test]
fn space_key_returns_toggle_mark() {
    let msg = key_to_msg(KeyCode::Char(' '));
    assert!(matches!(msg, Some(Msg::ToggleMark)));
}

#[test]
fn a_key_returns_toggle_all_marks() {
    let msg = key_to_msg(KeyCode::Char('a'));
    assert!(matches!(msg, Some(Msg::ToggleAllMarks)));
}

#[test]
fn shift_u_key_returns_batch_update() {
    let msg = key_to_msg(KeyCode::Char('U'));
    assert!(matches!(msg, Some(Msg::BatchUpdate)));
}

#[test]
fn existing_keys_still_work() {
    assert!(matches!(key_to_msg(KeyCode::Up), Some(Msg::Up)));
    assert!(matches!(key_to_msg(KeyCode::Down), Some(Msg::Down)));
    assert!(matches!(key_to_msg(KeyCode::Enter), Some(Msg::Enter)));
    assert!(matches!(key_to_msg(KeyCode::Esc), Some(Msg::Back)));
}

// ============================================================================
// CacheState マーク状態保持テスト
// ============================================================================

#[test]
fn to_cache_preserves_marked_ids() {
    let data = make_data(&["plugin-a", "plugin-b"]);
    let mut model = Model::new(&data);

    // マーク状態を設定
    if let Model::PluginList { marked_ids, .. } = &mut model {
        marked_ids.insert("plugin-a".to_string());
    }

    let cache = model.to_cache();
    assert!(cache.marked_ids.contains("plugin-a"));
    assert!(!cache.marked_ids.contains("plugin-b"));
}

#[test]
fn from_cache_restores_marked_ids() {
    let data = make_data(&["plugin-a", "plugin-b"]);
    let mut marked = HashSet::new();
    marked.insert("plugin-a".to_string());

    let cache = CacheState {
        selected_plugin_id: None,
        marked_ids: marked,
    };

    let model = Model::from_cache(&data, &cache);
    if let Model::PluginList { marked_ids, .. } = &model {
        assert!(marked_ids.contains("plugin-a"));
        assert!(!marked_ids.contains("plugin-b"));
    } else {
        panic!("Expected PluginList");
    }
}

// ============================================================================
// CacheState 復元時の不在プラグイン除外テスト
// ============================================================================

#[test]
fn from_cache_excludes_missing_plugins_from_marked_ids() {
    let data = make_data(&["plugin-a"]);
    let mut marked = HashSet::new();
    marked.insert("plugin-a".to_string());
    marked.insert("deleted-plugin".to_string());

    let cache = CacheState {
        selected_plugin_id: None,
        marked_ids: marked,
    };

    let model = Model::from_cache(&data, &cache);
    if let Model::PluginList { marked_ids, .. } = &model {
        assert!(marked_ids.contains("plugin-a"));
        assert!(
            !marked_ids.contains("deleted-plugin"),
            "Deleted plugin should be excluded from restored marks"
        );
    } else {
        panic!("Expected PluginList");
    }
}
