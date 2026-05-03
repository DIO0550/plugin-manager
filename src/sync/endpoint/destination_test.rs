use super::*;

#[test]
fn test_destination_new() {
    let dest = SyncDestination::new(TargetKind::Copilot, Path::new("."));
    assert!(dest.is_ok());
    assert_eq!(dest.unwrap().name(), "copilot");
}
