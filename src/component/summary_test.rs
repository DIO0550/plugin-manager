use super::*;

#[test]
fn test_component_type_count_title_skill() {
    let count = ComponentTypeCount {
        kind: ComponentKind::Skill,
        count: 3,
    };
    assert_eq!(count.title(), "Skills");
}

#[test]
fn test_component_type_count_title_agent() {
    let count = ComponentTypeCount {
        kind: ComponentKind::Agent,
        count: 2,
    };
    assert_eq!(count.title(), "Agents");
}

#[test]
fn test_component_type_count_title_command() {
    let count = ComponentTypeCount {
        kind: ComponentKind::Command,
        count: 1,
    };
    assert_eq!(count.title(), "Commands");
}

#[test]
fn test_component_type_count_title_instruction() {
    let count = ComponentTypeCount {
        kind: ComponentKind::Instruction,
        count: 5,
    };
    assert_eq!(count.title(), "Instructions");
}

#[test]
fn test_component_type_count_title_hook() {
    let count = ComponentTypeCount {
        kind: ComponentKind::Hook,
        count: 0,
    };
    assert_eq!(count.title(), "Hooks");
}

#[test]
fn test_component_type_count_all_titles_are_plural() {
    for kind in ComponentKind::all() {
        let count = ComponentTypeCount { kind: *kind, count: 1 };
        let title = count.title();
        assert!(title.ends_with('s'), "{:?} title should be plural", kind);
    }
}

#[test]
fn test_component_name_creation() {
    let name = ComponentName {
        name: "my-component".to_string(),
    };
    assert_eq!(name.name, "my-component");
}

#[test]
fn test_component_name_clone() {
    let name = ComponentName {
        name: "test".to_string(),
    };
    let cloned = name.clone();
    assert_eq!(name.name, cloned.name);
}
