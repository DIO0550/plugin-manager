//! AntigravityTarget unit tests

use super::*;
use crate::component::{ComponentRef, PlacementScope, ProjectContext};
use crate::target::PluginOrigin;
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
    assert_eq!(supported.len(), 2);
    assert!(supported.contains(&ComponentKind::Skill));
    assert!(supported.contains(&ComponentKind::Hook));
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
fn test_antigravity_supports_hook() {
    let target = AntigravityTarget::new();
    assert!(target.supports(ComponentKind::Hook));
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
    // Personal scope uses ~/.gemini/config/skills/<original_name>/
    let home = std::env::var("HOME").unwrap();
    let expected = std::path::PathBuf::from(home)
        .join(".gemini")
        .join("config")
        .join("skills")
        .join("my-skill");
    assert_eq!(location.as_path(), expected.as_path());
}

#[test]
fn test_antigravity_placement_location_skill_project() {
    let target = AntigravityTarget::new();
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
    // Project scope uses .agents/skills/<original_name>/
    assert_eq!(
        location.as_path(),
        Path::new("/project/.agents/skills/my-skill")
    );
}

#[test]
fn test_antigravity_placement_location_skill_without_original_name_returns_none() {
    let target = AntigravityTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");
    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Skill, "my-plugin_my-skill"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };

    assert!(target.placement_location(&ctx).is_none());
}

#[test]
fn test_antigravity_placement_location_agent_returns_none() {
    let target = AntigravityTarget::new();
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
fn test_antigravity_placement_with_hierarchy() {
    let target = AntigravityTarget::new();
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
        Path::new("/project/.agents/skills/my-skill")
    );
}

#[test]
fn test_antigravity_placement_location_skill_with_prefixed_name() {
    let target = AntigravityTarget::new();
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
    assert_eq!(location.as_path(), Path::new("/project/.agents/skills/foo"));
}

#[test]
fn test_antigravity_list_placed_empty_dir() {
    let target = AntigravityTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // No .agents directory exists
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

    // 公式 1 階層: .agents/skills/<original_name>/SKILL.md
    let skill_path = project_root.join(".agents").join("skills").join("skill-1");
    std::fs::create_dir_all(&skill_path).unwrap();
    std::fs::write(skill_path.join("SKILL.md"), "# Skill 1").unwrap();

    let result = target
        .list_placed(ComponentKind::Skill, Scope::Project, project_root)
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], "skill-1");
}

#[test]
fn test_antigravity_list_placed_no_skill_md() {
    let target = AntigravityTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // SKILL.md 不在のディレクトリは無視される（フラット構造）
    let skill_path = project_root
        .join(".agents")
        .join("skills")
        .join("empty-skill");
    std::fs::create_dir_all(&skill_path).unwrap();

    let result = target
        .list_placed(ComponentKind::Skill, Scope::Project, project_root)
        .unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_antigravity_list_placed_ignores_legacy_skill_path() {
    let target = AntigravityTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    let legacy_skill = project_root
        .join(".agent")
        .join("skills")
        .join("plugin_old-skill");
    std::fs::create_dir_all(&legacy_skill).unwrap();
    std::fs::write(legacy_skill.join("SKILL.md"), "# Legacy").unwrap();

    let result = target
        .list_placed(ComponentKind::Skill, Scope::Project, project_root)
        .unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_antigravity_legacy_cleanup_removes_old_project_skill_path() {
    let target = AntigravityTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();
    let legacy_path = project_root
        .join(".agent")
        .join("skills")
        .join("my-plugin_my-skill");
    std::fs::create_dir_all(&legacy_path).unwrap();

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

    let operations = target.legacy_cleanup_operations(&ctx).unwrap();
    assert_eq!(operations.len(), 1);
    assert!(matches!(
        &operations[0],
        FileOperation::RemoveDir { path } if path.as_path() == legacy_path
    ));
}

#[test]
fn test_antigravity_legacy_cleanup_is_empty_when_old_path_is_missing() {
    let target = AntigravityTarget::new();
    let temp_dir = TempDir::new().unwrap();
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
        project: ProjectContext::new(temp_dir.path()),
    };

    assert!(target.legacy_cleanup_operations(&ctx).unwrap().is_empty());
}

#[test]
fn test_antigravity_skill_overwrite_error_rejects_unmanaged_existing_skill() {
    let target_root = TempDir::new().unwrap();
    let target_path = target_root.path().join("skills").join("my-skill");
    std::fs::create_dir_all(&target_path).unwrap();

    let plugin_root = TempDir::new().unwrap();
    let error = AntigravityTarget::skill_overwrite_error(&target_path, plugin_root.path());
    assert!(error.is_some());
    assert!(error.unwrap().contains("already exists"));
}

#[test]
fn test_antigravity_skill_overwrite_error_allows_owned_skill() {
    let target_root = TempDir::new().unwrap();
    let target_path = target_root.path().join("skills").join("my-skill");
    std::fs::create_dir_all(&target_path).unwrap();

    let plugin_root = TempDir::new().unwrap();
    let mut meta = crate::plugin::meta::PluginMeta::default();
    meta.add_managed_file("antigravity", &target_path);
    crate::plugin::meta::write_meta(plugin_root.path(), &meta).unwrap();

    assert!(AntigravityTarget::skill_overwrite_error(&target_path, plugin_root.path()).is_none());
}

#[test]
fn test_antigravity_post_place_records_skill_ownership() {
    let target = AntigravityTarget::new();
    let plugin_root = TempDir::new().unwrap();
    let deployed_root = TempDir::new().unwrap();
    let deployed_path = deployed_root.path().join("skills").join("my-skill");
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
        project: ProjectContext::new(Path::new("/project")),
    };

    target.post_place(&ctx, &deployed_path, plugin_root.path(), false);

    let meta = crate::plugin::meta::load_meta(plugin_root.path()).unwrap();
    assert!(meta.manages_file("antigravity", &deployed_path));
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

#[test]
fn test_antigravity_placement_location_hook_project() {
    let target = AntigravityTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Hook, "my-hooks"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();
    assert!(location.is_file());
    assert_eq!(location.as_path(), Path::new("/project/.agents/hooks.json"));
}

#[test]
fn test_antigravity_placement_location_hook_personal() {
    let target = AntigravityTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Hook, "my-hooks"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();
    let home = std::env::var("HOME").unwrap();
    let expected = std::path::PathBuf::from(home)
        .join(".gemini")
        .join("config")
        .join("hooks.json");
    assert_eq!(location.as_path(), expected.as_path());
}

#[test]
fn test_antigravity_hook_component_conflict_rejects_multiple() {
    use crate::component::Component;
    use std::path::PathBuf;

    let components = vec![
        Component::new(ComponentKind::Hook, "a", PathBuf::from("hooks/a.json")),
        Component::new(ComponentKind::Hook, "b", PathBuf::from("hooks/b.json")),
    ];
    let err = AntigravityTarget::hook_component_conflict_error(&components);
    assert!(err.unwrap().contains("single hooks.json"));
}

#[test]
fn test_antigravity_list_placed_hooks() {
    let target = AntigravityTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    let hooks_dir = project_root.join(".agents");
    std::fs::create_dir_all(&hooks_dir).unwrap();
    std::fs::write(hooks_dir.join("hooks.json"), "{}").unwrap();

    let result = target
        .list_placed(ComponentKind::Hook, Scope::Project, project_root)
        .unwrap();
    assert_eq!(result, vec!["hooks"]);
}
