use super::*;
use std::collections::HashSet;
use std::fs;
use tempfile::TempDir;

fn write_tree(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn lists_references_and_skips_skills() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_tree(root, "skills/foo/SKILL.md", "# skill\n");
    write_tree(root, "references/tdd.md", "tdd\n");
    write_tree(root, "docs/guide.md", "guide\n");

    let mut excluded_paths = HashSet::new();
    excluded_paths.insert(root.join("skills"));
    let excluded_names = HashSet::new();

    let entries = list_plugin_attached_resources(root, &excluded_paths, &excluded_names);
    let names: Vec<_> = entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, vec!["docs", "references"]);
    assert!(entries.iter().any(|e| e.absolute.join("tdd.md").is_file()));
}

#[test]
fn excludes_hooks_parent_when_hooks_file_declared() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_tree(root, "hooks/hooks.json", "{}\n");
    write_tree(root, "hooks/extra.sh", "echo\n");
    write_tree(root, "references/a.md", "a\n");

    let mut excluded_paths = HashSet::new();
    excluded_paths.insert(root.join("hooks/hooks.json"));
    let excluded_names = HashSet::new();

    let entries = list_plugin_attached_resources(root, &excluded_paths, &excluded_names);
    let names: Vec<_> = entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, vec!["references"]);
}

#[test]
fn excludes_vcs_names() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join(".git")).unwrap();
    write_tree(root, ".gitignore", "target\n");
    write_tree(root, "references/a.md", "a\n");

    let excluded_paths = HashSet::new();
    let excluded_names: HashSet<&str> = [".git", ".gitignore"].into_iter().collect();

    let entries = list_plugin_attached_resources(root, &excluded_paths, &excluded_names);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "references");
}

#[test]
fn custom_skills_path_excluded_default_skills_can_be_attached() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_tree(root, "my-skills/foo/SKILL.md", "#\n");
    write_tree(root, "skills/orphan.md", "orphan\n");
    write_tree(root, "references/a.md", "a\n");

    let mut excluded_paths = HashSet::new();
    excluded_paths.insert(root.join("my-skills"));
    let excluded_names = HashSet::new();

    let entries = list_plugin_attached_resources(root, &excluded_paths, &excluded_names);
    let names: Vec<_> = entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, vec!["references", "skills"]);
}

#[test]
fn returns_empty_when_root_missing() {
    let tmp = TempDir::new().unwrap();
    let missing = tmp.path().join("nope");
    let entries = list_plugin_attached_resources(&missing, &HashSet::new(), &HashSet::new());
    assert!(entries.is_empty());
}
