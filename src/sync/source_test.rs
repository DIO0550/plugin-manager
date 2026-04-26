use super::*;

#[test]
fn test_parse_component_name_single_segment() {
    let (origin, name) = parse_component_name("test-plugin_my-skill").unwrap();
    // フラット化後は origin は復元できないためプレースホルダ
    assert_eq!(origin.marketplace, "_");
    assert_eq!(origin.plugin, "_");
    assert_eq!(name, "test-plugin_my-skill");
}

#[test]
fn test_parse_component_name_instruction_agents() {
    let (origin, name) = parse_component_name("AGENTS.md").unwrap();
    assert_eq!(origin.marketplace, "");
    assert_eq!(origin.plugin, "");
    assert_eq!(name, "AGENTS.md");
}

#[test]
fn test_parse_component_name_instruction_gemini() {
    let (origin, name) = parse_component_name("GEMINI.md").unwrap();
    assert_eq!(origin.marketplace, "");
    assert_eq!(origin.plugin, "");
    assert_eq!(name, "GEMINI.md");
}

#[test]
fn test_parse_component_name_instruction_copilot() {
    let (origin, name) = parse_component_name("copilot-instructions.md").unwrap();
    assert_eq!(origin.marketplace, "");
    assert_eq!(origin.plugin, "");
    assert_eq!(name, "copilot-instructions.md");
}

#[test]
fn test_parse_component_name_rejects_slashes() {
    // 旧 3 セグメント形式は破壊的変更で拒否される
    let result = parse_component_name("github/my-plugin/my-skill");
    assert!(result.is_err());

    let result = parse_component_name("only/two");
    assert!(result.is_err());

    let result = parse_component_name("/leading-slash");
    assert!(result.is_err());
}
