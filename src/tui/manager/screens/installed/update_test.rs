use super::update;
use crate::application::PluginSummary;
use crate::tui::manager::core::DataStore;
use crate::tui::manager::screens::installed::model::{Model, Msg, UpdateStatusDisplay};

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
// BatchUpdate テスト
// ============================================================================

#[test]
fn batch_update_with_no_marks_is_noop() {
    let mut data = make_data(&["plugin-a"]);
    let mut model = Model::new(&data);

    let effect = update(&mut model, Msg::BatchUpdate, &mut data, "");

    assert!(
        !effect.needs_execute_batch,
        "Should not trigger batch when no marks"
    );
    assert!(!effect.should_focus_filter);

    if let Model::PluginList {
        update_statuses, ..
    } = &model
    {
        assert!(update_statuses.is_empty());
    } else {
        panic!("Expected PluginList");
    }
}

#[test]
fn batch_update_phase1_sets_updating_and_returns_execute_batch() {
    let mut data = make_data(&["plugin-a", "plugin-b", "plugin-c"]);
    let mut model = Model::new(&data);

    // plugin-a と plugin-c をマーク
    update(&mut model, Msg::ToggleMark, &mut data, ""); // mark plugin-a
    update(&mut model, Msg::Down, &mut data, ""); // move to plugin-b
    update(&mut model, Msg::Down, &mut data, ""); // move to plugin-c
    update(&mut model, Msg::ToggleMark, &mut data, ""); // mark plugin-c

    // Phase 1: BatchUpdate
    let effect = update(&mut model, Msg::BatchUpdate, &mut data, "");

    assert!(
        effect.needs_execute_batch,
        "BatchUpdate with marks should request execute_batch"
    );
    assert!(!effect.should_focus_filter);

    if let Model::PluginList {
        update_statuses, ..
    } = &model
    {
        assert!(
            matches!(
                update_statuses.get("plugin-a"),
                Some(UpdateStatusDisplay::Updating)
            ),
            "plugin-a should be Updating"
        );
        assert!(
            !update_statuses.contains_key("plugin-b"),
            "plugin-b should have no status (not marked)"
        );
        assert!(
            matches!(
                update_statuses.get("plugin-c"),
                Some(UpdateStatusDisplay::Updating)
            ),
            "plugin-c should be Updating"
        );
    } else {
        panic!("Expected PluginList");
    }
}

#[test]
fn batch_update_phase1_clears_stale_statuses() {
    let mut data = make_data(&["plugin-a", "plugin-b"]);
    let mut model = Model::new(&data);

    // plugin-a をマークして Phase 1 実行（stale ステータスを作る）
    update(&mut model, Msg::ToggleMark, &mut data, "");
    update(&mut model, Msg::BatchUpdate, &mut data, "");

    // 手動で stale ステータスを残す（Phase 2 後の状態をシミュレート）
    if let Model::PluginList {
        update_statuses,
        marked_ids,
        ..
    } = &mut model
    {
        update_statuses.insert("plugin-a".to_string(), UpdateStatusDisplay::Updated);
        marked_ids.clear();
        // plugin-b のみマーク
        marked_ids.insert("plugin-b".to_string());
    }

    // 新しい BatchUpdate: plugin-a の stale ステータスがクリアされるべき
    let effect = update(&mut model, Msg::BatchUpdate, &mut data, "");

    assert!(effect.needs_execute_batch);

    if let Model::PluginList {
        update_statuses, ..
    } = &model
    {
        assert!(
            !update_statuses.contains_key("plugin-a"),
            "Stale status for plugin-a should be cleared"
        );
        assert!(
            matches!(
                update_statuses.get("plugin-b"),
                Some(UpdateStatusDisplay::Updating)
            ),
            "plugin-b should be Updating"
        );
    } else {
        panic!("Expected PluginList");
    }
}
