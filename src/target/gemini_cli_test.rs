//! GeminiCliTarget unit tests

use super::*;
use crate::component::{ComponentRef, PlacementScope, ProjectContext};
use crate::target::PluginOrigin;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_gemini_cli_name() {
    let target = GeminiCliTarget::new();
    assert_eq!(target.name(), "gemini");
}

#[test]
fn test_gemini_cli_display_name() {
    let target = GeminiCliTarget::new();
    assert_eq!(target.display_name(), "Gemini CLI");
}

#[test]
fn test_gemini_cli_supported_components() {
    let target = GeminiCliTarget::new();
    let supported = target.supported_components();
    assert_eq!(supported.len(), 2);
    assert!(supported.contains(&ComponentKind::Skill));
    assert!(supported.contains(&ComponentKind::Instruction));
}

#[test]
fn test_gemini_cli_supports_skill() {
    let target = GeminiCliTarget::new();
    assert!(target.supports(ComponentKind::Skill));
}

#[test]
fn test_gemini_cli_supports_instruction() {
    let target = GeminiCliTarget::new();
    assert!(target.supports(ComponentKind::Instruction));
}

#[test]
fn test_gemini_cli_not_supports_agent() {
    let target = GeminiCliTarget::new();
    assert!(!target.supports(ComponentKind::Agent));
}

#[test]
fn test_gemini_cli_not_supports_command() {
    let target = GeminiCliTarget::new();
    assert!(!target.supports(ComponentKind::Command));
}

#[test]
fn test_gemini_cli_not_supports_hook() {
    let target = GeminiCliTarget::new();
    assert!(!target.supports(ComponentKind::Hook));
}

#[test]
fn test_gemini_cli_supports_scope_skill_personal() {
    let target = GeminiCliTarget::new();
    assert!(target.supports_scope(ComponentKind::Skill, Scope::Personal));
}

#[test]
fn test_gemini_cli_supports_scope_skill_project() {
    let target = GeminiCliTarget::new();
    assert!(target.supports_scope(ComponentKind::Skill, Scope::Project));
}

#[test]
fn test_gemini_cli_supports_scope_instruction_personal() {
    let target = GeminiCliTarget::new();
    assert!(target.supports_scope(ComponentKind::Instruction, Scope::Personal));
}

#[test]
fn test_gemini_cli_supports_scope_instruction_project() {
    let target = GeminiCliTarget::new();
    assert!(target.supports_scope(ComponentKind::Instruction, Scope::Project));
}

#[test]
fn test_gemini_cli_placement_location_skill_personal() {
    let target = GeminiCliTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Skill, "my-skill"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_dir());
    let home = std::env::var("HOME").unwrap();
    let expected = format!("{}/.gemini/skills/my-skill", home);
    assert_eq!(location.as_path(), Path::new(&expected));
}

#[test]
fn test_gemini_cli_placement_location_skill_project() {
    let target = GeminiCliTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Skill, "my-skill"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_dir());
    assert_eq!(
        location.as_path(),
        Path::new("/project/.gemini/skills/my-skill")
    );
}

#[test]
fn test_gemini_cli_placement_location_skill_with_prefixed_name() {
    let target = GeminiCliTarget::new();
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
        Path::new("/project/.gemini/skills/myplugin_foo")
    );
}

#[test]
fn test_gemini_cli_placement_location_instruction_personal() {
    let target = GeminiCliTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Instruction, "test"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_file());
    let home = std::env::var("HOME").unwrap();
    let expected = format!("{}/.gemini/GEMINI.md", home);
    assert_eq!(location.as_path(), Path::new(&expected));
}

#[test]
fn test_gemini_cli_placement_location_instruction_project() {
    let target = GeminiCliTarget::new();
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
    assert_eq!(location.as_path(), Path::new("/project/GEMINI.md"));
}

#[test]
fn test_gemini_cli_placement_location_agent_returns_none() {
    let target = GeminiCliTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Agent, "my-agent"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    assert!(target.placement_location(&ctx).is_none());
}

#[test]
fn test_gemini_cli_placement_location_command_returns_none() {
    let target = GeminiCliTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Command, "test"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    assert!(target.placement_location(&ctx).is_none());
}

#[test]
fn test_gemini_cli_placement_location_hook_returns_none() {
    let target = GeminiCliTarget::new();
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
fn test_gemini_cli_placement_with_github_origin() {
    let target = GeminiCliTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_github("owner", "repo");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Skill, "my-skill"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_dir());
    assert_eq!(
        location.as_path(),
        Path::new("/project/.gemini/skills/my-skill")
    );
}

#[test]
fn test_gemini_cli_list_placed_empty_dir() {
    let target = GeminiCliTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    let result = target
        .list_placed(ComponentKind::Skill, Scope::Project, project_root)
        .unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_gemini_cli_list_placed_with_skills() {
    let target = GeminiCliTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // フラット 2 階層: .gemini/skills/<flattened_name>/SKILL.md
    let skill_path = project_root
        .join(".gemini")
        .join("skills")
        .join("plugin_skill-1");
    std::fs::create_dir_all(&skill_path).unwrap();
    std::fs::write(skill_path.join("SKILL.md"), "# Skill 1").unwrap();

    let result = target
        .list_placed(ComponentKind::Skill, Scope::Project, project_root)
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], "_/_/plugin_skill-1");
}

#[test]
fn test_gemini_cli_list_placed_no_skill_md() {
    let target = GeminiCliTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    let skill_path = project_root
        .join(".gemini")
        .join("skills")
        .join("plugin_empty-skill");
    std::fs::create_dir_all(&skill_path).unwrap();

    let result = target
        .list_placed(ComponentKind::Skill, Scope::Project, project_root)
        .unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_gemini_cli_list_placed_agent_returns_empty() {
    let target = GeminiCliTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    let result = target
        .list_placed(ComponentKind::Agent, Scope::Project, project_root)
        .unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_gemini_cli_list_placed_instruction_exists() {
    let target = GeminiCliTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    std::fs::write(project_root.join("GEMINI.md"), "# Instructions").unwrap();

    let result = target
        .list_placed(ComponentKind::Instruction, Scope::Project, project_root)
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], "GEMINI.md");
}

#[test]
fn test_gemini_cli_list_placed_instruction_not_exists() {
    let target = GeminiCliTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    let result = target
        .list_placed(ComponentKind::Instruction, Scope::Project, project_root)
        .unwrap();
    assert!(result.is_empty());
}
