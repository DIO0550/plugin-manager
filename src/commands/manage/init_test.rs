//! plm init コマンドのテスト

use super::*;
use std::fs;
use tempfile::TempDir;

fn run_init(
    name: &str,
    component_type: ComponentType,
    cwd: &std::path::Path,
) -> Result<(), String> {
    init_in_dir(
        &Args {
            name: name.to_string(),
            component_type,
        },
        cwd,
    )
    .map(|_| ())
}

#[test]
fn init_skill_creates_skill_md() {
    let tmp = TempDir::new().unwrap();
    run_init("my-skill", ComponentType::Skill, tmp.path()).unwrap();

    let skill_md = tmp.path().join("my-skill").join("SKILL.md");
    assert!(skill_md.is_file());
    let content = fs::read_to_string(&skill_md).unwrap();
    assert!(content.contains("name: my-skill"));
    assert!(content.contains("description:"));
    assert!(content.contains("# my-skill"));
}

#[test]
fn init_agent_creates_agent_md() {
    let tmp = TempDir::new().unwrap();
    run_init("my-agent", ComponentType::Agent, tmp.path()).unwrap();

    let path = tmp.path().join("my-agent.agent.md");
    assert!(path.is_file());
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("name: my-agent"));
    assert!(content.contains("tools:"));
    assert!(content.contains("# my-agent"));
}

#[test]
fn init_command_creates_prompt_md() {
    let tmp = TempDir::new().unwrap();
    run_init("my-command", ComponentType::Command, tmp.path()).unwrap();

    let path = tmp.path().join("my-command.prompt.md");
    assert!(path.is_file());
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("name: my-command"));
    assert!(content.contains("description:"));
    assert!(content.contains("# my-command"));
}

#[test]
fn init_skill_fails_when_directory_exists() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("my-skill")).unwrap();

    let err = run_init("my-skill", ComponentType::Skill, tmp.path()).unwrap_err();
    assert!(
        err.contains("already exists") || err.contains("exists"),
        "unexpected error: {err}"
    );
}

#[test]
fn init_agent_fails_when_file_exists() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("my-agent.agent.md"), "existing").unwrap();

    let err = run_init("my-agent", ComponentType::Agent, tmp.path()).unwrap_err();
    assert!(
        err.contains("already exists") || err.contains("exists"),
        "unexpected error: {err}"
    );
}

#[test]
fn init_rejects_empty_name() {
    let tmp = TempDir::new().unwrap();
    let err = run_init("", ComponentType::Skill, tmp.path()).unwrap_err();
    assert!(
        err.contains("name") || err.contains("empty") || err.contains("invalid"),
        "unexpected error: {err}"
    );
}

#[test]
fn init_rejects_path_separators_in_name() {
    let tmp = TempDir::new().unwrap();
    let err = run_init("foo/bar", ComponentType::Skill, tmp.path()).unwrap_err();
    assert!(
        err.contains("invalid") || err.contains("name"),
        "unexpected error: {err}"
    );
}
