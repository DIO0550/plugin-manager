use super::*;
use crate::component::Scope;
use crate::fs::RealFs;
use crate::target::TargetKind;
use std::fs;
use tempfile::TempDir;

fn sample_manifest(name: &str) -> PluginManifest {
    PluginManifest {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        description: None,
        author: None,
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
    }
}

fn write_tree(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn exclusion_includes_default_component_dirs() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let manifest = sample_manifest("spec-plugin");
    let paths = attached_exclusion_paths(root, &manifest);
    assert!(paths.contains(&root.join("skills")));
    assert!(paths.contains(&root.join("agents")));
    assert!(paths.contains(&root.join("commands")));
    assert!(paths.contains(&root.join("hooks")));
    assert!(paths.contains(&root.join(".claude-plugin")));
}

#[test]
fn exclusion_hooks_file_also_excludes_parent() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let mut manifest = sample_manifest("p");
    manifest.hooks = Some("./hooks/hooks.json".into());
    let paths = attached_exclusion_paths(root, &manifest);
    assert!(paths.contains(&root.join("hooks/hooks.json")));
    assert!(paths.contains(&root.join("hooks")));
}

#[test]
fn list_attached_skips_components_keeps_references() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_tree(root, "skills/foo/SKILL.md", "#\n");
    write_tree(root, "references/tdd-guidelines.md", "tdd\n");
    write_tree(root, ".gitignore", "x\n");
    let manifest = sample_manifest("spec-plugin");
    let entries = list_attached_for_plugin(root, &manifest);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "references");
}

#[test]
fn custom_skills_path_not_attached_default_skills_is() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_tree(root, "my-skills/foo/SKILL.md", "#\n");
    write_tree(root, "skills/readme.md", "orphan\n");
    write_tree(root, "references/a.md", "a\n");
    let mut manifest = sample_manifest("p");
    manifest.skills = Some("./my-skills".into());
    let entries = list_attached_for_plugin(root, &manifest);
    let names: Vec<_> = entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, vec!["references", "skills"]);
}

#[test]
fn deploy_preserves_structure_under_plugins_plugin_name() {
    let tmp = TempDir::new().unwrap();
    let plugin_root = tmp.path().join("plugin");
    let project = tmp.path().join("project");
    fs::create_dir_all(&plugin_root).unwrap();
    fs::create_dir_all(&project).unwrap();
    write_tree(&plugin_root, "skills/foo/SKILL.md", "#\n");
    write_tree(&plugin_root, "references/tdd-guidelines.md", "guidelines\n");
    write_tree(
        &plugin_root,
        "references/test-design-patterns.md",
        "patterns\n",
    );

    let manifest = sample_manifest("spec-plugin");
    let dest = deploy_attached_resources(
        &RealFs,
        &DeployAttachedRequest {
            plugin_root: &plugin_root,
            manifest: &manifest,
            target_kind: TargetKind::Codex,
            scope: Scope::Project,
            project_root: &project,
        },
    )
    .unwrap()
    .expect("dest");

    assert_eq!(dest, project.join(".codex/plugins/spec-plugin"));
    assert_eq!(
        fs::read_to_string(dest.join("references/tdd-guidelines.md")).unwrap(),
        "guidelines\n"
    );
    assert!(dest.join("references/test-design-patterns.md").is_file());
    assert!(!dest.join("skills").exists());
}

#[test]
fn deploy_replace_removes_stale_attached_files() {
    let tmp = TempDir::new().unwrap();
    let plugin_root = tmp.path().join("plugin");
    let project = tmp.path().join("project");
    fs::create_dir_all(&plugin_root).unwrap();
    fs::create_dir_all(&project).unwrap();
    write_tree(&plugin_root, "references/keep.md", "keep\n");

    let manifest = sample_manifest("p");
    let dest = plugin_attached_root(TargetKind::Cursor, Scope::Project, &project, "p").unwrap();
    fs::create_dir_all(dest.join("references")).unwrap();
    fs::write(dest.join("references/stale.md"), "stale\n").unwrap();

    deploy_attached_resources(
        &RealFs,
        &DeployAttachedRequest {
            plugin_root: &plugin_root,
            manifest: &manifest,
            target_kind: TargetKind::Cursor,
            scope: Scope::Project,
            project_root: &project,
        },
    )
    .unwrap();

    assert!(dest.join("references/keep.md").is_file());
    assert!(!dest.join("references/stale.md").exists());
}

#[test]
fn deploy_all_skill_target_kinds() {
    let tmp = TempDir::new().unwrap();
    let plugin_root = tmp.path().join("plugin");
    let project = tmp.path().join("project");
    fs::create_dir_all(&plugin_root).unwrap();
    fs::create_dir_all(&project).unwrap();
    write_tree(&plugin_root, "references/a.md", "a\n");
    let manifest = sample_manifest("demo");

    for kind in [
        TargetKind::Codex,
        TargetKind::Copilot,
        TargetKind::Antigravity,
        TargetKind::GeminiCli,
        TargetKind::Cursor,
    ] {
        let dest = deploy_attached_resources(
            &RealFs,
            &DeployAttachedRequest {
                plugin_root: &plugin_root,
                manifest: &manifest,
                target_kind: kind,
                scope: Scope::Project,
                project_root: &project,
            },
        )
        .unwrap()
        .expect("dest");
        assert!(
            dest.join("references/a.md").is_file(),
            "missing for {:?}",
            kind
        );
    }
}

#[test]
fn remove_attached_deletes_root() {
    let tmp = TempDir::new().unwrap();
    let project = tmp.path();
    let dest =
        plugin_attached_root(TargetKind::Codex, Scope::Project, project, "spec-plugin").unwrap();
    fs::create_dir_all(dest.join("references")).unwrap();
    fs::write(dest.join("references/a.md"), "a\n").unwrap();

    let removed = remove_attached_resources(
        &RealFs,
        TargetKind::Codex,
        Scope::Project,
        project,
        "spec-plugin",
    )
    .unwrap();
    assert_eq!(removed, Some(dest.clone()));
    assert!(!dest.exists());
}
