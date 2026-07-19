use super::*;
use crate::component::{Component, ComponentKind, FileOperation};
use tempfile::TempDir;

#[test]
fn test_plugin_intent_expand_empty() {
    let intent = PluginIntent::new(
        PluginAction::Enable {
            plugin_name: "test-plugin".to_string(),
            marketplace: None,
        },
        vec![],
        std::env::temp_dir(),
    );

    let result = intent.expand();
    assert!(result.operations.is_empty());
    assert!(result.validation_errors.is_empty());
}

#[test]
fn test_plugin_intent_expand_adds_cursor_legacy_cleanup_operation() {
    let project_root = TempDir::new().unwrap();
    let source_root = TempDir::new().unwrap();
    let skill_dir = source_root.path().join("skills").join("review");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(skill_dir.join("SKILL.md"), "---\nname: review\n---\n").unwrap();

    let legacy = project_root
        .path()
        .join(".cursor")
        .join("skills")
        .join("test-plugin_review");
    std::fs::create_dir_all(&legacy).unwrap();
    std::fs::write(legacy.join("SKILL.md"), "# legacy").unwrap();

    let intent = PluginIntent::with_target_filter(
        PluginAction::Enable {
            plugin_name: "test-plugin".to_string(),
            marketplace: None,
        },
        vec![Component::flattened(
            ComponentKind::Skill,
            "test-plugin",
            "review",
            &skill_dir,
        )],
        project_root.path().to_path_buf(),
        Some("cursor"),
    );

    let result = intent.expand();

    assert!(
        result.validation_errors.is_empty(),
        "{:?}",
        result.validation_errors
    );
    assert_eq!(result.operations.len(), 2);
    assert!(result.operations.iter().any(|(kind, op)| {
        *kind == crate::target::TargetKind::Cursor
            && matches!(op, FileOperation::RemoveDir { path } if path.as_path() == legacy)
    }));
}

#[test]
fn test_plugin_intent_expand_skips_cursor_legacy_cleanup_when_legacy_dir_missing() {
    let project_root = TempDir::new().unwrap();
    let source_root = TempDir::new().unwrap();
    let skill_dir = source_root.path().join("skills").join("review");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(skill_dir.join("SKILL.md"), "---\nname: review\n---\n").unwrap();

    let intent = PluginIntent::with_target_filter(
        PluginAction::Enable {
            plugin_name: "test-plugin".to_string(),
            marketplace: None,
        },
        vec![Component::flattened(
            ComponentKind::Skill,
            "test-plugin",
            "review",
            &skill_dir,
        )],
        project_root.path().to_path_buf(),
        Some("cursor"),
    );

    let result = intent.expand();

    assert!(
        result.validation_errors.is_empty(),
        "{:?}",
        result.validation_errors
    );
    assert_eq!(result.operations.len(), 1);
    assert!(!result.operations.iter().any(|(_, op)| {
        matches!(op, FileOperation::RemoveDir { path } if path.as_path().ends_with("test-plugin_review"))
    }));
}
