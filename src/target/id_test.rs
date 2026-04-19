use super::*;

#[test]
fn test_target_id() {
    let id = TargetId::new("codex");
    assert_eq!(id.as_str(), "codex");
    assert_eq!(format!("{}", id), "codex");

    let from_str: TargetId = "copilot".into();
    assert_eq!(from_str.as_str(), "copilot");
}
