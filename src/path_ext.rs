//! Path 拡張トレイト
//!
//! 標準ライブラリの `Path` に便利メソッドを追加する。

use crate::error::Result;
use crate::fs::{FileSystem, RealFs};
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
    ///
    /// # Arguments
    ///
    /// * `custom` - Optional custom path segment that overrides `default`.
    /// * `default` - Fallback path segment used when `custom` is `None`.
    fn join_or(&self, custom: Option<&str>, default: &str) -> PathBuf;

    /// ディレクトリを再帰的にコピー
    ///
    /// # Arguments
    ///
    /// * `target` - Destination directory to copy into.
    fn copy_dir_to(&self, target: &Path) -> Result<()>;

    /// ファイルをコピー（親ディレクトリも作成）
    ///
    /// # Arguments
    ///
    /// * `target` - Destination file path. Parent directories are created as needed.
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
        RealFs.copy_dir_replace(self, target)
    }

    fn copy_file_to(&self, target: &Path) -> Result<()> {
        RealFs.copy_file(self, target)
    }
}

#[cfg(test)]
#[path = "path_ext_test.rs"]
mod tests;
