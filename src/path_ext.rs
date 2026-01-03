//! Path 拡張トレイト
//!
//! 標準ライブラリの `Path` に便利メソッドを追加する。

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
}
