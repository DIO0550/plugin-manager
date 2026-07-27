use super::*;
use crate::component::ComponentKind;
use crate::placement_names::COPILOT_COMMAND_SUBDIR;
use std::collections::HashSet;
use std::fs;
use tempfile::TempDir;

fn exclusions_default(_plugin_root: &Path) -> HashSet<PathBuf> {
    [
        ComponentKind::Skill.plural(),
        ComponentKind::Agent.plural(),
        ComponentKind::Command.plural(),
        ComponentKind::Hook.plural(),
        ComponentKind::Instruction.plural(),
        COPILOT_COMMAND_SUBDIR,
    ]
    .into_iter()
    .map(PathBuf::from)
    .collect()
}

#[test]
fn list_includes_references_and_loose_markdown() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("skills/foo")).unwrap();
    fs::write(root.join("skills/foo/SKILL.md"), "# s\n").unwrap();
    fs::create_dir_all(root.join("references")).unwrap();
    fs::write(root.join("references/tdd-guidelines.md"), "tdd\n").unwrap();
    fs::write(root.join("notes.md"), "note\n").unwrap();

    let entries = list_plugin_attached_resources(root, &exclusions_default(root));
    let names: Vec<_> = entries
        .iter()
        .map(|e| e.relative.to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, vec!["notes.md", "references"]);
}

#[test]
fn list_excludes_component_dirs_and_reserved() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    for dir in [
        "skills",
        "agents",
        "commands",
        "hooks",
        "instructions",
        "prompts",
        ".claude-plugin",
        ".git",
        ".github",
    ] {
        fs::create_dir_all(root.join(dir)).unwrap();
    }
    fs::write(root.join("plugin.json"), "{}").unwrap();
    fs::write(root.join(".plm-meta.json"), "{}").unwrap();
    fs::write(root.join(".gitignore"), "").unwrap();
    fs::write(root.join("AGENTS.md"), "").unwrap();
    fs::write(root.join("README.md"), "").unwrap();
    fs::write(root.join("LICENSE"), "").unwrap();
    fs::create_dir_all(root.join("docs")).unwrap();

    let entries = list_plugin_attached_resources(root, &exclusions_default(root));
    let names: Vec<_> = entries
        .iter()
        .map(|e| e.relative.to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, vec!["docs"]);
}

#[test]
fn list_excludes_manifest_resolved_custom_skills_path() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("lib/skills/foo")).unwrap();
    fs::write(root.join("lib/skills/foo/SKILL.md"), "# s\n").unwrap();
    fs::create_dir_all(root.join("references")).unwrap();

    let mut excluded = HashSet::new();
    excluded.insert(PathBuf::from("lib/skills"));

    let entries = list_plugin_attached_resources(root, &excluded);
    let names: Vec<_> = entries
        .iter()
        .map(|e| e.relative.to_string_lossy().into_owned())
        .collect();
    // `lib` is ancestor of excluded `lib/skills` → skipped; references kept
    assert_eq!(names, vec!["references"]);
}

#[test]
fn list_skips_symlinks() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("real-refs")).unwrap();
    fs::write(root.join("real-refs/a.md"), "a\n").unwrap();
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root.join("real-refs"), root.join("references")).unwrap();
    }
    #[cfg(not(unix))]
    {
        return;
    }

    let entries = list_plugin_attached_resources(root, &HashSet::new());
    let names: Vec<_> = entries
        .iter()
        .map(|e| e.relative.to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, vec!["real-refs"]);
}

#[test]
fn is_attached_name_excluded_readme_variants() {
    assert!(is_attached_name_excluded("README"));
    assert!(is_attached_name_excluded("README.md"));
    assert!(is_attached_name_excluded("readme.txt"));
    assert!(is_attached_name_excluded("LICENSE-MIT"));
    assert!(is_attached_name_excluded("CHANGELOG.md"));
    assert!(is_attached_name_excluded("CONTRIBUTING"));
    assert!(!is_attached_name_excluded("references"));
    assert!(!is_attached_name_excluded("READ"));
}
