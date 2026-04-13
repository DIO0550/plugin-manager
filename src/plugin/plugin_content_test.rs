//! Plugin のユニットテスト

use super::Plugin;
use crate::component::{Component, ComponentKind};
use crate::plugin::PluginManifest;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn make_manifest(name: &str) -> PluginManifest {
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
        installed_at: None,
    }
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn test_plugin_new_with_skills() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("skills/my-skill/SKILL.md"), "# Skill");

    let plugin = Plugin::new(make_manifest("test"), path.clone());

    let components = plugin.components();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].kind, ComponentKind::Skill);
    assert_eq!(components[0].name, "my-skill");
    assert_eq!(components[0].path, path.join("skills").join("my-skill"));
}

#[test]
fn test_plugin_new_empty_dir() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();

    let plugin = Plugin::new(make_manifest("test"), path);

    assert!(plugin.components().is_empty());
}

#[test]
fn test_plugin_components_returns_slice() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("skills/my-skill/SKILL.md"), "# Skill");

    let plugin = Plugin::new(make_manifest("test"), path);

    let components: &[Component] = plugin.components();
    assert_eq!(components.len(), 1);
}

#[test]
fn test_plugin_new_with_agents() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("agents/my-agent.agent.md"), "# Agent");

    let plugin = Plugin::new(make_manifest("test"), path.clone());

    let components = plugin.components();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].kind, ComponentKind::Agent);
    assert_eq!(components[0].name, "my-agent");
    assert_eq!(
        components[0].path,
        path.join("agents").join("my-agent.agent.md")
    );
}

#[test]
fn test_plugin_new_with_agent_single_file() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("custom-agent.md"), "# Agent");

    let mut manifest = make_manifest("test");
    manifest.agents = Some("custom-agent.md".to_string());

    let plugin = Plugin::new(manifest, path.clone());

    let components = plugin.components();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].kind, ComponentKind::Agent);
    assert_eq!(components[0].name, "custom-agent");
    assert_eq!(components[0].path, path.join("custom-agent.md"));
}

#[test]
fn test_plugin_new_with_commands() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("commands/my-command.prompt.md"), "# Command");

    let plugin = Plugin::new(make_manifest("test"), path.clone());

    let components = plugin.components();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].kind, ComponentKind::Command);
    assert_eq!(components[0].name, "my-command");
    assert_eq!(
        components[0].path,
        path.join("commands").join("my-command.prompt.md")
    );
}

#[test]
fn test_plugin_new_with_command_md_fallback() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("commands/legacy-cmd.md"), "# Command");

    let plugin = Plugin::new(make_manifest("test"), path.clone());

    let components = plugin.components();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].kind, ComponentKind::Command);
    assert_eq!(components[0].name, "legacy-cmd");
    assert_eq!(
        components[0].path,
        path.join("commands").join("legacy-cmd.md")
    );
}

#[test]
fn test_plugin_new_with_instructions() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("docs/usage.md"), "# Usage");

    let mut manifest = make_manifest("test");
    manifest.instructions = Some("docs".to_string());

    let plugin = Plugin::new(manifest, path.clone());

    let components = plugin.components();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].kind, ComponentKind::Instruction);
    assert_eq!(components[0].name, "usage");
    assert_eq!(components[0].path, path.join("docs").join("usage.md"));
}

#[test]
fn test_plugin_new_with_instructions_dir_containing_agents_md() {
    // manifest.instructions が "docs" ディレクトリを指し、その中に AGENTS.md がある場合、
    // resolve_instruction_path はルートの AGENTS.md ではなく docs/AGENTS.md を返すべき。
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("docs/AGENTS.md"), "# Agents in docs");

    let mut manifest = make_manifest("test");
    manifest.instructions = Some("docs".to_string());

    let plugin = Plugin::new(manifest, path.clone());

    let components = plugin.components();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].kind, ComponentKind::Instruction);
    assert_eq!(components[0].name, "AGENTS");
    assert_eq!(components[0].path, path.join("docs").join("AGENTS.md"));
}

#[test]
fn test_plugin_new_with_default_agents_md_instruction() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("AGENTS.md"), "# Agents");

    let plugin = Plugin::new(make_manifest("test"), path.clone());

    let components = plugin.components();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].kind, ComponentKind::Instruction);
    assert_eq!(components[0].name, "AGENTS");
    assert_eq!(components[0].path, path.join("AGENTS.md"));
}

#[test]
fn test_plugin_new_with_hooks() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("hooks/on-apply-patch.sh"), "#!/bin/sh\n");

    let plugin = Plugin::new(make_manifest("test"), path.clone());

    let components = plugin.components();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].kind, ComponentKind::Hook);
    assert_eq!(components[0].name, "on-apply-patch");
    assert_eq!(
        components[0].path,
        path.join("hooks").join("on-apply-patch.sh")
    );
}

#[test]
fn test_plugin_new_with_instruction_file_manifest() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("SETUP.md"), "# Setup guide");

    let mut manifest = make_manifest("test");
    manifest.instructions = Some("SETUP.md".to_string());

    let plugin = Plugin::new(manifest, path.clone());

    let components = plugin.components();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].kind, ComponentKind::Instruction);
    assert_eq!(components[0].name, "SETUP");
    assert_eq!(components[0].path, path.join("SETUP.md"));
}

#[test]
fn test_plugin_new_with_missing_instruction_path() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();

    let mut manifest = make_manifest("test");
    manifest.instructions = Some("nonexistent".to_string());

    let plugin = Plugin::new(manifest, path);

    let instructions: Vec<&Component> = plugin
        .components()
        .iter()
        .filter(|c| c.kind == ComponentKind::Instruction)
        .collect();
    assert!(instructions.is_empty());
}

#[test]
fn test_plugin_new_with_hooks_same_stem_different_ext() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("hooks/pre-commit.sh"), "#!/bin/sh\n");
    write_file(&path.join("hooks/pre-commit.py"), "#!/usr/bin/env python\n");

    let plugin = Plugin::new(make_manifest("test"), path.clone());

    let hooks: Vec<&Component> = plugin
        .components()
        .iter()
        .filter(|c| c.kind == ComponentKind::Hook)
        .collect();
    // Hooks with the same stem but different extensions should be kept as
    // separate components, each preserving its own file path.
    assert_eq!(hooks.len(), 2);
    assert!(hooks.iter().all(|h| h.name == "pre-commit"));
    assert!(hooks.iter().all(|h| {
        h.path == path.join("hooks/pre-commit.sh") || h.path == path.join("hooks/pre-commit.py")
    }));
}

#[test]
fn test_plugin_clone_preserves_components() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("skills/my-skill/SKILL.md"), "# Skill");

    let plugin = Plugin::new(make_manifest("test"), path);
    let cloned = plugin.clone();

    assert_eq!(cloned.components().len(), 1);
    assert_eq!(cloned.components()[0].name, "my-skill");
}
