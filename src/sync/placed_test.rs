use super::*;

#[test]
fn test_component_identity_eq() {
    let id1 = ComponentIdentity::new(ComponentKind::Skill, "test", Scope::Personal);
    let id2 = ComponentIdentity::new(ComponentKind::Skill, "test", Scope::Personal);
    let id3 = ComponentIdentity::new(ComponentKind::Skill, "test", Scope::Project);

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

#[test]
fn test_placed_component_accessors() {
    let comp = PlacedComponent::new(
        ComponentKind::Skill,
        "my-skill",
        Scope::Personal,
        "/path/to/skill",
    );

    assert_eq!(comp.kind(), ComponentKind::Skill);
    assert_eq!(comp.name(), "my-skill");
    assert_eq!(comp.scope(), Scope::Personal);
}
