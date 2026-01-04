//! Path 拡張トレイト
//!
//! 標準ライブラリの `Path` に便利メソッドを追加する。

use crate::error::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// Path の拡張トレイト
pub trait PathExt {
    /// ディレクトリのエントリを読み取り、パスのリストを返す
    ///
    /// ディレクトリが存在しない場合や読み取りエラーの場合は空のベクタを返す。
    fn read_dir_entries(&self) -> Vec<PathBuf>;

    /// カスタムパスまたはデフォルトパスを結合する
    ///
    /// `custom` が `Some` の場合はそのパスを、`None` の場合は `default` を
    /// ベースパスに結合して返す。
    fn join_or(&self, custom: Option<&str>, default: &str) -> PathBuf;

    /// ディレクトリを再帰的にコピー
    fn copy_dir_to(&self, target: &Path) -> Result<()>;

    /// ファイルをコピー（親ディレクトリも作成）
    fn copy_file_to(&self, target: &Path) -> Result<()>;
}

impl PathExt for Path {
    fn read_dir_entries(&self) -> Vec<PathBuf> {
        std::fs::read_dir(self)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .collect()
    }

    fn join_or(&self, custom: Option<&str>, default: &str) -> PathBuf {
        custom
            .map(|p| self.join(p))
            .unwrap_or_else(|| self.join(default))
    }

    fn copy_dir_to(&self, target: &Path) -> Result<()> {
        if target.exists() {
            fs::remove_dir_all(target)?;
        }
        fs::create_dir_all(target)?;

        for entry in fs::read_dir(self)? {
            let entry = entry?;
            let source_path = entry.path();
            let target_path = target.join(entry.file_name());

            if source_path.is_dir() {
                source_path.copy_dir_to(&target_path)?;
            } else {
                fs::copy(&source_path, &target_path)?;
            }
        }

        Ok(())
    }

    fn copy_file_to(&self, target: &Path) -> Result<()> {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(self, target)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ========================================
    // read_dir_entries tests
    // ========================================

    #[test]
    fn test_read_dir_entries_returns_paths() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("a.txt"), "").unwrap();
        fs::write(temp.path().join("b.txt"), "").unwrap();
        fs::create_dir(temp.path().join("subdir")).unwrap();

        let entries = temp.path().read_dir_entries();

        assert_eq!(entries.len(), 3);
    }

    #[test]
    fn test_read_dir_entries_empty_dir() {
        let temp = TempDir::new().unwrap();

        let entries = temp.path().read_dir_entries();

        assert!(entries.is_empty());
    }

    #[test]
    fn test_read_dir_entries_nonexistent_returns_empty() {
        let path = Path::new("/nonexistent/path/that/does/not/exist");

        let entries = path.read_dir_entries();

        assert!(entries.is_empty());
    }

    // ========================================
    // join_or tests
    // ========================================

    #[test]
    fn test_join_or_with_custom_path() {
        let base = Path::new("/base");

        let result = base.join_or(Some("custom"), "default");

        assert_eq!(result, PathBuf::from("/base/custom"));
    }

    #[test]
    fn test_join_or_with_none_uses_default() {
        let base = Path::new("/base");

        let result = base.join_or(None, "default");

        assert_eq!(result, PathBuf::from("/base/default"));
    }

    #[test]
    fn test_join_or_with_nested_path() {
        let base = Path::new("/base");

        let result = base.join_or(Some("a/b/c"), "default");

        assert_eq!(result, PathBuf::from("/base/a/b/c"));
    }

    // ========================================
    // copy_file_to tests
    // ========================================

    #[test]
    fn test_copy_file_to_creates_parent_dirs() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source.txt");
        let target = temp.path().join("a/b/c/target.txt");

        fs::write(&source, "hello").unwrap();

        source.copy_file_to(&target).unwrap();

        assert!(target.exists());
        assert_eq!(fs::read_to_string(&target).unwrap(), "hello");
    }

    #[test]
    fn test_copy_file_to_overwrites_existing() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source.txt");
        let target = temp.path().join("target.txt");

        fs::write(&source, "new content").unwrap();
        fs::write(&target, "old content").unwrap();

        source.copy_file_to(&target).unwrap();

        assert_eq!(fs::read_to_string(&target).unwrap(), "new content");
    }

    #[test]
    fn test_copy_file_to_nonexistent_source_fails() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("nonexistent.txt");
        let target = temp.path().join("target.txt");

        let result = source.copy_file_to(&target);

        assert!(result.is_err());
    }

    #[test]
    fn test_copy_dir_to_copies_files() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let target = temp.path().join("target");

        fs::create_dir(&source).unwrap();
        fs::write(source.join("file1.txt"), "content1").unwrap();
        fs::write(source.join("file2.txt"), "content2").unwrap();

        source.copy_dir_to(&target).unwrap();

        assert!(target.exists());
        assert_eq!(fs::read_to_string(target.join("file1.txt")).unwrap(), "content1");
        assert_eq!(fs::read_to_string(target.join("file2.txt")).unwrap(), "content2");
    }

    #[test]
    fn test_copy_dir_to_copies_nested_dirs() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let target = temp.path().join("target");

        fs::create_dir_all(source.join("a/b")).unwrap();
        fs::write(source.join("root.txt"), "root").unwrap();
        fs::write(source.join("a/nested.txt"), "nested").unwrap();
        fs::write(source.join("a/b/deep.txt"), "deep").unwrap();

        source.copy_dir_to(&target).unwrap();

        assert_eq!(fs::read_to_string(target.join("root.txt")).unwrap(), "root");
        assert_eq!(fs::read_to_string(target.join("a/nested.txt")).unwrap(), "nested");
        assert_eq!(fs::read_to_string(target.join("a/b/deep.txt")).unwrap(), "deep");
    }

    #[test]
    fn test_copy_dir_to_replaces_existing_target() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let target = temp.path().join("target");

        // Create source with one file
        fs::create_dir(&source).unwrap();
        fs::write(source.join("new.txt"), "new").unwrap();

        // Create target with different file
        fs::create_dir(&target).unwrap();
        fs::write(target.join("old.txt"), "old").unwrap();

        source.copy_dir_to(&target).unwrap();

        // Old file should be gone, new file should exist
        assert!(!target.join("old.txt").exists());
        assert!(target.join("new.txt").exists());
    }

    #[test]
    fn test_copy_dir_to_creates_target_parent_dirs() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let target = temp.path().join("a/b/c/target");

        fs::create_dir(&source).unwrap();
        fs::write(source.join("file.txt"), "content").unwrap();

        source.copy_dir_to(&target).unwrap();

        assert!(target.exists());
        assert_eq!(fs::read_to_string(target.join("file.txt")).unwrap(), "content");
    }

    #[test]
    fn test_copy_dir_to_nonexistent_source_fails() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("nonexistent");
        let target = temp.path().join("target");

        let result = source.copy_dir_to(&target);

        assert!(result.is_err());
    }

    #[test]
    fn test_copy_dir_to_empty_dir() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let target = temp.path().join("target");

        fs::create_dir(&source).unwrap();

        source.copy_dir_to(&target).unwrap();

        assert!(target.exists());
        assert!(target.is_dir());
        assert_eq!(fs::read_dir(&target).unwrap().count(), 0);
    }
}
