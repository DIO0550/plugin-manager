use super::*;
use std::path::PathBuf;

fn scanned(name: &str, is_dir: bool, path: PathBuf) -> ScannedComponent {
    ScannedComponent {
        origin: crate::target::PluginOrigin::Unknown,
        name: name.to_string(),
        path,
        is_dir,
    }
}

#[test]
fn filter_skill_dir_with_skill_md() {
    let dir = tempfile::tempdir().unwrap();
    let skill = dir.path().join("my-skill");
    std::fs::create_dir_all(&skill).unwrap();
    std::fs::write(skill.join("SKILL.md"), "# skill").unwrap();
    let c = scanned("my-skill", true, skill);
    assert_eq!(filter_skill_dir(&c).as_deref(), Some("my-skill"));
}

#[test]
fn filter_skill_dir_without_skill_md() {
    let dir = tempfile::tempdir().unwrap();
    let skill = dir.path().join("empty");
    std::fs::create_dir_all(&skill).unwrap();
    let c = scanned("empty", true, skill);
    assert!(filter_skill_dir(&c).is_none());
}

#[test]
fn filter_skill_dir_rejects_file_entry() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("SKILL.md");
    std::fs::write(&file, "# skill").unwrap();
    let c = scanned("SKILL.md", false, file);
    assert!(filter_skill_dir(&c).is_none());
}

#[test]
fn filter_suffix_file_strips_suffix() {
    let c = scanned(
        "helper.agent.md",
        false,
        PathBuf::from("/tmp/helper.agent.md"),
    );
    assert_eq!(
        filter_suffix_file(&c, ".agent.md").as_deref(),
        Some("helper")
    );
}

#[test]
fn filter_plain_markdown_rejects_agent_suffix() {
    let c = scanned("x.agent.md", false, PathBuf::from("/tmp/x.agent.md"));
    assert!(filter_plain_markdown(&c).is_none());
}

#[test]
fn filter_exact_file_hooks() {
    let c = scanned("hooks.json", false, PathBuf::from("/tmp/hooks.json"));
    assert_eq!(
        filter_exact_file(&c, "hooks.json", "hooks").as_deref(),
        Some("hooks")
    );
}

#[test]
fn filter_suffix_file_preserves_repeated_suffix() {
    for (filename, suffix, expected) in [
        ("x.agent.md.agent.md", ".agent.md", "x.agent.md"),
        ("x.prompt.md.prompt.md", ".prompt.md", "x.prompt.md"),
        ("a.json.json", ".json", "a.json"),
        (".md.md", ".md", ".md"),
    ] {
        let c = scanned(filename, false, PathBuf::from(filename));
        let name = filter_suffix_file(&c, suffix).unwrap();
        assert_eq!(name, expected);
        assert_eq!(format!("{name}{suffix}"), filename);
    }
}

#[test]
fn filter_suffix_file_rejects_empty_names_directories_and_wrong_suffixes() {
    for suffix in [".md", ".agent.md", ".prompt.md", ".json"] {
        for (filename, is_dir) in [(suffix, false), ("x.txt", false)] {
            let c = scanned(filename, is_dir, PathBuf::from(filename));
            assert!(filter_suffix_file(&c, suffix).is_none());
        }
        let filename = format!("x{suffix}");
        let c = scanned(&filename, true, PathBuf::from(&filename));
        assert!(filter_suffix_file(&c, suffix).is_none());
    }
}

#[test]
fn filter_plain_markdown_preserves_repeated_suffix_and_rejects_empty_name() {
    for (filename, expected) in [
        ("guide.md.md", Some("guide.md")),
        (".md.md", Some(".md")),
        (".md", None),
        ("x.prompt.md", None),
    ] {
        let c = scanned(filename, false, PathBuf::from(filename));
        assert_eq!(filter_plain_markdown(&c).as_deref(), expected);
    }
}

#[test]
fn filter_json_suffix_preserves_repeated_suffix_and_rejects_empty_name() {
    let c = scanned("a.json.json", false, PathBuf::from("a.json.json"));
    assert_eq!(filter_json_suffix(&c).as_deref(), Some("a.json"));
    let c = scanned(".json", false, PathBuf::from(".json"));
    assert!(filter_json_suffix(&c).is_none());
}
