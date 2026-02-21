use super::*;
use std::path::{Path, PathBuf};

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

#[test]
fn test_scoped_path_rejects_traversal_with_dotdot() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    // root/../outside_file はroot外に出るため拒否される
    let malicious_path = root.join("..").join("outside_file");
    let result = ScopedPath::new(malicious_path, root);
    assert!(
        result.is_err(),
        "Path with .. escaping root should be rejected"
    );
}

#[test]
fn test_scoped_path_allows_dotdot_within_root() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();
    let sub_dir = root.join("sub");
    std::fs::create_dir_all(&sub_dir).unwrap();

    // root/sub/../file はroot内に留まるため許可される
    let valid_path = sub_dir.join("..").join("file.txt");
    let result = ScopedPath::new(valid_path, root);
    assert!(
        result.is_ok(),
        "Path with .. staying within root should be accepted"
    );
}

#[cfg(unix)]
#[test]
fn test_scoped_path_rejects_symlink_escaping_root() {
    use std::os::unix::fs::symlink;

    let temp_dir = std::env::temp_dir().join("plm_test_symlink");
    let inside = temp_dir.join("inside");
    std::fs::create_dir_all(&inside).unwrap();

    // root外のディレクトリを作成
    let outside = std::env::temp_dir().join("plm_test_outside_target");
    std::fs::create_dir_all(&outside).unwrap();
    let outside_file = outside.join("secret.txt");
    std::fs::write(&outside_file, "secret").unwrap();

    // root内にroot外を指すシンボリックリンクを作成
    let link_path = inside.join("escape_link");
    symlink(&outside, &link_path).unwrap();

    // シンボリックリンク経由のパスはcanonicalizeでroot外と判定される
    let malicious_path = link_path.join("secret.txt");
    let result = ScopedPath::new(malicious_path, &temp_dir);
    assert!(
        result.is_err(),
        "Path through symlink escaping root should be rejected"
    );

    std::fs::remove_dir_all(&temp_dir).ok();
    std::fs::remove_dir_all(&outside).ok();
}

#[test]
fn test_normalize_path() {
    assert_eq!(
        normalize_path(Path::new("/a/b/../c")),
        PathBuf::from("/a/c")
    );
    assert_eq!(
        normalize_path(Path::new("/a/./b/c")),
        PathBuf::from("/a/b/c")
    );
    assert_eq!(
        normalize_path(Path::new("/a/b/../../c")),
        PathBuf::from("/c")
    );
}
