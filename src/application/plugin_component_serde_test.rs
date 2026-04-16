use super::*;
use std::path::PathBuf;

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
