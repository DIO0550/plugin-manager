use super::*;
use std::path::PathBuf;

#[test]
fn test_file_operation_copy_file_constructed() {
    let temp_dir = std::env::temp_dir();
    let scoped = ScopedPath::new(temp_dir.join("test.txt"), &temp_dir).unwrap();

    let op = FileOperation::CopyFile {
        source: PathBuf::from("/src/file.txt"),
        target: scoped,
    };
    assert!(matches!(op, FileOperation::CopyFile { .. }));
}
