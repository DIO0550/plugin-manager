//! CursorTarget unit tests

use super::*;
use crate::component::{ComponentRef, PlacementScope, ProjectContext};
use crate::target::PluginOrigin;
use std::path::Path;
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
    assert_eq!(supported.len(), 3);
    assert!(supported.contains(&ComponentKind::Skill));
    assert!(supported.contains(&ComponentKind::Agent));
    assert!(supported.contains(&ComponentKind::Command));
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
fn test_cursor_not_supports_instruction() {
    let target = CursorTarget::new();
    assert!(!target.supports(ComponentKind::Instruction));
}

#[test]
fn test_cursor_not_supports_hook() {
    let target = CursorTarget::new();
    assert!(!target.supports(ComponentKind::Hook));
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
fn test_cursor_placement_location_skill_personal() {
    // インストール経路では `Component.name` が `flatten_name(plugin, original)
    // = "{plugin}_{original}"` に平坦化されるため、テストもその形を使う。
    let target = CursorTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Skill, "my-plugin_my-skill"),
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
        .join("my-plugin_my-skill");
    assert_eq!(location.as_path(), expected.as_path());
}

#[test]
fn test_cursor_placement_location_skill_project() {
    let target = CursorTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Skill, "my-plugin_my-skill"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_dir());
    assert_eq!(
        location.as_path(),
        Path::new("/project/.cursor/skills/my-plugin_my-skill")
    );
}

#[test]
fn test_cursor_placement_location_skill_with_prefixed_name() {
    let target = CursorTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Skill, "myplugin_foo"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();
    assert_eq!(
        location.as_path(),
        Path::new("/project/.cursor/skills/myplugin_foo")
    );
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
fn test_cursor_placement_location_instruction_returns_none() {
    let target = CursorTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Instruction, "test"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    assert!(target.placement_location(&ctx).is_none());
}

#[test]
fn test_cursor_placement_location_hook_returns_none() {
    let target = CursorTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Hook, "test"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    assert!(target.placement_location(&ctx).is_none());
}

#[test]
fn test_cursor_placement_with_github_origin() {
    let target = CursorTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_github("owner", "repo");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Skill, "my-plugin_my-skill"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_dir());
    assert_eq!(
        location.as_path(),
        Path::new("/project/.cursor/skills/my-plugin_my-skill")
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

    // フラット 2 階層: .cursor/skills/<flattened_name>/SKILL.md
    let skill_path = project_root
        .join(".cursor")
        .join("skills")
        .join("plugin_skill-1");
    std::fs::create_dir_all(&skill_path).unwrap();
    std::fs::write(skill_path.join("SKILL.md"), "# Skill 1").unwrap();

    let result = target
        .list_placed(ComponentKind::Skill, Scope::Project, project_root)
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], "plugin_skill-1");
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
