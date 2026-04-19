use super::*;
use std::path::PathBuf;

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
