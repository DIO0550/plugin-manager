use crossterm::event::KeyCode;

use super::app::{Model, Msg, Screen, ScreenCache};
use super::data::DataStore;
use crate::tui::manager::screens::installed;

/// テスト用の最小構成 Model を構築するヘルパー
fn make_model(filter_focused: bool, top_level: bool) -> Model {
    let data = DataStore::for_test(vec![], vec![], None);

    let screen = if top_level {
        Screen::Installed(installed::Model::new(&data))
    } else {
        // サブ画面: PluginDetail は is_top_level == false
        Screen::Installed(installed::Model::PluginDetail {
            plugin_id: "dummy".to_string(),
            state: ratatui::widgets::ListState::default(),
            saved_marked_ids: Default::default(),
            saved_update_statuses: Default::default(),
        })
    };

    Model {
        data,
        screen,
        cache: ScreenCache::default(),
        should_quit: false,
        filter_text: String::new(),
        filter_focused,
    }
}

// ============================================================================
// Right / Left キー — トップレベル
// ============================================================================

#[test]
fn right_key_returns_next_tab_at_top_level() {
    let model = make_model(false, true);
    let msg = model.key_to_msg(KeyCode::Right);
    assert!(matches!(msg, Some(Msg::NextTab)));
}

#[test]
fn left_key_returns_prev_tab_at_top_level() {
    let model = make_model(false, true);
    let msg = model.key_to_msg(KeyCode::Left);
    assert!(matches!(msg, Some(Msg::PrevTab)));
}

// ============================================================================
// Right / Left キー — サブ画面 (is_top_level == false)
// ============================================================================

#[test]
fn right_key_returns_none_at_sub_screen() {
    let model = make_model(false, false);
    let msg = model.key_to_msg(KeyCode::Right);
    assert!(msg.is_none());
}

#[test]
fn left_key_returns_none_at_sub_screen() {
    let model = make_model(false, false);
    let msg = model.key_to_msg(KeyCode::Left);
    assert!(msg.is_none());
}

// ============================================================================
// Right / Left キー — フィルタフォーカス中
// ============================================================================

#[test]
fn right_key_returns_next_tab_when_filter_focused() {
    let model = make_model(true, true);
    let msg = model.key_to_msg(KeyCode::Right);
    assert!(matches!(msg, Some(Msg::NextTab)));
}

#[test]
fn left_key_returns_prev_tab_when_filter_focused() {
    let model = make_model(true, true);
    let msg = model.key_to_msg(KeyCode::Left);
    assert!(matches!(msg, Some(Msg::PrevTab)));
}

// ============================================================================
// Tab / BackTab — 回帰テスト
// ============================================================================

#[test]
fn tab_key_still_returns_next_tab() {
    let model = make_model(false, true);
    let msg = model.key_to_msg(KeyCode::Tab);
    assert!(matches!(msg, Some(Msg::NextTab)));
}

#[test]
fn backtab_key_still_returns_prev_tab() {
    let model = make_model(false, true);
    let msg = model.key_to_msg(KeyCode::BackTab);
    assert!(matches!(msg, Some(Msg::PrevTab)));
}
