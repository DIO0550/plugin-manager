use super::*;
use std::path::Path;

#[test]
fn skill_dir_joins_skills_and_name() {
    let loc = skill_dir(Path::new("/base"), "my-skill");
    assert_eq!(loc.as_path(), Path::new("/base/skills/my-skill"));
    assert!(loc.is_dir());
}

#[test]
fn agent_file_uses_agent_suffix() {
    let loc = agent_file(Path::new("/base"), "helper");
    assert_eq!(loc.as_path(), Path::new("/base/agents/helper.agent.md"));
    assert!(!loc.is_dir());
}

#[test]
fn instruction_file_project_uses_root() {
    let loc = instruction_file(
        Scope::Project,
        Path::new("/proj"),
        Path::new("/base"),
        "AGENTS.md",
    );
    assert_eq!(loc.as_path(), Path::new("/proj/AGENTS.md"));
}

#[test]
fn instruction_file_personal_uses_base() {
    let loc = instruction_file(
        Scope::Personal,
        Path::new("/proj"),
        Path::new("/base"),
        "AGENTS.md",
    );
    assert_eq!(loc.as_path(), Path::new("/base/AGENTS.md"));
}

#[test]
fn named_file_custom_suffix() {
    let loc = named_file(Path::new("/base"), "commands", "run", ".md");
    assert_eq!(loc.as_path(), Path::new("/base/commands/run.md"));
}
