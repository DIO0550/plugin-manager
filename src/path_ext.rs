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
#[path = "path_ext_test.rs"]
mod tests;
