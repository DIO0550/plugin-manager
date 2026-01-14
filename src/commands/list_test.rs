use super::*;

fn create_empty_plugin(name: &str) -> PluginSummary {
    PluginSummary {
        name: name.to_string(),
        marketplace: None,
        version: "1.0.0".to_string(),
        skills: vec![],
        agents: vec![],
        commands: vec![],
        instructions: vec![],
        hooks: vec![],
        enabled: false,
    }
}

fn create_plugin_with_skills(name: &str, skill_count: usize, enabled: bool) -> PluginSummary {
    PluginSummary {
        name: name.to_string(),
        marketplace: Some("github".to_string()),
        version: "1.0.0".to_string(),
        skills: (0..skill_count).map(|i| format!("skill{}", i)).collect(),
        agents: vec![],
        commands: vec![],
        instructions: vec![],
        hooks: vec![],
        enabled,
    }
}

fn create_full_plugin(name: &str, enabled: bool) -> PluginSummary {
    PluginSummary {
        name: name.to_string(),
        marketplace: Some("github".to_string()),
        version: "2.0.0".to_string(),
        skills: vec!["skill1".to_string(), "skill2".to_string()],
        agents: vec!["agent1".to_string()],
        commands: vec!["cmd1".to_string()],
        instructions: vec!["inst1".to_string()],
        hooks: vec!["hook1".to_string()],
        enabled,
    }
}

// ========================================
// filter_by_type tests
// ========================================

#[test]
fn test_filter_by_type_none_passes_all() {
    let empty = create_empty_plugin("empty");
    let with_skills = create_plugin_with_skills("with-skills", 2, true);

    assert!(filter_by_type(&empty, None));
    assert!(filter_by_type(&with_skills, None));
}

#[test]
fn test_filter_by_type_skill_matches() {
    let plugin = create_plugin_with_skills("test", 2, true);
    assert!(filter_by_type(&plugin, Some(&ComponentKind::Skill)));
}

#[test]
fn test_filter_by_type_skill_no_match() {
    let plugin = create_empty_plugin("test");
    assert!(!filter_by_type(&plugin, Some(&ComponentKind::Skill)));
}

#[test]
fn test_filter_by_type_agent_matches() {
    let plugin = create_full_plugin("test", true);
    assert!(filter_by_type(&plugin, Some(&ComponentKind::Agent)));
}

#[test]
fn test_filter_by_type_command_matches() {
    let plugin = create_full_plugin("test", true);
    assert!(filter_by_type(&plugin, Some(&ComponentKind::Command)));
}

#[test]
fn test_filter_by_type_instruction_matches() {
    let plugin = create_full_plugin("test", true);
    assert!(filter_by_type(&plugin, Some(&ComponentKind::Instruction)));
}

#[test]
fn test_filter_by_type_hook_matches() {
    let plugin = create_full_plugin("test", true);
    assert!(filter_by_type(&plugin, Some(&ComponentKind::Hook)));
}

#[test]
fn test_filter_by_type_hook_no_match() {
    let plugin = create_plugin_with_skills("test", 1, true);
    assert!(!filter_by_type(&plugin, Some(&ComponentKind::Hook)));
}

// ========================================
// filter_by_target tests
// ========================================

#[test]
fn test_filter_by_target_none_passes_all() {
    let enabled = create_plugin_with_skills("enabled", 1, true);
    let disabled = create_plugin_with_skills("disabled", 1, false);

    assert!(filter_by_target(&enabled, None));
    assert!(filter_by_target(&disabled, None));
}

#[test]
fn test_filter_by_target_enabled_only() {
    let enabled = create_plugin_with_skills("enabled", 1, true);
    let disabled = create_plugin_with_skills("disabled", 1, false);

    assert!(filter_by_target(&enabled, Some(&TargetKind::Codex)));
    assert!(!filter_by_target(&disabled, Some(&TargetKind::Codex)));

    assert!(filter_by_target(&enabled, Some(&TargetKind::Copilot)));
    assert!(!filter_by_target(&disabled, Some(&TargetKind::Copilot)));
}

// ========================================
// format_components tests
// ========================================

#[test]
fn test_format_components_empty() {
    let plugin = create_empty_plugin("empty");
    assert_eq!(format_components(&plugin), "-");
}

#[test]
fn test_format_components_single() {
    let plugin = create_plugin_with_skills("test", 2, true);
    assert_eq!(format_components(&plugin), "2 skills");
}

#[test]
fn test_format_components_multiple() {
    let plugin = create_full_plugin("test", true);
    // 固定順序: Skill → Agent → Command → Instruction → Hook
    assert_eq!(
        format_components(&plugin),
        "2 skills, 1 agents, 1 commands, 1 instructions, 1 hooks"
    );
}

// ========================================
// filter_plugins tests
// ========================================

#[test]
fn test_filter_plugins_combined() {
    let plugins = vec![
        create_plugin_with_skills("enabled-with-skills", 2, true),
        create_plugin_with_skills("disabled-with-skills", 1, false),
        create_empty_plugin("enabled-empty"),
        create_full_plugin("enabled-full", true),
    ];

    // --type skill --target codex
    let args = Args {
        component_type: Some(ComponentKind::Skill),
        target: Some(TargetKind::Codex),
        json: false,
        simple: false,
        outdated: false,
    };

    let filtered = filter_plugins(plugins, &args);

    // enabled かつ skills を持つもののみ
    assert_eq!(filtered.len(), 2);
    assert_eq!(filtered[0].name, "enabled-with-skills");
    assert_eq!(filtered[1].name, "enabled-full");
}
