use super::*;
use std::fs;
use std::os::unix::fs::symlink;
use tempfile::TempDir;

#[tokio::test]
async fn test_unlink_removes_symlink_and_preserves_target() {
    let tmp = TempDir::new().unwrap();
    let target_file = tmp.path().join("original.txt");
    fs::write(&target_file, "hello").unwrap();

    let link_path = tmp.path().join("link.txt");
    symlink(&target_file, &link_path).unwrap();

    let args = Args {
        path: link_path.clone(),
    };
    let result: Result<(), String> = run(args).await;
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);

    // Symlink should be gone
    assert!(!link_path.exists(), "Symlink should have been removed");
    // Original file should still exist
    assert!(target_file.exists(), "Original file should still exist");
}

#[tokio::test]
async fn test_unlink_path_not_found() {
    let tmp = TempDir::new().unwrap();
    let non_existent = tmp.path().join("does_not_exist");

    let args = Args { path: non_existent };
    let result: Result<(), String> = run(args).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("Path not found"),
        "Expected 'Path not found' in error, got: {}",
        err
    );
}

#[tokio::test]
async fn test_unlink_rejects_regular_file() {
    let tmp = TempDir::new().unwrap();
    let regular_file = tmp.path().join("regular.txt");
    fs::write(&regular_file, "content").unwrap();

    let args = Args { path: regular_file };
    let result: Result<(), String> = run(args).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("Not a symlink"),
        "Expected 'Not a symlink' in error, got: {}",
        err
    );
}

#[tokio::test]
async fn test_unlink_rejects_directory() {
    let tmp = TempDir::new().unwrap();
    let dir_path = tmp.path().join("mydir");
    fs::create_dir(&dir_path).unwrap();

    let args = Args { path: dir_path };
    let result: Result<(), String> = run(args).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("Not a symlink"),
        "Expected 'Not a symlink' in error, got: {}",
        err
    );
}

#[tokio::test]
async fn test_unlink_removes_broken_symlink() {
    let tmp = TempDir::new().unwrap();
    let target_file = tmp.path().join("will_be_deleted.txt");
    fs::write(&target_file, "temporary").unwrap();

    let link_path = tmp.path().join("broken_link.txt");
    symlink(&target_file, &link_path).unwrap();

    // Delete the target to make the symlink broken
    fs::remove_file(&target_file).unwrap();

    // Verify the symlink is indeed broken (target doesn't exist)
    assert!(
        !link_path.exists(),
        "Broken symlink target should not exist via exists()"
    );
    // But symlink_metadata should still find it
    assert!(
        fs::symlink_metadata(&link_path).is_ok(),
        "symlink_metadata should detect broken symlink"
    );

    let args = Args {
        path: link_path.clone(),
    };
    let result: Result<(), String> = run(args).await;
    assert!(
        result.is_ok(),
        "Expected Ok for broken symlink, got: {:?}",
        result
    );

    // Broken symlink should be removed
    assert!(
        fs::symlink_metadata(&link_path).is_err(),
        "Broken symlink should have been removed"
    );
}
