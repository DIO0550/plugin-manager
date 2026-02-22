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

#[test]
fn test_scoped_path_stores_validated_path() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();
    let sub_dir = root.join("sub");
    std::fs::create_dir_all(&sub_dir).unwrap();

    // root/sub/../file.txt を渡すと、正規化された root/file.txt が保存される
    let input_path = sub_dir.join("..").join("file.txt");
    let scoped = ScopedPath::new(input_path.clone(), root).unwrap();
    let stored = scoped.as_path();

    // 保存されたパスに ".." が含まれていないことを確認（正規化済み）
    assert!(
        !stored
            .components()
            .any(|c| c == std::path::Component::ParentDir),
        "Stored path should not contain '..' components, got: {}",
        stored.display()
    );
    // 正規化後のパスがルート直下の file.txt を指すことを確認
    assert!(
        stored.ends_with("file.txt"),
        "Stored path should end with file.txt, got: {}",
        stored.display()
    );
}

#[cfg(unix)]
#[test]
fn test_scoped_path_rejects_symlink_escaping_root() {
    use std::os::unix::fs::symlink;

    let root_dir = tempfile::tempdir().unwrap();
    let inside = root_dir.path().join("inside");
    std::fs::create_dir_all(&inside).unwrap();

    // root外のディレクトリを作成
    let outside_dir = tempfile::tempdir().unwrap();
    let outside_file = outside_dir.path().join("secret.txt");
    std::fs::write(&outside_file, "secret").unwrap();

    // root内にroot外を指すシンボリックリンクを作成
    let link_path = inside.join("escape_link");
    symlink(outside_dir.path(), &link_path).unwrap();

    // シンボリックリンク経由のパスはcanonicalizeでroot外と判定される
    let malicious_path = link_path.join("secret.txt");
    let result = ScopedPath::new(malicious_path, root_dir.path());
    assert!(
        result.is_err(),
        "Path through symlink escaping root should be rejected"
    );
}

#[test]
fn test_normalize_path() {
    // 絶対パスの .. 正規化
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
    // ルートを越える .. は無視される
    assert_eq!(
        normalize_path(Path::new("/a/../../../c")),
        PathBuf::from("/c")
    );
    // 相対パスの先頭の .. は保持される
    assert_eq!(normalize_path(Path::new("../a/b")), PathBuf::from("../a/b"));
    assert_eq!(
        normalize_path(Path::new("../../a")),
        PathBuf::from("../../a")
    );
}
