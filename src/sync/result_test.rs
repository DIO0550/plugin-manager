use super::*;
use crate::component::{ComponentKind, Scope};

fn make_component(name: &str) -> PlacedComponent {
    PlacedComponent::new(ComponentKind::Skill, name, Scope::Personal, "/path")
}

#[test]
fn test_sync_result_counts() {
    let result = SyncResult {
        created: vec![make_component("c1"), make_component("c2")],
        updated: vec![make_component("u1")],
        deleted: vec![make_component("d1")],
        skipped: vec![make_component("s1"), make_component("s2")],
        unsupported: vec![make_component("n1")],
        failed: vec![SyncFailure::new(
            make_component("f1"),
            SyncAction::Create,
            "error",
        )],
        dry_run: false,
    };

    assert_eq!(result.create_count(), 2);
    assert_eq!(result.update_count(), 1);
    assert_eq!(result.delete_count(), 1);
    assert_eq!(result.success_count(), 4);
    assert_eq!(result.skip_count(), 3);
    assert_eq!(result.failure_count(), 1);
    assert_eq!(result.total_count(), 8);
    assert!(!result.is_success());
}

#[test]
fn test_sync_result_empty() {
    let result = SyncResult::default();
    assert!(result.is_empty());
    assert!(result.is_success());
}

#[test]
fn test_sync_result_dry_run() {
    let result = SyncResult::dry_run(vec![make_component("c1")], vec![], vec![], vec![], vec![]);

    assert!(result.dry_run);
    assert_eq!(result.create_count(), 1);
}
