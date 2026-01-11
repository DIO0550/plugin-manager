use super::*;

#[test]
fn test_plugin_action_kind() {
    let install = PluginAction::Install {
        plugin_name: "test".to_string(),
        marketplace: None,
    };
    assert_eq!(install.kind(), "install");
    assert!(install.is_deploy());
    assert!(!install.is_remove());

    let disable = PluginAction::Disable {
        plugin_name: "test".to_string(),
        marketplace: Some("official".to_string()),
    };
    assert_eq!(disable.kind(), "disable");
    assert!(!disable.is_deploy());
    assert!(disable.is_remove());
}

#[test]
fn test_target_id() {
    let id = TargetId::new("codex");
    assert_eq!(id.as_str(), "codex");
    assert_eq!(format!("{}", id), "codex");

    let from_str: TargetId = "copilot".into();
    assert_eq!(from_str.as_str(), "copilot");
}

#[test]
fn test_validated_path_under_project() {
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("test.txt");
    let result = ValidatedPath::new(path.clone(), &temp_dir);
    assert!(result.is_ok());
}

#[test]
fn test_file_operation_kind() {
    let temp_dir = std::env::temp_dir();
    let validated = ValidatedPath::new(temp_dir.join("test.txt"), &temp_dir).unwrap();

    let copy = FileOperation::CopyFile {
        source: PathBuf::from("/src/file.txt"),
        target: validated.clone(),
    };
    assert_eq!(copy.kind(), "copy_file");

    let remove = FileOperation::RemoveFile { path: validated };
    assert_eq!(remove.kind(), "remove_file");
}

#[test]
fn test_plugin_plan_expand_empty() {
    let plan = PluginIntent::new(
        PluginAction::Enable {
            plugin_name: "test-plugin".to_string(),
            marketplace: None,
        },
        vec![],
        std::env::temp_dir(),
    );

    let ops = plan.expand();
    assert!(ops.is_empty());
}
