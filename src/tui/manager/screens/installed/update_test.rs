use super::{execute_batch_with, update};
use crate::application::PluginSummary;
use crate::tui::manager::core::DataStore;
use crate::tui::manager::screens::installed::model::{Model, Msg, UpdateStatusDisplay};

/// スタブ: 全プラグインを Updated として返す
fn stub_run_updates(names: &[String]) -> Vec<(String, UpdateStatusDisplay)> {
    names
        .iter()
        .map(|name| (name.clone(), UpdateStatusDisplay::Updated))
        .collect()
}

/// スタブ: reload を何もしない（プラグインリストを維持）
fn stub_reload(_data: &mut DataStore) -> std::io::Result<()> {
    Ok(())
}

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
        marketplaces: vec![],
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

// ============================================================================
// UpdateNow テスト
// ============================================================================

#[test]
fn update_now_transitions_to_plugin_list_with_updating_status() {
    let mut data = make_data(&["plugin-a", "plugin-b"]);
    let mut model = Model::new(&data);

    // PluginList → PluginDetail に遷移
    update(&mut model, Msg::Enter, &mut data, "");
    assert!(matches!(model, Model::PluginDetail { .. }));

    // PluginDetail で UpdateNow のインデックスに移動
    // for_enabled(): [Disable, MarkForUpdate, UpdateNow, Uninstall, ViewComponents, Back]
    // UpdateNow はインデックス 2
    update(&mut model, Msg::Down, &mut data, ""); // index 1
    update(&mut model, Msg::Down, &mut data, ""); // index 2 (UpdateNow)

    // Enter で UpdateNow を実行
    let effect = update(&mut model, Msg::Enter, &mut data, "");

    // PluginList に遷移していること
    assert!(
        matches!(model, Model::PluginList { .. }),
        "Should transition to PluginList"
    );

    // needs_execute_batch が true であること
    assert!(
        effect.needs_execute_batch,
        "UpdateNow should request execute_batch"
    );

    // 対象プラグインに Updating がセットされていること
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
        assert_eq!(
            update_statuses.len(),
            1,
            "Only the target plugin should have Updating status"
        );
    } else {
        panic!("Expected PluginList");
    }
}

#[test]
fn update_now_restores_saved_marks() {
    let mut data = make_data(&["plugin-a", "plugin-b", "plugin-c"]);
    let mut model = Model::new(&data);

    // plugin-a をマーク
    update(&mut model, Msg::ToggleMark, &mut data, "");

    // PluginList → PluginDetail (plugin-a) に遷移
    update(&mut model, Msg::Enter, &mut data, "");

    // UpdateNow のインデックスに移動 (index 2)
    update(&mut model, Msg::Down, &mut data, ""); // index 1
    update(&mut model, Msg::Down, &mut data, ""); // index 2 (UpdateNow)

    // Enter で UpdateNow を実行
    update(&mut model, Msg::Enter, &mut data, "");

    // マーク状態が復元されていること
    if let Model::PluginList { marked_ids, .. } = &model {
        assert!(
            marked_ids.contains("plugin-a"),
            "plugin-a mark should be restored"
        );
    } else {
        panic!("Expected PluginList");
    }
}

#[test]
fn update_now_returns_execute_batch_effect() {
    let mut data = make_data(&["plugin-a"]);
    let mut model = Model::new(&data);

    // PluginList → PluginDetail に遷移
    update(&mut model, Msg::Enter, &mut data, "");

    // UpdateNow のインデックスに移動 (index 2)
    update(&mut model, Msg::Down, &mut data, ""); // index 1
    update(&mut model, Msg::Down, &mut data, ""); // index 2 (UpdateNow)

    let effect = update(&mut model, Msg::Enter, &mut data, "");

    assert!(
        effect.needs_execute_batch,
        "UpdateNow should return execute_batch effect"
    );
    assert!(!effect.should_focus_filter);
}

#[test]
fn update_now_clears_stale_statuses() {
    let mut data = make_data(&["plugin-a", "plugin-b"]);
    let mut model = Model::new(&data);

    // PluginList → PluginDetail に遷移
    update(&mut model, Msg::Enter, &mut data, "");

    // saved_update_statuses に stale ステータスを注入
    if let Model::PluginDetail {
        saved_update_statuses,
        ..
    } = &mut model
    {
        saved_update_statuses.insert("plugin-b".to_string(), UpdateStatusDisplay::Updated);
    }

    // UpdateNow のインデックスに移動 (index 2)
    update(&mut model, Msg::Down, &mut data, ""); // index 1
    update(&mut model, Msg::Down, &mut data, ""); // index 2 (UpdateNow)

    update(&mut model, Msg::Enter, &mut data, "");

    if let Model::PluginList {
        update_statuses, ..
    } = &model
    {
        assert!(
            !update_statuses.contains_key("plugin-b"),
            "Stale status for plugin-b should be cleared"
        );
        assert!(
            matches!(
                update_statuses.get("plugin-a"),
                Some(UpdateStatusDisplay::Updating)
            ),
            "plugin-a should be Updating"
        );
    } else {
        panic!("Expected PluginList");
    }
}

// ============================================================================
// UpdateAll テスト
// ============================================================================

#[test]
fn update_all_sets_all_plugins_to_updating() {
    let mut data = make_data(&["plugin-a", "plugin-b", "plugin-c"]);
    let mut model = Model::new(&data);

    let effect = update(&mut model, Msg::UpdateAll, &mut data, "");

    assert!(
        effect.needs_execute_batch,
        "UpdateAll should request execute_batch"
    );

    if let Model::PluginList {
        update_statuses, ..
    } = &model
    {
        assert_eq!(update_statuses.len(), 3);
        assert!(matches!(
            update_statuses.get("plugin-a"),
            Some(UpdateStatusDisplay::Updating)
        ));
        assert!(matches!(
            update_statuses.get("plugin-b"),
            Some(UpdateStatusDisplay::Updating)
        ));
        assert!(matches!(
            update_statuses.get("plugin-c"),
            Some(UpdateStatusDisplay::Updating)
        ));
    } else {
        panic!("Expected PluginList");
    }
}

#[test]
fn update_all_ignores_filter() {
    let mut data = make_data(&["alpha", "beta", "gamma"]);
    let mut model = Model::new(&data);

    // フィルタありでも全プラグインが Updating になること
    let effect = update(&mut model, Msg::UpdateAll, &mut data, "alpha");

    assert!(effect.needs_execute_batch);

    if let Model::PluginList {
        update_statuses, ..
    } = &model
    {
        assert_eq!(
            update_statuses.len(),
            3,
            "All plugins should be Updating regardless of filter"
        );
        assert!(update_statuses.contains_key("alpha"));
        assert!(update_statuses.contains_key("beta"));
        assert!(update_statuses.contains_key("gamma"));
    } else {
        panic!("Expected PluginList");
    }
}

#[test]
fn update_all_on_empty_list_does_nothing() {
    let mut data = make_data(&[]);
    let mut model = Model::new(&data);

    let effect = update(&mut model, Msg::UpdateAll, &mut data, "");

    assert!(
        !effect.needs_execute_batch,
        "UpdateAll on empty list should not trigger execute_batch"
    );
}

#[test]
fn update_all_clears_stale_statuses() {
    let mut data = make_data(&["plugin-a", "plugin-b"]);
    let mut model = Model::new(&data);

    // stale ステータスを手動でセット
    if let Model::PluginList {
        update_statuses, ..
    } = &mut model
    {
        update_statuses.insert("plugin-a".to_string(), UpdateStatusDisplay::Updated);
    }

    let effect = update(&mut model, Msg::UpdateAll, &mut data, "");

    assert!(effect.needs_execute_batch);

    if let Model::PluginList {
        update_statuses, ..
    } = &model
    {
        // stale ステータスがクリアされ、全プラグインが Updating であること
        assert!(
            matches!(
                update_statuses.get("plugin-a"),
                Some(UpdateStatusDisplay::Updating)
            ),
            "plugin-a should be Updating, not stale Updated"
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

// ============================================================================
// execute_batch: stale marked_ids の正規化テスト（密閉テスト）
// ============================================================================

#[test]
fn execute_batch_removes_stale_marks_for_nonexistent_plugins() {
    let mut data = make_data(&["plugin-a", "plugin-b"]);
    let mut model = Model::new(&data);

    // plugin-a と "plugin-removed"（存在しない）をマーク
    // plugin-a のみ Updating
    if let Model::PluginList {
        marked_ids,
        update_statuses,
        ..
    } = &mut model
    {
        marked_ids.insert("plugin-a".to_string());
        marked_ids.insert("plugin-removed".to_string());
        update_statuses.insert("plugin-a".to_string(), UpdateStatusDisplay::Updating);
    }

    execute_batch_with(&mut model, &mut data, "", stub_run_updates, stub_reload);

    if let Model::PluginList { marked_ids, .. } = &model {
        assert!(
            !marked_ids.contains("plugin-removed"),
            "Stale mark for nonexistent plugin should be removed after reload"
        );
    } else {
        panic!("Expected PluginList");
    }
}

// ============================================================================
// execute_batch 変更のテスト（密閉テスト）
// ============================================================================

#[test]
fn execute_batch_collects_from_update_statuses() {
    let mut data = make_data(&["plugin-a", "plugin-b", "plugin-c"]);
    let mut model = Model::new(&data);

    // update_statuses に直接 Updating をセット（execute_batch はここから収集する）
    if let Model::PluginList {
        update_statuses, ..
    } = &mut model
    {
        update_statuses.insert("plugin-a".to_string(), UpdateStatusDisplay::Updating);
        update_statuses.insert("plugin-c".to_string(), UpdateStatusDisplay::Updating);
    }

    execute_batch_with(&mut model, &mut data, "", stub_run_updates, stub_reload);

    // plugin-a と plugin-c に結果が反映されていること（Updated になっている）
    if let Model::PluginList {
        update_statuses, ..
    } = &model
    {
        assert!(
            matches!(
                update_statuses.get("plugin-a"),
                Some(UpdateStatusDisplay::Updated)
            ),
            "plugin-a should be Updated"
        );
        assert!(
            matches!(
                update_statuses.get("plugin-c"),
                Some(UpdateStatusDisplay::Updated)
            ),
            "plugin-c should be Updated"
        );
        assert!(
            !update_statuses.contains_key("plugin-b"),
            "plugin-b should have no status (was not Updating)"
        );
    } else {
        panic!("Expected PluginList");
    }
}

#[test]
fn execute_batch_preserves_marks_when_not_all_marked_are_updated() {
    let mut data = make_data(&["plugin-a", "plugin-b", "plugin-c"]);
    let mut model = Model::new(&data);

    // plugin-a と plugin-b をマーク
    if let Model::PluginList {
        marked_ids,
        update_statuses,
        ..
    } = &mut model
    {
        marked_ids.insert("plugin-a".to_string());
        marked_ids.insert("plugin-b".to_string());
        // UpdateNow 経由: plugin-a のみ Updating（marked_ids は 2 つ）
        update_statuses.insert("plugin-a".to_string(), UpdateStatusDisplay::Updating);
    }

    execute_batch_with(&mut model, &mut data, "", stub_run_updates, stub_reload);

    if let Model::PluginList { marked_ids, .. } = &model {
        assert!(
            !marked_ids.is_empty(),
            "Marks should be preserved when not all marked plugins are updated"
        );
        assert!(
            marked_ids.contains("plugin-b"),
            "plugin-b mark should be preserved"
        );
    } else {
        panic!("Expected PluginList");
    }
}

#[test]
fn execute_batch_clears_marks_when_all_marked_are_updated() {
    let mut data = make_data(&["plugin-a", "plugin-b"]);
    let mut model = Model::new(&data);

    // plugin-a と plugin-b をマーク、両方 Updating
    if let Model::PluginList {
        marked_ids,
        update_statuses,
        ..
    } = &mut model
    {
        marked_ids.insert("plugin-a".to_string());
        marked_ids.insert("plugin-b".to_string());
        update_statuses.insert("plugin-a".to_string(), UpdateStatusDisplay::Updating);
        update_statuses.insert("plugin-b".to_string(), UpdateStatusDisplay::Updating);
    }

    execute_batch_with(&mut model, &mut data, "", stub_run_updates, stub_reload);

    if let Model::PluginList { marked_ids, .. } = &model {
        assert!(
            marked_ids.is_empty(),
            "Marks should be cleared when all marked plugins are updated"
        );
    } else {
        panic!("Expected PluginList");
    }
}

#[test]
fn execute_batch_sets_error_on_failed_updates() {
    let mut data = make_data(&["plugin-a"]);
    let mut model = Model::new(&data);

    if let Model::PluginList {
        update_statuses, ..
    } = &mut model
    {
        update_statuses.insert("plugin-a".to_string(), UpdateStatusDisplay::Updating);
    }

    // スタブ: 全プラグインを Failed として返す
    let fail_updates = |names: &[String]| -> Vec<(String, UpdateStatusDisplay)> {
        names
            .iter()
            .map(|name| {
                (
                    name.clone(),
                    UpdateStatusDisplay::Failed("test error".to_string()),
                )
            })
            .collect()
    };

    execute_batch_with(&mut model, &mut data, "", fail_updates, stub_reload);

    assert!(
        data.last_error.is_some(),
        "last_error should be set on failed update"
    );
    assert!(
        data.last_error.as_ref().unwrap().contains("test error"),
        "Error message should contain the failure reason"
    );

    if let Model::PluginList {
        update_statuses, ..
    } = &model
    {
        assert!(
            matches!(
                update_statuses.get("plugin-a"),
                Some(UpdateStatusDisplay::Failed(_))
            ),
            "plugin-a should be Failed"
        );
    } else {
        panic!("Expected PluginList");
    }
}

#[test]
fn execute_batch_handles_reload_error() {
    let mut data = make_data(&["plugin-a"]);
    let mut model = Model::new(&data);

    if let Model::PluginList {
        update_statuses, ..
    } = &mut model
    {
        update_statuses.insert("plugin-a".to_string(), UpdateStatusDisplay::Updating);
    }

    // スタブ: reload がエラーを返す
    let fail_reload = |_data: &mut DataStore| -> std::io::Result<()> {
        Err(std::io::Error::other("reload failed"))
    };

    execute_batch_with(&mut model, &mut data, "", stub_run_updates, fail_reload);

    assert!(
        data.last_error.is_some(),
        "last_error should be set on reload failure"
    );
    assert!(
        data.last_error.as_ref().unwrap().contains("reload failed"),
        "Error message should contain the reload failure reason"
    );
}
