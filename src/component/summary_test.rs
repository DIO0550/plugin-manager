use super::*;
use std::path::PathBuf;

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
        let count = ComponentTypeCount {
            kind: *kind,
            count: 1,
        };
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

fn comp(kind: ComponentKind, name: &str) -> Component {
    Component {
        kind,
        name: name.to_string(),
        path: PathBuf::from(format!("dummy/{}", name)),
    }
}

#[test]
fn test_serialize_components_groups_by_kind() {
    let components = vec![
        comp(ComponentKind::Skill, "s1"),
        comp(ComponentKind::Agent, "a1"),
        comp(ComponentKind::Skill, "s2"),
    ];
    let json = serde_json::to_value(&SerHelper(&components)).unwrap();

    assert_eq!(json["skills"], serde_json::json!(["s1", "s2"]));
    assert_eq!(json["agents"], serde_json::json!(["a1"]));
    assert_eq!(json["commands"], serde_json::json!([]));
    assert_eq!(json["instructions"], serde_json::json!([]));
    assert_eq!(json["hooks"], serde_json::json!([]));
}

#[test]
fn test_serialize_components_empty() {
    let components: Vec<Component> = Vec::new();
    let json = serde_json::to_value(&SerHelper(&components)).unwrap();

    for key in ["skills", "agents", "commands", "instructions", "hooks"] {
        assert_eq!(json[key], serde_json::json!([]), "{} should be empty", key);
    }
}

struct SerHelper<'a>(&'a [Component]);
impl serde::Serialize for SerHelper<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_components(self.0, serializer)
    }
}
