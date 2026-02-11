//! plm link コマンドのテスト

use super::*;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use tempfile::TempDir;

// ---------------------------------------------------------------
// relative_path_from tests
// ---------------------------------------------------------------

mod relative_path_from_tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn same_directory() {
        // src=/a/b.md, dest_parent=/a -> b.md
        let result = relative_path_from(Path::new("/a/b.md"), Path::new("/a"));
        assert_eq!(result, PathBuf::from("b.md"));
    }

    #[test]
    fn child_directory() {
        // src=/a/CLAUDE.md, dest_parent=/a/.github -> ../CLAUDE.md
        let result = relative_path_from(Path::new("/a/CLAUDE.md"), Path::new("/a/.github"));
        assert_eq!(result, PathBuf::from("../CLAUDE.md"));
    }

    #[test]
    fn deep_nesting() {
        // src=/a/.codex/skills/mp/plugin/skill
        // dest_parent=/a/.github/skills/mp/plugin
        // -> ../../../../.codex/skills/mp/plugin/skill
        let result = relative_path_from(
            Path::new("/a/.codex/skills/mp/plugin/skill"),
            Path::new("/a/.github/skills/mp/plugin"),
        );
        assert_eq!(
            result,
            PathBuf::from("../../../../.codex/skills/mp/plugin/skill")
        );
    }

    #[test]
    fn root_common_prefix() {
        // src=/foo/bar, dest_parent=/baz/qux -> ../../foo/bar
        let result = relative_path_from(Path::new("/foo/bar"), Path::new("/baz/qux"));
        assert_eq!(result, PathBuf::from("../../foo/bar"));
    }

    #[test]
    fn dest_is_subdirectory_of_src_parent() {
        // src=/a/b/c/file.txt, dest_parent=/a/b -> c/file.txt
        let result = relative_path_from(Path::new("/a/b/c/file.txt"), Path::new("/a/b"));
        assert_eq!(result, PathBuf::from("c/file.txt"));
    }
}

// ---------------------------------------------------------------
// absolutize tests
// ---------------------------------------------------------------

mod absolutize_tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn absolute_path_unchanged() {
        let result = absolutize(Path::new("/foo/bar/baz"));
        assert_eq!(result, PathBuf::from("/foo/bar/baz"));
    }

    #[test]
    fn removes_dot_components() {
        let result = absolutize(Path::new("/foo/./bar/./baz"));
        assert_eq!(result, PathBuf::from("/foo/bar/baz"));
    }

    #[test]
    fn resolves_parent_components() {
        let result = absolutize(Path::new("/foo/bar/../baz"));
        assert_eq!(result, PathBuf::from("/foo/baz"));
    }

    #[test]
    fn resolves_multiple_parent_components() {
        let result = absolutize(Path::new("/a/b/c/../../d"));
        assert_eq!(result, PathBuf::from("/a/d"));
    }

    #[test]
    fn relative_path_becomes_absolute() {
        let result = absolutize(Path::new("relative/path"));
        assert!(result.is_absolute());
        assert!(result.ends_with("relative/path"));
    }
}

// ---------------------------------------------------------------
// run() integration tests
// ---------------------------------------------------------------

mod run_tests {
    use super::*;

    fn make_args(src: PathBuf, dest: PathBuf, force: bool) -> Args {
        Args { src, dest, force }
    }

    #[tokio::test]
    async fn link_file_creates_symlink() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("source.txt");
        fs::write(&src, "hello").unwrap();

        let dest = tmp.path().join("link.txt");

        let args = make_args(src.clone(), dest.clone(), false);
        let result: Result<(), String> = run(args).await;
        result.unwrap();

        assert!(dest.symlink_metadata().unwrap().file_type().is_symlink());
        assert_eq!(fs::read_to_string(&dest).unwrap(), "hello");
    }

    #[tokio::test]
    async fn link_directory_creates_symlink() {
        let tmp = TempDir::new().unwrap();
        let src_dir = tmp.path().join("src_dir");
        fs::create_dir(&src_dir).unwrap();
        fs::write(src_dir.join("file.txt"), "content").unwrap();

        let dest = tmp.path().join("link_dir");

        let args = make_args(src_dir.clone(), dest.clone(), false);
        let result: Result<(), String> = run(args).await;
        result.unwrap();

        assert!(dest.symlink_metadata().unwrap().file_type().is_symlink());
        assert_eq!(
            fs::read_to_string(dest.join("file.txt")).unwrap(),
            "content"
        );
    }

    #[tokio::test]
    async fn error_when_src_not_found() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("nonexistent");
        let dest = tmp.path().join("link");

        let args = make_args(src, dest, false);
        let result: Result<(), String> = run(args).await;
        let err = result.unwrap_err();
        assert!(
            err.contains("Source not found"),
            "Expected 'Source not found' error, got: {}",
            err
        );
    }

    #[tokio::test]
    async fn error_when_dest_exists_without_force() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("source.txt");
        fs::write(&src, "hello").unwrap();

        let dest = tmp.path().join("existing.txt");
        fs::write(&dest, "old").unwrap();

        let args = make_args(src, dest, false);
        let result: Result<(), String> = run(args).await;
        let err = result.unwrap_err();
        assert!(
            err.contains("Dest already exists"),
            "Expected 'Dest already exists' error, got: {}",
            err
        );
        assert!(err.contains("--force"));
    }

    #[tokio::test]
    async fn force_overwrites_existing_file() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("source.txt");
        fs::write(&src, "new content").unwrap();

        let dest = tmp.path().join("existing.txt");
        fs::write(&dest, "old content").unwrap();

        let args = make_args(src, dest.clone(), true);
        let result: Result<(), String> = run(args).await;
        result.unwrap();

        assert!(dest.symlink_metadata().unwrap().file_type().is_symlink());
        assert_eq!(fs::read_to_string(&dest).unwrap(), "new content");
    }

    #[tokio::test]
    async fn force_overwrites_existing_symlink() {
        let tmp = TempDir::new().unwrap();
        let old_target = tmp.path().join("old_target.txt");
        fs::write(&old_target, "old").unwrap();

        let new_target = tmp.path().join("new_target.txt");
        fs::write(&new_target, "new").unwrap();

        let dest = tmp.path().join("link.txt");
        symlink(&old_target, &dest).unwrap();

        let args = make_args(new_target, dest.clone(), true);
        let result: Result<(), String> = run(args).await;
        result.unwrap();

        assert!(dest.symlink_metadata().unwrap().file_type().is_symlink());
        assert_eq!(fs::read_to_string(&dest).unwrap(), "new");
    }

    #[tokio::test]
    async fn force_errors_on_non_empty_directory() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("source.txt");
        fs::write(&src, "hello").unwrap();

        let dest = tmp.path().join("non_empty_dir");
        fs::create_dir(&dest).unwrap();
        fs::write(dest.join("child.txt"), "data").unwrap();

        let args = make_args(src, dest, true);
        let result: Result<(), String> = run(args).await;
        let err = result.unwrap_err();
        assert!(
            err.contains("non-empty directory"),
            "Expected 'non-empty directory' error, got: {}",
            err
        );
    }

    #[tokio::test]
    async fn force_removes_empty_directory() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("source.txt");
        fs::write(&src, "hello").unwrap();

        let dest = tmp.path().join("empty_dir");
        fs::create_dir(&dest).unwrap();

        let args = make_args(src, dest.clone(), true);
        let result: Result<(), String> = run(args).await;
        result.unwrap();

        assert!(dest.symlink_metadata().unwrap().file_type().is_symlink());
    }

    #[tokio::test]
    async fn creates_parent_directories_for_dest() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("source.txt");
        fs::write(&src, "hello").unwrap();

        let dest = tmp.path().join("a").join("b").join("c").join("link.txt");

        let args = make_args(src, dest.clone(), false);
        let result: Result<(), String> = run(args).await;
        result.unwrap();

        assert!(dest.symlink_metadata().unwrap().file_type().is_symlink());
        assert_eq!(fs::read_to_string(&dest).unwrap(), "hello");
    }

    #[tokio::test]
    async fn error_when_same_path_absolute() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("file.txt");
        fs::write(&file, "data").unwrap();

        let args = make_args(file.clone(), file.clone(), false);
        let result: Result<(), String> = run(args).await;
        let err = result.unwrap_err();
        assert!(
            err.contains("same path"),
            "Expected 'same path' error, got: {}",
            err
        );
    }

    #[tokio::test]
    async fn error_when_same_path_via_dot_components() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("file.txt");
        fs::write(&file, "data").unwrap();

        // Use ./file.txt and ../<dirname>/file.txt to refer to same path
        let dir_name = tmp.path().file_name().unwrap().to_owned();
        let parent = tmp.path().parent().unwrap();
        let dest_via_dotdot = parent.join(dir_name).join("./file.txt");

        let args = make_args(file, dest_via_dotdot, false);
        let result: Result<(), String> = run(args).await;
        let err = result.unwrap_err();
        assert!(
            err.contains("same path"),
            "Expected 'same path' error, got: {}",
            err
        );
    }

    #[tokio::test]
    async fn symlink_uses_relative_path() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("source.txt");
        fs::write(&src, "hello").unwrap();

        let sub = tmp.path().join("subdir");
        fs::create_dir(&sub).unwrap();
        let dest = sub.join("link.txt");

        let args = make_args(src, dest.clone(), false);
        let result: Result<(), String> = run(args).await;
        result.unwrap();

        // Read the raw symlink target - it should be relative, not absolute
        let link_target = fs::read_link(&dest).unwrap();
        assert!(
            link_target.is_relative(),
            "Symlink target should be relative, got: {}",
            link_target.display()
        );
        assert_eq!(link_target, PathBuf::from("../source.txt"));
    }
}
