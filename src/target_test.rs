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
    assert_eq!(targets.len(), 4);
    assert!(targets.iter().any(|t| t.name() == "antigravity"));
    assert!(targets.iter().any(|t| t.name() == "codex"));
    assert!(targets.iter().any(|t| t.name() == "copilot"));
    assert!(targets.iter().any(|t| t.name() == "gemini"));
}

#[test]
fn test_parse_target_gemini() {
    let target = parse_target("gemini").unwrap();
    assert_eq!(target.name(), "gemini");
}

#[test]
fn test_plugin_origin_qualify_returns_name_only() {
    // フラット化後は marketplace/plugin の段が配置物識別に使われないため、
    // qualify は引数 name をそのまま返す。
    let origin = PluginOrigin::from_marketplace("any-mp", "any-plg");
    assert_eq!(origin.qualify("foo"), "foo");
    assert_eq!(origin.qualify("plugin_my-skill"), "plugin_my-skill");
}
