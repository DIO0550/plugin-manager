use super::*;

#[test]
fn test_copilot_target_name() {
    let target = CopilotTarget::new();
    assert_eq!(target.name(), "copilot");
    assert_eq!(target.display_name(), "GitHub Copilot");
}

#[test]
fn test_copilot_supported_components() {
    let target = CopilotTarget::new();
    assert!(target.supports(ComponentKind::Skill));
    assert!(target.supports(ComponentKind::Agent));
    assert!(target.supports(ComponentKind::Command));
    assert!(target.supports(ComponentKind::Instruction));
    assert!(!target.supports(ComponentKind::Hook));
}

#[test]
fn test_copilot_skill_personal_not_supported() {
    let target = CopilotTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    // Personal scope for skills is not supported
    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Skill, "my-skill"),
        origin: &origin,
        scope: PlacementScope(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    assert!(target.placement_location(&ctx).is_none());
}

#[test]
fn test_copilot_placement_location_skill_project() {
    let target = CopilotTarget::new();
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
        Path::new("/project/.github/skills/official/my-plugin/my-skill")
    );
}

#[test]
fn test_copilot_placement_location_agent() {
    let target = CopilotTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    // Personal scope
    let ctx_personal = PlacementContext {
        component: ComponentRef::new(ComponentKind::Agent, "test"),
        origin: &origin,
        scope: PlacementScope(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    let location_personal = target.placement_location(&ctx_personal).unwrap();
    assert!(location_personal.is_file());
    assert!(location_personal
        .as_path()
        .to_string_lossy()
        .contains(".copilot/agents/official/my-plugin/test.agent.md"));

    // Project scope
    let ctx_project = PlacementContext {
        component: ComponentRef::new(ComponentKind::Agent, "test"),
        origin: &origin,
        scope: PlacementScope(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location_project = target.placement_location(&ctx_project).unwrap();
    assert!(location_project.is_file());
    assert_eq!(
        location_project.as_path(),
        Path::new("/project/.github/agents/official/my-plugin/test.agent.md")
    );
}

#[test]
fn test_copilot_placement_location_command() {
    let target = CopilotTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    // Personal scope for commands is not supported
    let ctx_personal = PlacementContext {
        component: ComponentRef::new(ComponentKind::Command, "my-command"),
        origin: &origin,
        scope: PlacementScope(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    assert!(target.placement_location(&ctx_personal).is_none());

    // Project scope
    let ctx_project = PlacementContext {
        component: ComponentRef::new(ComponentKind::Command, "my-command"),
        origin: &origin,
        scope: PlacementScope(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx_project).unwrap();
    assert!(location.is_file());
    assert_eq!(
        location.as_path(),
        Path::new("/project/.github/prompts/official/my-plugin/my-command.prompt.md")
    );
}

#[test]
fn test_copilot_placement_location_instruction() {
    let target = CopilotTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Instruction, "test"),
        origin: &origin,
        scope: PlacementScope(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_file());
    assert_eq!(
        location.as_path(),
        Path::new("/project/.github/copilot-instructions.md")
    );
}
