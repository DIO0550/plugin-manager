use crate::tui::manager::screens::installed::model::{Model, Msg};
use super::update;
use crate::application::PluginSummary;
use crate::tui::manager::core::DataStore;

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
// ToggleMark テスト
// ============================================================================

#[test]
fn toggle_mark_adds_selected_plugin() {
    let mut data = make_data(&["plugin-a", "plugin-b"]);
    let mut model = Model::new(&data);
    let effect = update(&mut model, Msg::ToggleMark, &mut data, "");

    assert!(!effect.should_focus_filter);
    assert!(!effect.needs_execute_batch);

    if let Model::PluginList { marked_ids, .. } = &model {
        assert!(marked_ids.contains("plugin-a"));
    } else {
        panic!("Expected PluginList");
    }
}

#[test]
fn toggle_mark_removes_already_marked_plugin() {
    let mut data = make_data(&["plugin-a", "plugin-b"]);
    let mut model = Model::new(&data);

    // 1回目: マーク
    update(&mut model, Msg::ToggleMark, &mut data, "");
    // 2回目: マーク解除
    update(&mut model, Msg::ToggleMark, &mut data, "");

    if let Model::PluginList { marked_ids, .. } = &model {
        assert!(!marked_ids.contains("plugin-a"));
    } else {
        panic!("Expected PluginList");
    }
}

// ============================================================================
// ToggleAllMarks テスト
// ============================================================================

#[test]
fn toggle_all_marks_selects_all_filtered_plugins() {
    let mut data = make_data(&["plugin-a", "plugin-b", "plugin-c"]);
    let mut model = Model::new(&data);

    update(&mut model, Msg::ToggleAllMarks, &mut data, "");

    if let Model::PluginList { marked_ids, .. } = &model {
        assert!(marked_ids.contains("plugin-a"));
        assert!(marked_ids.contains("plugin-b"));
        assert!(marked_ids.contains("plugin-c"));
    } else {
        panic!("Expected PluginList");
    }
}

#[test]
fn toggle_all_marks_deselects_when_all_marked() {
    let mut data = make_data(&["plugin-a", "plugin-b"]);
    let mut model = Model::new(&data);

    // 全選択
    update(&mut model, Msg::ToggleAllMarks, &mut data, "");
    // 全解除
    update(&mut model, Msg::ToggleAllMarks, &mut data, "");

    if let Model::PluginList { marked_ids, .. } = &model {
        assert!(marked_ids.is_empty());
    } else {
        panic!("Expected PluginList");
    }
}

#[test]
fn toggle_all_marks_preserves_marks_outside_filter() {
    let mut data = make_data(&["alpha", "beta", "gamma"]);
    let mut model = Model::new(&data);

    // "alpha" を個別にマーク
    update(&mut model, Msg::ToggleMark, &mut data, "");

    // "beta" でフィルタして全選択（alpha はフィルタ外）
    update(&mut model, Msg::ToggleAllMarks, &mut data, "beta");

    if let Model::PluginList { marked_ids, .. } = &model {
        assert!(
            marked_ids.contains("alpha"),
            "alpha should remain marked (outside filter)"
        );
        assert!(
            marked_ids.contains("beta"),
            "beta should be marked (in filter)"
        );
        assert!(
            !marked_ids.contains("gamma"),
            "gamma should not be marked (outside filter, was not previously marked)"
        );
    } else {
        panic!("Expected PluginList");
    }
}

#[test]
fn toggle_all_marks_with_filter_only_deselects_filtered() {
    let mut data = make_data(&["alpha", "beta", "gamma"]);
    let mut model = Model::new(&data);

    // 全てマーク（フィルタなし）
    update(&mut model, Msg::ToggleAllMarks, &mut data, "");

    // "beta" でフィルタして全解除
    update(&mut model, Msg::ToggleAllMarks, &mut data, "beta");

    if let Model::PluginList { marked_ids, .. } = &model {
        assert!(
            marked_ids.contains("alpha"),
            "alpha should remain marked (outside filter)"
        );
        assert!(
            !marked_ids.contains("beta"),
            "beta should be unmarked (in filter, was toggled off)"
        );
        assert!(
            marked_ids.contains("gamma"),
            "gamma should remain marked (outside filter)"
        );
    } else {
        panic!("Expected PluginList");
    }
}

// ============================================================================
// BatchUpdate 空マーク時の no-op テスト
// ============================================================================

#[test]
fn batch_update_with_no_marks_is_noop() {
    let mut data = make_data(&["plugin-a"]);
    let mut model = Model::new(&data);

    let effect = update(&mut model, Msg::BatchUpdate, &mut data, "");

    assert!(!effect.needs_execute_batch, "Should not trigger batch when no marks");
    assert!(!effect.should_focus_filter);

    if let Model::PluginList { update_statuses, .. } = &model {
        assert!(update_statuses.is_empty());
    } else {
        panic!("Expected PluginList");
    }
}
