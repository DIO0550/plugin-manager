use super::*;

#[test]
fn test_component_kind_as_str() {
    assert_eq!(ComponentKind::Skill.as_str(), "skill");
    assert_eq!(ComponentKind::Agent.as_str(), "agent");
    assert_eq!(ComponentKind::Command.as_str(), "command");
    assert_eq!(ComponentKind::Instruction.as_str(), "instruction");
    assert_eq!(ComponentKind::Hook.as_str(), "hook");
}

#[test]
fn test_component_kind_plural() {
    assert_eq!(ComponentKind::Skill.plural(), "skills");
    assert_eq!(ComponentKind::Agent.plural(), "agents");
}

#[test]
fn test_component_kind_all() {
    assert_eq!(ComponentKind::all().len(), 5);
}

#[test]
fn test_scope_as_str() {
    assert_eq!(Scope::Personal.as_str(), "personal");
    assert_eq!(Scope::Project.as_str(), "project");
}

#[test]
fn test_component_kind_all_elements_unique() {
    let all = ComponentKind::all();
    let mut seen = std::collections::HashSet::new();
    for kind in all {
        assert!(
            seen.insert(kind),
            "Duplicate ComponentKind found: {:?}",
            kind
        );
    }
}

#[test]
fn test_component_kind_as_str_not_empty() {
    for kind in ComponentKind::all() {
        assert!(!kind.as_str().is_empty(), "{:?}.as_str() is empty", kind);
    }
}

#[test]
fn test_component_kind_plural_not_empty() {
    for kind in ComponentKind::all() {
        assert!(!kind.plural().is_empty(), "{:?}.plural() is empty", kind);
    }
}

#[test]
fn test_component_kind_display_name_not_empty() {
    for kind in ComponentKind::all() {
        assert!(
            !kind.display_name().is_empty(),
            "{:?}.display_name() is empty",
            kind
        );
    }
}

#[test]
fn test_component_kind_as_str_unique() {
    let all = ComponentKind::all();
    let mut seen = std::collections::HashSet::new();
    for kind in all {
        let s = kind.as_str();
        assert!(seen.insert(s), "Duplicate as_str found: {}", s);
    }
}

#[test]
fn test_component_kind_plural_unique() {
    let all = ComponentKind::all();
    let mut seen = std::collections::HashSet::new();
    for kind in all {
        let s = kind.plural();
        assert!(seen.insert(s), "Duplicate plural found: {}", s);
    }
}

#[test]
fn test_scope_as_str_not_empty() {
    assert!(!Scope::Personal.as_str().is_empty());
    assert!(!Scope::Project.as_str().is_empty());
}

#[test]
fn test_scope_display_name_not_empty() {
    assert!(!Scope::Personal.display_name().is_empty());
    assert!(!Scope::Project.display_name().is_empty());
}

#[test]
fn test_scope_description_not_empty() {
    assert!(!Scope::Personal.description().is_empty());
    assert!(!Scope::Project.description().is_empty());
}

#[test]
fn test_flatten_name_basic() {
    assert_eq!(flatten_name("myplugin", "foo"), "myplugin_foo");
}

#[test]
fn test_flatten_name_original_with_underscore() {
    assert_eq!(flatten_name("myplugin", "foo_bar"), "myplugin_foo_bar");
}

#[test]
fn test_flatten_name_plugin_with_underscore() {
    assert_eq!(flatten_name("my_plugin", "foo"), "my_plugin_foo");
}

#[test]
fn test_flatten_name_empty_plugin_name() {
    assert_eq!(flatten_name("", "foo"), "_foo");
}

#[test]
fn test_component_flattened_uses_flatten_name() {
    let c = Component::flattened(
        ComponentKind::Skill,
        "myplugin",
        "foo",
        std::path::PathBuf::from("/skills/foo"),
    );
    assert_eq!(c.name, "myplugin_foo");
    assert_eq!(c.original_name.as_deref(), Some("foo"));
    assert_eq!(c.plugin_name, "myplugin");
}

#[test]
fn test_component_new_leaves_original_name_unset() {
    let c = Component::new(
        ComponentKind::Instruction,
        "AGENTS",
        std::path::PathBuf::from("/AGENTS.md"),
    );
    assert!(c.original_name.is_none());
    assert!(c.plugin_name.is_empty());
}
