use super::*;

#[test]
fn test_restore_repo_from_source_repo() {
    let mut meta = PluginMeta::default();
    meta.set_source_repo("owner", "repo");

    let repo = restore_repo(&meta, "any-name", "main").unwrap();
    assert_eq!(repo.owner(), "owner");
    assert_eq!(repo.name(), "repo");
    assert_eq!(repo.git_ref(), Some("main"));
}

#[test]
fn test_restore_repo_from_plugin_name() {
    let meta = PluginMeta::default();

    let repo = restore_repo(&meta, "owner--repo", "main").unwrap();
    assert_eq!(repo.owner(), "owner");
    assert_eq!(repo.name(), "repo");
    assert_eq!(repo.git_ref(), Some("main"));
}

#[test]
fn test_restore_repo_invalid_name() {
    let meta = PluginMeta::default();

    let result = restore_repo(&meta, "invalid-name", "main");
    assert!(result.is_err());
}

#[test]
fn test_update_result_factories() {
    let updated = UpdateResult::updated(
        "test",
        Some("abc123".to_string()),
        "def456".to_string(),
        vec!["codex".to_string()],
        vec![],
    );
    assert!(matches!(updated.status, UpdateStatus::Updated { .. }));

    let up_to_date = UpdateResult::up_to_date("test");
    assert!(matches!(up_to_date.status, UpdateStatus::AlreadyUpToDate));

    let failed = UpdateResult::failed("test", "error".to_string());
    assert!(matches!(failed.status, UpdateStatus::Failed));

    let skipped = UpdateResult::skipped("test", "reason".to_string());
    assert!(matches!(skipped.status, UpdateStatus::Skipped { .. }));
}
