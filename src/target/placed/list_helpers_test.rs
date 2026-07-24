use super::*;
use crate::target::placed::filter::filter_skill_dir;

#[test]
fn scan_and_filter_finds_skill() {
    let dir = tempfile::tempdir().unwrap();
    let skill = dir.path().join("skills").join("plugin_skill");
    std::fs::create_dir_all(&skill).unwrap();
    std::fs::write(skill.join("SKILL.md"), "# skill").unwrap();

    let names = scan_and_filter(dir.path(), "skills", filter_skill_dir).unwrap();
    assert_eq!(names, vec!["plugin_skill".to_string()]);
}

#[test]
fn scan_and_filter_empty_dir() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("skills")).unwrap();
    let names = scan_and_filter(dir.path(), "skills", filter_skill_dir).unwrap();
    assert!(names.is_empty());
}

#[test]
fn scan_and_filter_missing_subdir() {
    let dir = tempfile::tempdir().unwrap();
    let names = scan_and_filter(dir.path(), "skills", filter_skill_dir).unwrap();
    assert!(names.is_empty());
}

#[test]
fn list_instruction_at_exists() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("AGENTS.md");
    std::fs::write(&path, "# agents").unwrap();
    assert_eq!(
        list_instruction_at(&path, "AGENTS.md"),
        vec!["AGENTS.md".to_string()]
    );
}

#[test]
fn list_instruction_at_missing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("AGENTS.md");
    assert!(list_instruction_at(&path, "AGENTS.md").is_empty());
}
