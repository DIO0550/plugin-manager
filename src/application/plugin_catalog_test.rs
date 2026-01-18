use super::*;

fn create_empty_summary() -> PluginSummary {
    PluginSummary {
        name: "test-plugin".to_string(),
        marketplace: None,
        version: "1.0.0".to_string(),
        skills: vec![],
        agents: vec![],
        commands: vec![],
        instructions: vec![],
        hooks: vec![],
        enabled: true,
    }
}

fn create_full_summary() -> PluginSummary {
    PluginSummary {
        name: "full-plugin".to_string(),
        marketplace: Some("awesome-marketplace".to_string()),
        version: "2.0.0".to_string(),
        skills: vec!["skill1".to_string(), "skill2".to_string()],
        agents: vec!["agent1".to_string()],
        commands: vec!["cmd1".to_string(), "cmd2".to_string(), "cmd3".to_string()],
        instructions: vec!["inst1".to_string()],
        hooks: vec!["hook1".to_string(), "hook2".to_string()],
        enabled: true,
    }
}

// ========================================
// component_count tests
// ========================================

#[test]
fn test_component_count_empty() {
    let summary = create_empty_summary();
    assert_eq!(summary.component_count(), 0);
}

#[test]
fn test_component_count_full() {
    let summary = create_full_summary();
    // 2 skills + 1 agent + 3 commands + 1 instruction + 2 hooks = 9
    assert_eq!(summary.component_count(), 9);
}

#[test]
fn test_component_count_partial() {
    let summary = PluginSummary {
        name: "partial".to_string(),
        marketplace: None,
        version: "1.0.0".to_string(),
        skills: vec!["s1".to_string()],
        agents: vec![],
        commands: vec!["c1".to_string(), "c2".to_string()],
        instructions: vec![],
        hooks: vec![],
        enabled: true,
    };
    assert_eq!(summary.component_count(), 3);
}

// ========================================
// has_components tests
// ========================================

#[test]
fn test_has_components_empty() {
    let summary = create_empty_summary();
    assert!(!summary.has_components());
}

#[test]
fn test_has_components_with_skills() {
    let mut summary = create_empty_summary();
    summary.skills = vec!["skill".to_string()];
    assert!(summary.has_components());
}

#[test]
fn test_has_components_with_hooks_only() {
    let mut summary = create_empty_summary();
    summary.hooks = vec!["hook".to_string()];
    assert!(summary.has_components());
}

// ========================================
// component_type_counts tests
// ========================================

#[test]
fn test_component_type_counts_empty() {
    let summary = create_empty_summary();
    let counts = summary.component_type_counts();
    assert!(counts.is_empty());
}

#[test]
fn test_component_type_counts_full() {
    let summary = create_full_summary();
    let counts = summary.component_type_counts();

    assert_eq!(counts.len(), 5);

    // Check each type
    let skill_count = counts
        .iter()
        .find(|c| c.kind == ComponentKind::Skill)
        .unwrap();
    assert_eq!(skill_count.count, 2);

    let agent_count = counts
        .iter()
        .find(|c| c.kind == ComponentKind::Agent)
        .unwrap();
    assert_eq!(agent_count.count, 1);

    let cmd_count = counts
        .iter()
        .find(|c| c.kind == ComponentKind::Command)
        .unwrap();
    assert_eq!(cmd_count.count, 3);

    let inst_count = counts
        .iter()
        .find(|c| c.kind == ComponentKind::Instruction)
        .unwrap();
    assert_eq!(inst_count.count, 1);

    let hook_count = counts
        .iter()
        .find(|c| c.kind == ComponentKind::Hook)
        .unwrap();
    assert_eq!(hook_count.count, 2);
}

#[test]
fn test_component_type_counts_partial() {
    let summary = PluginSummary {
        name: "partial".to_string(),
        marketplace: None,
        version: "1.0.0".to_string(),
        skills: vec![],
        agents: vec!["a1".to_string(), "a2".to_string()],
        commands: vec![],
        instructions: vec![],
        hooks: vec!["h1".to_string()],
        enabled: true,
    };

    let counts = summary.component_type_counts();

    assert_eq!(counts.len(), 2);
    assert!(counts
        .iter()
        .any(|c| c.kind == ComponentKind::Agent && c.count == 2));
    assert!(counts
        .iter()
        .any(|c| c.kind == ComponentKind::Hook && c.count == 1));
}

#[test]
fn test_component_type_counts_order() {
    let summary = create_full_summary();
    let counts = summary.component_type_counts();

    // Should be in order: Skill, Agent, Command, Instruction, Hook
    assert_eq!(counts[0].kind, ComponentKind::Skill);
    assert_eq!(counts[1].kind, ComponentKind::Agent);
    assert_eq!(counts[2].kind, ComponentKind::Command);
    assert_eq!(counts[3].kind, ComponentKind::Instruction);
    assert_eq!(counts[4].kind, ComponentKind::Hook);
}

// ========================================
// component_names tests
// ========================================

#[test]
fn test_component_names_skills() {
    let summary = create_full_summary();
    let names = summary.component_names(ComponentKind::Skill);

    assert_eq!(names.len(), 2);
    assert_eq!(names[0].name, "skill1");
    assert_eq!(names[1].name, "skill2");
}

#[test]
fn test_component_names_agents() {
    let summary = create_full_summary();
    let names = summary.component_names(ComponentKind::Agent);

    assert_eq!(names.len(), 1);
    assert_eq!(names[0].name, "agent1");
}

#[test]
fn test_component_names_commands() {
    let summary = create_full_summary();
    let names = summary.component_names(ComponentKind::Command);

    assert_eq!(names.len(), 3);
}

#[test]
fn test_component_names_instructions() {
    let summary = create_full_summary();
    let names = summary.component_names(ComponentKind::Instruction);

    assert_eq!(names.len(), 1);
    assert_eq!(names[0].name, "inst1");
}

#[test]
fn test_component_names_hooks() {
    let summary = create_full_summary();
    let names = summary.component_names(ComponentKind::Hook);

    assert_eq!(names.len(), 2);
}

#[test]
fn test_component_names_empty() {
    let summary = create_empty_summary();

    for kind in ComponentKind::all() {
        let names = summary.component_names(*kind);
        assert!(names.is_empty(), "{:?} should be empty", kind);
    }
}
