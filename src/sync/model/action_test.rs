use super::*;

#[test]
fn test_sync_action_display() {
    assert_eq!(SyncAction::Create.display_name(), "Create");
    assert_eq!(SyncAction::Update.display_name(), "Update");
    assert_eq!(SyncAction::Delete.display_name(), "Delete");
}
