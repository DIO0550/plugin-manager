use super::*;
use crate::plugin::PluginManifest;
use std::fs;
use tempfile::TempDir;

fn bare_manifest(skills: Option<&str>) -> PluginManifest {
    PluginManifest {
        name: "spec-plugin".into(),
        version: "1.0.0".into(),
        description: None,
        author: None,
        homepage: None,
        repository: None,
        license: None,
        keywords: None,
        commands: None,
        agents: None,
        skills: skills.map(str::to_string),
        instructions: None,
        hooks: None,
        mcp_servers: None,
        lsp_servers: None,
    }
}

#[test]
fn list_attached_for_default_layout_finds_references() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("skills/implementation-plan")).unwrap();
    fs::write(root.join("skills/implementation-plan/SKILL.md"), "# p\n").unwrap();
    fs::create_dir_all(root.join("agents")).unwrap();
    fs::create_dir_all(root.join("hooks")).unwrap();
    fs::create_dir_all(root.join(".claude-plugin")).unwrap();
    fs::write(root.join(".claude-plugin/plugin.json"), "{}").unwrap();
    fs::create_dir_all(root.join("references")).unwrap();
    fs::write(root.join("references/tdd-guidelines.md"), "tdd\n").unwrap();
    fs::write(
        root.join("references/test-design-patterns.md"),
        "patterns\n",
    )
    .unwrap();

    let entries = list_attached_for_plugin(&bare_manifest(None), root);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].relative, PathBuf::from("references"));
}

#[test]
fn attached_exclusion_paths_includes_custom_skills() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    let manifest = bare_manifest(Some("packages/skills"));
    let set = attached_exclusion_paths(&manifest, root);
    assert!(set.contains(&PathBuf::from("packages/skills")));
}
