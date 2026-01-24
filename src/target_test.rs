use super::*;

#[test]
fn test_parse_target_codex() {
    let target = parse_target("codex").unwrap();
    assert_eq!(target.name(), "codex");
}

#[test]
fn test_parse_target_copilot() {
    let target = parse_target("copilot").unwrap();
    assert_eq!(target.name(), "copilot");
}

#[test]
fn test_parse_target_unknown() {
    let result = parse_target("unknown");
    assert!(result.is_err());
}

#[test]
fn test_parse_target_antigravity() {
    let target = parse_target("antigravity").unwrap();
    assert_eq!(target.name(), "antigravity");
}

#[test]
fn test_all_targets() {
    let targets = all_targets();
    assert_eq!(targets.len(), 3);
    assert!(targets.iter().any(|t| t.name() == "antigravity"));
    assert!(targets.iter().any(|t| t.name() == "codex"));
    assert!(targets.iter().any(|t| t.name() == "copilot"));
}
