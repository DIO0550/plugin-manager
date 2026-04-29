//! Plugin のユニットテスト

use super::{flatten_name, Plugin};
use crate::component::{Component, ComponentKind};
use crate::error::PlmError;
use crate::plugin::PluginManifest;
use crate::target::PluginOrigin;
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

    let plugin = Plugin::new(
        make_manifest("test"),
        path.clone(),
        PluginOrigin::from_marketplace("test", "test"),
    )
    .unwrap();

    let components = plugin.components();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].kind, ComponentKind::Skill);
    assert_eq!(components[0].name, "test_my-skill");
    assert_eq!(components[0].path, path.join("skills").join("my-skill"));
}

#[test]
fn test_plugin_new_empty_dir() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();

    let plugin = Plugin::new(
        make_manifest("test"),
        path,
        PluginOrigin::from_marketplace("test", "test"),
    )
    .unwrap();

    assert!(plugin.components().is_empty());
}

#[test]
fn test_plugin_components_returns_slice() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("skills/my-skill/SKILL.md"), "# Skill");

    let plugin = Plugin::new(
        make_manifest("test"),
        path,
        PluginOrigin::from_marketplace("test", "test"),
    )
    .unwrap();

    let components: &[Component] = plugin.components();
    assert_eq!(components.len(), 1);
}

#[test]
fn test_plugin_new_with_agents() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("agents/my-agent.agent.md"), "# Agent");

    let plugin = Plugin::new(
        make_manifest("test"),
        path.clone(),
        PluginOrigin::from_marketplace("test", "test"),
    )
    .unwrap();

    let components = plugin.components();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].kind, ComponentKind::Agent);
    assert_eq!(components[0].name, "test_my-agent");
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

    let plugin = Plugin::new(
        manifest,
        path.clone(),
        PluginOrigin::from_marketplace("test", "test"),
    )
    .unwrap();

    let components = plugin.components();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].kind, ComponentKind::Agent);
    assert_eq!(components[0].name, "test_custom-agent");
    assert_eq!(components[0].path, path.join("custom-agent.md"));
}

#[test]
fn test_plugin_new_with_commands() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("commands/my-command.prompt.md"), "# Command");

    let plugin = Plugin::new(
        make_manifest("test"),
        path.clone(),
        PluginOrigin::from_marketplace("test", "test"),
    )
    .unwrap();

    let components = plugin.components();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].kind, ComponentKind::Command);
    assert_eq!(components[0].name, "test_my-command");
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

    let plugin = Plugin::new(
        make_manifest("test"),
        path.clone(),
        PluginOrigin::from_marketplace("test", "test"),
    )
    .unwrap();

    let components = plugin.components();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].kind, ComponentKind::Command);
    assert_eq!(components[0].name, "test_legacy-cmd");
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

    let plugin = Plugin::new(
        manifest,
        path.clone(),
        PluginOrigin::from_marketplace("test", "test"),
    )
    .unwrap();

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

    let plugin = Plugin::new(
        manifest,
        path.clone(),
        PluginOrigin::from_marketplace("test", "test"),
    )
    .unwrap();

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

    let plugin = Plugin::new(
        make_manifest("test"),
        path.clone(),
        PluginOrigin::from_marketplace("test", "test"),
    )
    .unwrap();

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

    let plugin = Plugin::new(
        make_manifest("test"),
        path.clone(),
        PluginOrigin::from_marketplace("test", "test"),
    )
    .unwrap();

    let components = plugin.components();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].kind, ComponentKind::Hook);
    // flatten_name 適用後: "{plugin}_{original}" = "test_on-apply-patch"
    assert_eq!(components[0].name, "test_on-apply-patch");
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

    let plugin = Plugin::new(
        manifest,
        path.clone(),
        PluginOrigin::from_marketplace("test", "test"),
    )
    .unwrap();

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

    let plugin = Plugin::new(
        manifest,
        path,
        PluginOrigin::from_marketplace("test", "test"),
    )
    .unwrap();

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

    let result = Plugin::new(
        make_manifest("test"),
        path,
        PluginOrigin::from_marketplace("test", "test"),
    );

    // 同一プラグイン内の同 stem・別拡張子 Hook は detect_name_collisions が
    // Validation エラーで報告する（Skill / Agent / Command と挙動を揃える）。
    let err = result.unwrap_err();
    let msg = match err {
        PlmError::Validation(s) => s,
        other => panic!("expected Validation error, got: {:?}", other),
    };
    assert!(msg.contains("test_pre-commit"), "msg: {msg}");
    assert!(msg.contains("Hook"), "msg: {msg}");
    assert!(msg.contains("conflicts with"), "msg: {msg}");
}

#[test]
fn test_plugin_clone_preserves_components() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("skills/my-skill/SKILL.md"), "# Skill");

    let plugin = Plugin::new(
        make_manifest("test"),
        path,
        PluginOrigin::from_marketplace("test", "test"),
    )
    .unwrap();
    let cloned = plugin.clone();

    assert_eq!(cloned.components().len(), 1);
    assert_eq!(cloned.components()[0].name, "test_my-skill");
}

// =========================================================================
// flatten_name 純粋関数
// =========================================================================

#[test]
fn test_flatten_name_basic() {
    assert_eq!(flatten_name("myplugin", "foo"), "myplugin_foo");
}

#[test]
fn test_flatten_name_original_with_underscore() {
    assert_eq!(flatten_name("myplugin", "foo_bar"), "myplugin_foo_bar");
}

#[test]
fn test_flatten_name_plugin_with_underscore() {
    assert_eq!(flatten_name("my_plugin", "foo"), "my_plugin_foo");
}

#[test]
fn test_flatten_name_empty_plugin_name() {
    assert_eq!(flatten_name("", "foo"), "_foo");
}

// =========================================================================
// build_components: ネスト + 平坦化
// =========================================================================

#[test]
fn test_plugin_new_with_nested_skill_uses_flat_name() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("skills/bar/foo/SKILL.md"), "# Skill");

    let plugin = Plugin::new(
        make_manifest("myplugin"),
        path.clone(),
        PluginOrigin::from_marketplace("test", "test"),
    )
    .unwrap();

    let components = plugin.components();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].kind, ComponentKind::Skill);
    assert_eq!(components[0].name, "myplugin_foo");
    assert_eq!(components[0].path, path.join("skills/bar/foo"));
}

#[test]
fn test_plugin_new_skill_and_agent_same_name_no_collision() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("skills/foo/SKILL.md"), "# Skill");
    write_file(&path.join("agents/foo.agent.md"), "# Agent");

    let plugin = Plugin::new(
        make_manifest("myplugin"),
        path.clone(),
        PluginOrigin::from_marketplace("test", "test"),
    )
    .unwrap();

    let names: Vec<(_, _)> = plugin
        .components()
        .iter()
        .map(|c| (c.kind, c.name.as_str()))
        .collect();
    assert!(names.contains(&(ComponentKind::Skill, "myplugin_foo")));
    assert!(names.contains(&(ComponentKind::Agent, "myplugin_foo")));
}

#[test]
fn test_plugin_new_distinct_basenames_no_collision() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("skills/foo/SKILL.md"), "# foo");
    write_file(&path.join("skills/bar/baz/SKILL.md"), "# baz");

    let plugin = Plugin::new(
        make_manifest("myplugin"),
        path,
        PluginOrigin::from_marketplace("test", "test"),
    )
    .unwrap();

    let mut skill_names: Vec<String> = plugin
        .components()
        .iter()
        .filter(|c| c.kind == ComponentKind::Skill)
        .map(|c| c.name.clone())
        .collect();
    skill_names.sort();
    assert_eq!(skill_names, vec!["myplugin_baz", "myplugin_foo"]);
}

#[test]
fn test_plugin_new_collision_in_nested_skills_returns_validation_error() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("skills/a/foo/SKILL.md"), "# a/foo");
    write_file(&path.join("skills/b/foo/SKILL.md"), "# b/foo");

    let result = Plugin::new(
        make_manifest("myplugin"),
        path,
        PluginOrigin::from_marketplace("test", "test"),
    );

    let err = result.unwrap_err();
    let msg = match err {
        PlmError::Validation(s) => s,
        other => panic!("expected Validation error, got: {:?}", other),
    };
    assert!(msg.contains("myplugin_foo"), "msg: {msg}");
    assert!(msg.contains("Skill"), "msg: {msg}");
    assert!(msg.contains("conflicts with"), "msg: {msg}");
}

#[test]
fn test_plugin_new_collision_flat_and_nested_returns_validation_error() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("skills/foo/SKILL.md"), "# flat");
    write_file(&path.join("skills/bar/foo/SKILL.md"), "# nested");

    let result = Plugin::new(
        make_manifest("myplugin"),
        path,
        PluginOrigin::from_marketplace("test", "test"),
    );
    assert!(matches!(result, Err(PlmError::Validation(_))));
}

// =========================================================================
// パストラバーサル / 不正な plugin name の拒否
// =========================================================================

#[test]
fn test_plugin_new_rejects_plugin_name_with_path_separator() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("skills/foo/SKILL.md"), "# Skill");

    let result = Plugin::new(
        make_manifest("../evil"),
        path,
        PluginOrigin::from_marketplace("test", "test"),
    );
    let err = result.unwrap_err();
    let msg = match err {
        PlmError::Validation(s) => s,
        other => panic!("expected Validation error, got {:?}", other),
    };
    assert!(msg.contains("plugin name"), "msg: {msg}");
}

#[test]
fn test_plugin_new_rejects_plugin_name_with_backslash() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("skills/foo/SKILL.md"), "# Skill");

    let result = Plugin::new(
        make_manifest("a\\b"),
        path,
        PluginOrigin::from_marketplace("test", "test"),
    );
    assert!(matches!(result, Err(PlmError::Validation(_))));
}

#[test]
fn test_plugin_new_rejects_empty_plugin_name() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("skills/foo/SKILL.md"), "# Skill");

    let result = Plugin::new(
        make_manifest(""),
        path,
        PluginOrigin::from_marketplace("test", "test"),
    );
    assert!(matches!(result, Err(PlmError::Validation(_))));
}

#[test]
fn test_plugin_new_hook_uses_flat_name() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("hooks/pre-commit.json"), "{}");

    let plugin = Plugin::new(
        make_manifest("myplugin"),
        path.clone(),
        PluginOrigin::from_marketplace("test", "test"),
    )
    .unwrap();

    let components = plugin.components();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].kind, ComponentKind::Hook);
    assert_eq!(components[0].name, "myplugin_pre-commit");
    assert_eq!(
        components[0].path,
        path.join("hooks").join("pre-commit.json")
    );
}

#[test]
fn test_plugin_new_hook_collision_returns_validation_error() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    // 同 stem の Hook を 2 つ配置する: detect_name_collisions が拾うはず
    write_file(&path.join("hooks/pre-commit.sh"), "#!/bin/sh\n");
    write_file(&path.join("hooks/pre-commit.json"), "{}");

    let result = Plugin::new(
        make_manifest("myplugin"),
        path,
        PluginOrigin::from_marketplace("test", "test"),
    );

    let err = result.unwrap_err();
    let msg = match err {
        PlmError::Validation(s) => s,
        other => panic!("expected Validation error, got: {:?}", other),
    };
    assert!(msg.contains("myplugin_pre-commit"), "msg: {msg}");
    assert!(msg.contains("Hook"), "msg: {msg}");
}

#[test]
fn test_plugin_new_ignores_hooks_in_subdirectories() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    // ディレクトリ階層に置かれた Hook（再帰スキャンで `..` 等が混入した想定）
    write_file(&path.join("hooks/sub/inner.json"), "{}");

    // Hook は再帰スキャンしないので通常パスでは混入しないが、
    // 念のため list_hook_names が単一セグメントを返すことを保証する目的で
    // 通常の sub-dir 内ファイルは無視されることを確認する。
    let plugin = Plugin::new(
        make_manifest("myplugin"),
        path,
        PluginOrigin::from_marketplace("test", "test"),
    )
    .unwrap();

    // hooks サブディレクトリは無視されて Hook 0 件
    let hooks: Vec<&Component> = plugin
        .components()
        .iter()
        .filter(|c| c.kind == ComponentKind::Hook)
        .collect();
    assert!(hooks.is_empty());
}

#[test]
fn test_plugin_new_rejects_plugin_name_dotdot() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    write_file(&path.join("skills/foo/SKILL.md"), "# Skill");

    let result = Plugin::new(
        make_manifest(".."),
        path,
        PluginOrigin::from_marketplace("test", "test"),
    );
    assert!(matches!(result, Err(PlmError::Validation(_))));
}
