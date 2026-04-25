use super::*;
use crate::component::{Component, ComponentKind, Scope};
use std::path::PathBuf;

fn sample_component() -> Component {
    Component {
        kind: ComponentKind::Skill,
        name: "demo".to_string(),
        path: PathBuf::from("/src/demo"),
    }
}

#[test]
fn test_builder_fails_without_component() {
    let err = ComponentDeploymentBuilder::default()
        .scope(Scope::Project)
        .target_path("/dest")
        .build()
        .unwrap_err();
    assert!(
        matches!(err, crate::error::PlmError::Validation(msg) if msg == "component is required")
    );
}

#[test]
fn test_builder_fails_without_scope() {
    let err = ComponentDeploymentBuilder::default()
        .component(sample_component())
        .target_path("/dest")
        .build()
        .unwrap_err();
    assert!(matches!(err, crate::error::PlmError::Validation(msg) if msg == "scope is required"));
}

#[test]
fn test_builder_fails_without_target_path() {
    let err = ComponentDeploymentBuilder::default()
        .component(sample_component())
        .scope(Scope::Project)
        .build()
        .unwrap_err();
    assert!(
        matches!(err, crate::error::PlmError::Validation(msg) if msg == "target_path is required")
    );
}

#[test]
fn test_builder_builds_with_required_fields_and_default_conversion() {
    let deployment = ComponentDeploymentBuilder::default()
        .component(sample_component())
        .scope(Scope::Project)
        .target_path("/dest/demo")
        .build()
        .unwrap();

    assert_eq!(deployment.kind(), ComponentKind::Skill);
    assert_eq!(deployment.name(), "demo");
    assert_eq!(deployment.scope, Scope::Project);
    assert!(matches!(deployment.conversion, ConversionConfig::None));
}
