use super::*;
use crate::component::{Component, ComponentKind};
use crate::plugin::{Author, PluginManifest};
use std::path::PathBuf;

fn comp(kind: ComponentKind, name: &str) -> Component {
    Component {
        kind,
        name: name.to_string(),
        path: PathBuf::from(format!("dummy/{}", name)),
    }
}

fn create_empty_summary() -> InstalledPlugin {
    InstalledPlugin::new_for_test("test-plugin", "1.0.0", Vec::new(), None, None, true)
}

fn create_full_summary() -> InstalledPlugin {
    InstalledPlugin::new_for_test(
        "full-plugin",
        "2.0.0",
        vec![
            comp(ComponentKind::Skill, "skill1"),
            comp(ComponentKind::Skill, "skill2"),
            comp(ComponentKind::Agent, "agent1"),
            comp(ComponentKind::Command, "cmd1"),
            comp(ComponentKind::Command, "cmd2"),
            comp(ComponentKind::Command, "cmd3"),
            comp(ComponentKind::Instruction, "inst1"),
            comp(ComponentKind::Hook, "hook1"),
            comp(ComponentKind::Hook, "hook2"),
        ],
        None,
        Some("awesome-marketplace".to_string()),
        true,
    )
}

// ========================================
// id tests
// ========================================

#[test]
fn test_installed_plugin_id_returns_some_value() {
    let summary = InstalledPlugin::new_for_test(
        "test-plugin",
        "1.0.0",
        Vec::new(),
        Some("owner--repo".to_string()),
        None,
        true,
    );
    assert_eq!(summary.id(), "owner--repo");
}

#[test]
fn test_installed_plugin_id_falls_back_to_name() {
    let summary = create_empty_summary();
    assert_eq!(summary.id(), "test-plugin");
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

    let skill_count = counts.iter().find(|c| c.0 == ComponentKind::Skill).unwrap();
    assert_eq!(skill_count.1, 2);

    let agent_count = counts.iter().find(|c| c.0 == ComponentKind::Agent).unwrap();
    assert_eq!(agent_count.1, 1);

    let cmd_count = counts
        .iter()
        .find(|c| c.0 == ComponentKind::Command)
        .unwrap();
    assert_eq!(cmd_count.1, 3);

    let inst_count = counts
        .iter()
        .find(|c| c.0 == ComponentKind::Instruction)
        .unwrap();
    assert_eq!(inst_count.1, 1);

    let hook_count = counts.iter().find(|c| c.0 == ComponentKind::Hook).unwrap();
    assert_eq!(hook_count.1, 2);
}

#[test]
fn test_component_type_counts_partial() {
    let summary = InstalledPlugin::new_for_test(
        "partial",
        "1.0.0",
        vec![
            comp(ComponentKind::Agent, "a1"),
            comp(ComponentKind::Agent, "a2"),
            comp(ComponentKind::Hook, "h1"),
        ],
        None,
        None,
        true,
    );

    let counts = summary.component_type_counts();

    assert_eq!(counts.len(), 2);
    assert!(counts
        .iter()
        .any(|c| c.0 == ComponentKind::Agent && c.1 == 2));
    assert!(counts
        .iter()
        .any(|c| c.0 == ComponentKind::Hook && c.1 == 1));
}

#[test]
fn test_component_type_counts_order() {
    let summary = create_full_summary();
    let counts = summary.component_type_counts();

    assert_eq!(counts[0].0, ComponentKind::Skill);
    assert_eq!(counts[1].0, ComponentKind::Agent);
    assert_eq!(counts[2].0, ComponentKind::Command);
    assert_eq!(counts[3].0, ComponentKind::Instruction);
    assert_eq!(counts[4].0, ComponentKind::Hook);
}

// ========================================
// component_names tests
// ========================================

#[test]
fn test_component_names_skills() {
    let summary = create_full_summary();
    let names = summary.component_names(ComponentKind::Skill);

    assert_eq!(names.len(), 2);
    assert_eq!(names[0], "skill1");
    assert_eq!(names[1], "skill2");
}

#[test]
fn test_component_names_agents() {
    let summary = create_full_summary();
    let names = summary.component_names(ComponentKind::Agent);

    assert_eq!(names.len(), 1);
    assert_eq!(names[0], "agent1");
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
    assert_eq!(names[0], "inst1");
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

// ========================================
// InstalledPlugin::author() tests
// ========================================

fn installed_plugin_with_author(author: Option<Author>) -> InstalledPlugin {
    let manifest = PluginManifest {
        name: "author-test".to_string(),
        version: "1.0.0".to_string(),
        description: None,
        author,
        homepage: None,
        repository: None,
        license: None,
        keywords: None,
        commands: None,
        agents: None,
        skills: None,
        instructions: None,
        hooks: None,
        mcp_servers: None,
        lsp_servers: None,
        installed_at: None,
    };
    InstalledPlugin::new_for_test_full(
        manifest,
        PathBuf::from("/test"),
        Vec::new(),
        None,
        None,
        false,
    )
}

#[test]
fn author_returns_some_when_name_is_non_empty() {
    let summary = installed_plugin_with_author(Some(Author {
        name: "alice".to_string(),
        email: None,
        url: None,
    }));
    let author = summary.author();
    assert!(author.is_some());
    assert_eq!(author.unwrap().name, "alice");
}

#[test]
fn author_returns_none_when_name_is_empty() {
    let summary = installed_plugin_with_author(Some(Author {
        name: String::new(),
        email: None,
        url: None,
    }));
    assert!(summary.author().is_none());
}
