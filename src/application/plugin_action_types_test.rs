use super::*;
use std::path::PathBuf;

#[test]
fn test_target_id() {
    let id = TargetId::new("codex");
    assert_eq!(id.as_str(), "codex");
    assert_eq!(format!("{}", id), "codex");

    let from_str: TargetId = "copilot".into();
    assert_eq!(from_str.as_str(), "copilot");
}

#[test]
fn test_scoped_path_under_project() {
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("test.txt");
    let result = ScopedPath::new(path.clone(), &temp_dir);
    assert!(result.is_ok());
}

#[test]
fn test_file_operation_kind() {
    let temp_dir = std::env::temp_dir();
    let scoped = ScopedPath::new(temp_dir.join("test.txt"), &temp_dir).unwrap();

    let copy = FileOperation::CopyFile {
        source: PathBuf::from("/src/file.txt"),
        target: scoped.clone(),
    };
    assert_eq!(copy.kind(), "copy_file");

    let remove = FileOperation::RemoveFile { path: scoped };
    assert_eq!(remove.kind(), "remove_file");
}
