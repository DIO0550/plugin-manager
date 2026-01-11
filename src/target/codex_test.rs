use super::*;

#[test]
fn test_codex_target_name() {
    let target = CodexTarget::new();
    assert_eq!(target.name(), "codex");
    assert_eq!(target.display_name(), "OpenAI Codex");
}

#[test]
fn test_codex_supported_components() {
    let target = CodexTarget::new();
    assert!(target.supports(ComponentKind::Skill));
    assert!(target.supports(ComponentKind::Agent));
    assert!(target.supports(ComponentKind::Instruction));
    assert!(!target.supports(ComponentKind::Command));
    assert!(!target.supports(ComponentKind::Hook));
}

#[test]
fn test_codex_placement_location_skill_with_hierarchy() {
    let target = CodexTarget::new();
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
    assert_eq!(
        location.as_path(),
        Path::new("/project/.codex/skills/official/my-plugin/my-skill")
    );
}

#[test]
fn test_codex_placement_location_skill_github_direct() {
    let target = CodexTarget::new();
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
        Path::new("/project/.codex/skills/github/owner--repo/my-skill")
    );
}

#[test]
fn test_codex_placement_location_agent() {
    let target = CodexTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Agent, "my-agent"),
        origin: &origin,
        scope: PlacementScope(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_file());
    assert_eq!(
        location.as_path(),
        Path::new("/project/.codex/agents/official/my-plugin/my-agent.agent.md")
    );
}

#[test]
fn test_codex_placement_location_instruction() {
    let target = CodexTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    // Project scope
    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Instruction, "test"),
        origin: &origin,
        scope: PlacementScope(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_file());
    assert_eq!(location.as_path(), Path::new("/project/AGENTS.md"));
}

#[test]
fn test_codex_command_not_supported() {
    let target = CodexTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("test", "test");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Command, "test"),
        origin: &origin,
        scope: PlacementScope(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    assert!(target.placement_location(&ctx).is_none());
}
