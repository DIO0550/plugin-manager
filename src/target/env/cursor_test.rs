//! CursorTarget unit tests

use super::*;
use crate::component::{ComponentRef, PlacementScope, ProjectContext};
use crate::target::paths::home_dir;
use crate::target::PluginOrigin;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[test]
fn test_cursor_name() {
    let target = CursorTarget::new();
    assert_eq!(target.name(), "cursor");
}

#[test]
fn test_cursor_display_name() {
    let target = CursorTarget::new();
    assert_eq!(target.display_name(), "Cursor");
}

#[test]
fn test_cursor_kind() {
    let target = CursorTarget::new();
    assert_eq!(target.kind(), TargetKind::Cursor);
}

#[test]
fn test_cursor_supported_components() {
    let target = CursorTarget::new();
    let supported = target.supported_components();
    assert_eq!(supported.len(), 5);
    assert!(supported.contains(&ComponentKind::Skill));
    assert!(supported.contains(&ComponentKind::Agent));
    assert!(supported.contains(&ComponentKind::Command));
    assert!(supported.contains(&ComponentKind::Instruction));
    assert!(supported.contains(&ComponentKind::Hook));
}

#[test]
fn test_cursor_supports_skill() {
    let target = CursorTarget::new();
    assert!(target.supports(ComponentKind::Skill));
}

#[test]
fn test_cursor_supports_agent() {
    let target = CursorTarget::new();
    assert!(target.supports(ComponentKind::Agent));
}

#[test]
fn test_cursor_supports_command() {
    let target = CursorTarget::new();
    assert!(target.supports(ComponentKind::Command));
}

#[test]
fn test_cursor_supports_instruction() {
    let target = CursorTarget::new();
    assert!(target.supports(ComponentKind::Instruction));
}

#[test]
fn test_cursor_supports_hook() {
    let target = CursorTarget::new();
    assert!(target.supports(ComponentKind::Hook));
}

#[test]
fn test_cursor_supports_scope_skill_personal() {
    let target = CursorTarget::new();
    assert!(target.supports_scope(ComponentKind::Skill, Scope::Personal));
}

#[test]
fn test_cursor_supports_scope_skill_project() {
    let target = CursorTarget::new();
    assert!(target.supports_scope(ComponentKind::Skill, Scope::Project));
}

#[test]
fn test_cursor_supports_scope_agent() {
    let target = CursorTarget::new();
    assert!(target.supports_scope(ComponentKind::Agent, Scope::Personal));
    assert!(target.supports_scope(ComponentKind::Agent, Scope::Project));
}

#[test]
fn test_cursor_supports_scope_command() {
    let target = CursorTarget::new();
    assert!(target.supports_scope(ComponentKind::Command, Scope::Personal));
    assert!(target.supports_scope(ComponentKind::Command, Scope::Project));
}

#[test]
fn test_cursor_supports_scope_instruction_project() {
    let target = CursorTarget::new();
    assert!(target.supports_scope(ComponentKind::Instruction, Scope::Project));
}

#[test]
fn test_cursor_supports_scope_instruction_personal_returns_false() {
    let target = CursorTarget::new();
    assert!(!target.supports_scope(ComponentKind::Instruction, Scope::Personal));
}

#[test]
fn test_cursor_placement_location_skill_personal() {
    // Cursor Skill は frontmatter name 一致のため original_name で配置する (#377)。
    let target = CursorTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::with_names(
            ComponentKind::Skill,
            "my-plugin_my-skill",
            "my-skill",
            "my-plugin",
        ),
        origin: &origin,
        scope: PlacementScope::new(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_dir());
    let home = std::env::var("HOME").unwrap();
    let expected = std::path::PathBuf::from(home)
        .join(".cursor")
        .join("skills")
        .join("my-skill");
    assert_eq!(location.as_path(), expected.as_path());
}

#[test]
fn test_cursor_placement_location_skill_project() {
    let target = CursorTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::with_names(
            ComponentKind::Skill,
            "my-plugin_my-skill",
            "my-skill",
            "my-plugin",
        ),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_dir());
    assert_eq!(
        location.as_path(),
        Path::new("/project/.cursor/skills/my-skill")
    );
}

#[test]
fn test_cursor_placement_location_skill_with_prefixed_name() {
    let target = CursorTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::with_names(
            ComponentKind::Skill,
            "myplugin_foo",
            "foo",
            "myplugin",
        ),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();
    assert_eq!(location.as_path(), Path::new("/project/.cursor/skills/foo"));
}

#[test]
fn test_cursor_placement_location_agent_personal() {
    let target = CursorTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Agent, "my-plugin_my-agent"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_file());
    let home = std::env::var("HOME").unwrap();
    let expected = std::path::PathBuf::from(home)
        .join(".cursor")
        .join("agents")
        .join("my-plugin_my-agent.md");
    assert_eq!(location.as_path(), expected.as_path());
}

#[test]
fn test_cursor_placement_location_agent_project() {
    let target = CursorTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Agent, "my-plugin_my-agent"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_file());
    assert_eq!(
        location.as_path(),
        Path::new("/project/.cursor/agents/my-plugin_my-agent.md")
    );
}

#[test]
fn test_cursor_placement_location_command_personal() {
    let target = CursorTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Command, "my-plugin_my-command"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_file());
    let home = std::env::var("HOME").unwrap();
    let expected = std::path::PathBuf::from(home)
        .join(".cursor")
        .join("commands")
        .join("my-plugin_my-command.md");
    assert_eq!(location.as_path(), expected.as_path());
}

#[test]
fn test_cursor_placement_location_command_project() {
    let target = CursorTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Command, "my-plugin_my-command"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_file());
    assert_eq!(
        location.as_path(),
        Path::new("/project/.cursor/commands/my-plugin_my-command.md")
    );
}

#[test]
fn test_cursor_placement_location_instruction_project() {
    let target = CursorTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Instruction, "test"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_file());
    assert_eq!(location.as_path(), Path::new("/project/AGENTS.md"));
}

#[test]
fn test_cursor_placement_location_instruction_personal_returns_none() {
    let target = CursorTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Instruction, "test"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    assert!(target.placement_location(&ctx).is_none());
}

#[test]
fn test_cursor_placement_location_hook_project() {
    let target = CursorTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Hook, "test"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();
    assert!(location.is_file());
    assert_eq!(location.as_path(), Path::new("/project/.cursor/hooks.json"));
}

#[test]
fn test_cursor_placement_location_hook_personal() {
    let target = CursorTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Hook, "test"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();
    assert!(location.is_file());
    assert_eq!(location.as_path(), home_dir().join(".cursor/hooks.json"));
}

#[test]
fn hook_overwrite_error_returns_none_when_target_does_not_exist() {
    let temp = tempfile::TempDir::new().unwrap();
    let target_path = temp.path().join("hooks.json");
    let plugin_root = temp.path();

    assert!(CursorTarget::hook_overwrite_error(&target_path, plugin_root).is_none());
}

#[test]
fn hook_overwrite_error_returns_error_when_target_exists_and_not_managed() {
    let temp = tempfile::TempDir::new().unwrap();
    let target_path = temp.path().join("hooks.json");
    std::fs::write(&target_path, "{}").unwrap();

    let plugin_root_dir = tempfile::TempDir::new().unwrap();
    let result = CursorTarget::hook_overwrite_error(&target_path, plugin_root_dir.path());
    assert!(result.is_some());
    let msg = result.unwrap();
    assert!(msg.contains("already exists"));
    assert!(msg.contains("not managed by this plugin"));
}

#[test]
fn hook_overwrite_error_returns_none_when_plugin_already_owns_target_path() {
    let temp = tempfile::TempDir::new().unwrap();
    let target_path = temp.path().join("hooks.json");
    std::fs::write(&target_path, "{}").unwrap();

    let plugin_root_dir = tempfile::TempDir::new().unwrap();
    let mut meta = crate::plugin::meta::PluginMeta::default();
    meta.add_managed_file("cursor", &target_path);
    crate::plugin::meta::write_meta(plugin_root_dir.path(), &meta).unwrap();

    assert!(
        CursorTarget::hook_overwrite_error(&target_path, plugin_root_dir.path()).is_none(),
        "re-install of the same plugin must be allowed for a managed file path"
    );
}

#[test]
fn hook_component_conflict_error_rejects_multiple_hooks() {
    let components = vec![
        Component::new(ComponentKind::Hook, "hook-a", PathBuf::from("hooks/a.json")),
        Component::new(ComponentKind::Hook, "hook-b", PathBuf::from("hooks/b.json")),
    ];
    let err = CursorTarget::hook_component_conflict_error(&components);
    assert!(err.is_some());
    assert!(err.unwrap().contains("single hooks.json"));
}

#[test]
fn test_cursor_placement_with_github_origin() {
    let target = CursorTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_github("owner", "repo");

    let ctx = PlacementContext {
        component: ComponentRef::with_names(
            ComponentKind::Skill,
            "my-plugin_my-skill",
            "my-skill",
            "my-plugin",
        ),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_dir());
    assert_eq!(
        location.as_path(),
        Path::new("/project/.cursor/skills/my-skill")
    );
}

#[test]
fn test_cursor_list_placed_empty_dir() {
    let target = CursorTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    let result = target
        .list_placed(ComponentKind::Skill, Scope::Project, project_root)
        .unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_cursor_list_placed_with_skills() {
    let target = CursorTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Skill は original_name で配置される（#377）
    let skill_path = project_root.join(".cursor").join("skills").join("skill-1");
    std::fs::create_dir_all(&skill_path).unwrap();
    std::fs::write(skill_path.join("SKILL.md"), "# Skill 1").unwrap();

    let result = target
        .list_placed(ComponentKind::Skill, Scope::Project, project_root)
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], "skill-1");
}

#[test]
fn test_cursor_list_placed_no_skill_md() {
    let target = CursorTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    let skill_path = project_root
        .join(".cursor")
        .join("skills")
        .join("plugin_empty-skill");
    std::fs::create_dir_all(&skill_path).unwrap();

    let result = target
        .list_placed(ComponentKind::Skill, Scope::Project, project_root)
        .unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_cursor_list_placed_with_agents() {
    let target = CursorTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    let agents_dir = project_root.join(".cursor").join("agents");
    std::fs::create_dir_all(&agents_dir).unwrap();
    std::fs::write(agents_dir.join("plugin_agent-1.md"), "# Agent 1").unwrap();

    let result = target
        .list_placed(ComponentKind::Agent, Scope::Project, project_root)
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], "plugin_agent-1");
}

#[test]
fn test_cursor_list_placed_with_commands() {
    let target = CursorTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    let commands_dir = project_root.join(".cursor").join("commands");
    std::fs::create_dir_all(&commands_dir).unwrap();
    std::fs::write(commands_dir.join("plugin_cmd-1.md"), "# Command 1").unwrap();

    let result = target
        .list_placed(ComponentKind::Command, Scope::Project, project_root)
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], "plugin_cmd-1");
}

#[test]
fn test_cursor_list_placed_ignores_agent_md_suffix() {
    let target = CursorTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    let agents_dir = project_root.join(".cursor").join("agents");
    std::fs::create_dir_all(&agents_dir).unwrap();
    std::fs::write(agents_dir.join("legacy.agent.md"), "# Legacy").unwrap();

    let result = target
        .list_placed(ComponentKind::Agent, Scope::Project, project_root)
        .unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_cursor_list_placed_instruction_exists() {
    let target = CursorTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    std::fs::write(project_root.join("AGENTS.md"), "# Agents").unwrap();

    let result = target
        .list_placed(ComponentKind::Instruction, Scope::Project, project_root)
        .unwrap();
    assert_eq!(result, vec!["AGENTS.md".to_string()]);
}

#[test]
fn test_cursor_list_placed_instruction_not_exists() {
    let target = CursorTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    let result = target
        .list_placed(ComponentKind::Instruction, Scope::Project, project_root)
        .unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_cursor_list_placed_instruction_personal_returns_empty() {
    let target = CursorTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    std::fs::write(project_root.join("AGENTS.md"), "# Agents").unwrap();

    let result = target
        .list_placed(ComponentKind::Instruction, Scope::Personal, project_root)
        .unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_cursor_command_and_agent_format_are_claude_code() {
    let target = CursorTarget::new();
    assert_eq!(
        target.command_format(),
        crate::component::CommandFormat::ClaudeCode
    );
    assert_eq!(
        target.agent_format(),
        crate::component::AgentFormat::ClaudeCode
    );
}

#[test]
fn skill_overwrite_error_returns_none_when_target_does_not_exist() {
    let temp = tempfile::TempDir::new().unwrap();
    let target_path = temp.path().join("skills").join("my-skill");
    let plugin_root = temp.path();

    assert!(CursorTarget::skill_overwrite_error(&target_path, plugin_root).is_none());
}

#[test]
fn skill_overwrite_error_returns_error_when_target_exists_and_not_managed() {
    let temp = tempfile::TempDir::new().unwrap();
    let target_path = temp.path().join("skills").join("my-skill");
    std::fs::create_dir_all(&target_path).unwrap();
    std::fs::write(target_path.join("SKILL.md"), "---\nname: my-skill\n---\n").unwrap();

    let plugin_root_dir = tempfile::TempDir::new().unwrap();
    let result = CursorTarget::skill_overwrite_error(&target_path, plugin_root_dir.path());
    assert!(result.is_some());
    assert!(result.unwrap().contains("already exists"));
}

#[test]
fn skill_overwrite_error_returns_none_when_plugin_already_owns_target_path() {
    let temp = tempfile::TempDir::new().unwrap();
    let target_path = temp.path().join("skills").join("my-skill");
    std::fs::create_dir_all(&target_path).unwrap();

    let plugin_root_dir = tempfile::TempDir::new().unwrap();
    let mut meta = crate::plugin::meta::PluginMeta::default();
    meta.add_managed_file("cursor", &target_path);
    crate::plugin::meta::write_meta(plugin_root_dir.path(), &meta).unwrap();

    assert!(
        CursorTarget::skill_overwrite_error(&target_path, plugin_root_dir.path()).is_none(),
        "re-install of the same plugin must be allowed for a managed skill path"
    );
}

#[test]
fn legacy_flattened_skill_path_uses_skills_subdir() {
    let project_root = Path::new("/project");
    let path =
        CursorTarget::legacy_flattened_skill_path(Scope::Project, project_root, "plugin_skill");
    assert_eq!(path, Path::new("/project/.cursor/skills/plugin_skill"));
}
