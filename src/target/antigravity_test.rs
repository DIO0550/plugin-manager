//! AntigravityTarget unit tests

use super::*;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_antigravity_name() {
    let target = AntigravityTarget::new();
    assert_eq!(target.name(), "antigravity");
}

#[test]
fn test_antigravity_display_name() {
    let target = AntigravityTarget::new();
    assert_eq!(target.display_name(), "Google Antigravity");
}

#[test]
fn test_antigravity_supported_components() {
    let target = AntigravityTarget::new();
    let supported = target.supported_components();
    assert_eq!(supported.len(), 1);
    assert_eq!(supported[0], ComponentKind::Skill);
}

#[test]
fn test_antigravity_supports_skill() {
    let target = AntigravityTarget::new();
    assert!(target.supports(ComponentKind::Skill));
}

#[test]
fn test_antigravity_not_supports_agent() {
    let target = AntigravityTarget::new();
    assert!(!target.supports(ComponentKind::Agent));
}

#[test]
fn test_antigravity_not_supports_command() {
    let target = AntigravityTarget::new();
    assert!(!target.supports(ComponentKind::Command));
}

#[test]
fn test_antigravity_not_supports_instruction() {
    let target = AntigravityTarget::new();
    assert!(!target.supports(ComponentKind::Instruction));
}

#[test]
fn test_antigravity_not_supports_hook() {
    let target = AntigravityTarget::new();
    assert!(!target.supports(ComponentKind::Hook));
}

#[test]
fn test_antigravity_supports_scope_skill_personal() {
    let target = AntigravityTarget::new();
    assert!(target.supports_scope(ComponentKind::Skill, Scope::Personal));
}

#[test]
fn test_antigravity_supports_scope_skill_project() {
    let target = AntigravityTarget::new();
    assert!(target.supports_scope(ComponentKind::Skill, Scope::Project));
}

#[test]
fn test_antigravity_placement_location_skill_personal() {
    let target = AntigravityTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Skill, "my-skill"),
        origin: &origin,
        scope: PlacementScope(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_dir());
    // Personal scope uses ~/.gemini/antigravity/skills/
    let home = std::env::var("HOME").unwrap();
    let expected = format!(
        "{}/.gemini/antigravity/skills/official/my-plugin/my-skill",
        home
    );
    assert_eq!(location.as_path(), Path::new(&expected));
}

#[test]
fn test_antigravity_placement_location_skill_project() {
    let target = AntigravityTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Skill, "my-skill"),
        origin: &origin,
        scope: PlacementScope(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_dir());
    // Project scope uses .agent/skills/
    assert_eq!(
        location.as_path(),
        Path::new("/project/.agent/skills/official/my-plugin/my-skill")
    );
}

#[test]
fn test_antigravity_placement_location_agent_returns_none() {
    let target = AntigravityTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Agent, "my-agent"),
        origin: &origin,
        scope: PlacementScope(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    assert!(target.placement_location(&ctx).is_none());
}

#[test]
fn test_antigravity_placement_with_hierarchy() {
    let target = AntigravityTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_github("owner", "repo");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Skill, "my-skill"),
        origin: &origin,
        scope: PlacementScope(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_dir());
    assert_eq!(
        location.as_path(),
        Path::new("/project/.agent/skills/github/owner--repo/my-skill")
    );
}

#[test]
fn test_antigravity_list_placed_empty_dir() {
    let target = AntigravityTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // No .agent directory exists
    let result = target
        .list_placed(ComponentKind::Skill, Scope::Project, project_root)
        .unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_antigravity_list_placed_with_skills() {
    let target = AntigravityTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create skill directory structure with SKILL.md
    let skill_path = project_root
        .join(".agent")
        .join("skills")
        .join("marketplace")
        .join("plugin")
        .join("skill-1");
    std::fs::create_dir_all(&skill_path).unwrap();
    std::fs::write(skill_path.join("SKILL.md"), "# Skill 1").unwrap();

    let result = target
        .list_placed(ComponentKind::Skill, Scope::Project, project_root)
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], "marketplace/plugin/skill-1");
}

#[test]
fn test_antigravity_list_placed_no_skill_md() {
    let target = AntigravityTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create skill directory without SKILL.md (should be ignored)
    let skill_path = project_root
        .join(".agent")
        .join("skills")
        .join("marketplace")
        .join("plugin")
        .join("empty-skill");
    std::fs::create_dir_all(&skill_path).unwrap();

    let result = target
        .list_placed(ComponentKind::Skill, Scope::Project, project_root)
        .unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_antigravity_list_placed_agent_returns_empty() {
    let target = AntigravityTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Agent is not supported, should return empty
    let result = target
        .list_placed(ComponentKind::Agent, Scope::Project, project_root)
        .unwrap();
    assert!(result.is_empty());
}
