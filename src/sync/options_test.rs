use super::*;

#[test]
fn test_syncable_kind_all() {
    let all = SyncableKind::all();
    assert_eq!(all.len(), 4);
    assert!(all.contains(&SyncableKind::Skill));
    assert!(all.contains(&SyncableKind::Agent));
    assert!(all.contains(&SyncableKind::Command));
    assert!(all.contains(&SyncableKind::Instruction));
}

#[test]
fn test_sync_options_builder() {
    let opts = SyncOptions::new()
        .with_component_type(SyncableKind::Skill)
        .with_scope(Scope::Personal)
        .with_dry_run(true);

    assert_eq!(opts.component_type, Some(SyncableKind::Skill));
    assert_eq!(opts.scope, Some(Scope::Personal));
    assert!(opts.dry_run);
}
