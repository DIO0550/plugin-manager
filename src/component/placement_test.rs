use super::*;

#[test]
fn test_component_ref() {
    let component = ComponentIdentity::new(ComponentKind::Skill, "my-skill");
    assert_eq!(component.kind, ComponentKind::Skill);
    assert_eq!(component.name, "my-skill");
}

#[test]
fn test_placement_scope() {
    let personal = PlacementScope::new(Scope::Personal);
    assert_eq!(personal.scope(), Scope::Personal);

    let project = PlacementScope::new(Scope::Project);
    assert_eq!(project.scope(), Scope::Project);
}

#[test]
fn test_placement_location() {
    let file = PlacementLocation::file("/path/to/file.md");
    assert!(file.is_file());
    assert!(!file.is_dir());
    assert_eq!(file.as_path(), Path::new("/path/to/file.md"));

    let dir = PlacementLocation::dir("/path/to/dir");
    assert!(dir.is_dir());
    assert!(!dir.is_file());
    assert_eq!(dir.as_path(), Path::new("/path/to/dir"));
}

#[test]
fn test_placement_context() {
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");
    let project_root = Path::new("/project");

    let ctx = PlacementContext {
        component: ComponentIdentity::new(ComponentKind::Skill, "my-skill"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };

    assert_eq!(ctx.kind(), ComponentKind::Skill);
    assert_eq!(ctx.name(), "my-skill");
    assert_eq!(ctx.scope(), Scope::Project);
    assert_eq!(ctx.project_root(), project_root);
}
