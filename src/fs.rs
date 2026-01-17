//! ファイルシステム抽象化
//!
//! プロジェクト全体で使用するファイルシステム操作の抽象化レイヤー。
//! テスト時に MockFs を注入してファイル操作をモック化できる。

use crate::error::Result;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// ファイル種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsFileType {
    File,
    Dir,
    Symlink,
}

/// ファイルシステム抽象化のための独自 DirEntry
#[derive(Debug, Clone)]
pub struct FsDirEntry {
    pub path: PathBuf,
    pub file_type: FsFileType,
}

impl FsDirEntry {
    /// ディレクトリかどうか
    pub fn is_dir(&self) -> bool {
        self.file_type == FsFileType::Dir
    }

    /// ファイルかどうか
    pub fn is_file(&self) -> bool {
        self.file_type == FsFileType::File
    }

    /// シンボリックリンクかどうか
    pub fn is_symlink(&self) -> bool {
        self.file_type == FsFileType::Symlink
    }
}

/// ファイルシステム操作を抽象化するトレイト
///
/// テスト時に MockFs を注入してファイル操作をモック化できる。
/// 本番コードでは RealFs を使用する。
pub trait FileSystem: Send + Sync {
    /// ファイルをコピー
    ///
    /// - 宛先が存在すれば上書き
    /// - 親ディレクトリは自動作成
    /// - シンボリックリンクは追従（実体をコピー）
    fn copy_file(&self, src: &Path, dst: &Path) -> Result<()>;

    /// ディレクトリを再帰的にコピー
    ///
    /// - 宛先ディレクトリにマージ（既存ファイルは上書き）
    /// - シンボリックリンクは追従
    /// - 同一/子孫パスへのコピーは Err
    fn copy_dir(&self, src: &Path, dst: &Path) -> Result<()>;

    /// ファイルまたはディレクトリを削除
    ///
    /// - ファイルなら削除、ディレクトリなら再帰削除
    /// - 存在しない場合は Ok(())
    fn remove(&self, path: &Path) -> Result<()>;

    /// ファイルのみを削除
    ///
    /// - 存在しない場合は Ok(())
    /// - ディレクトリの場合は Err
    fn remove_file(&self, path: &Path) -> Result<()>;

    /// ディレクトリを再帰削除
    ///
    /// - 存在しない場合は Ok(())
    fn remove_dir_all(&self, path: &Path) -> Result<()>;

    /// ファイルまたはディレクトリを移動（リネーム）
    ///
    /// - 同一ファイルシステム内でのリネーム
    /// - クロスデバイス時は Err（呼び出し側で copy+remove 対応）
    fn rename(&self, src: &Path, dst: &Path) -> Result<()>;

    /// パスが存在するか（シンボリックリンク追従）
    fn exists(&self, path: &Path) -> bool;

    /// ディレクトリかどうか（シンボリックリンク追従）
    fn is_dir(&self, path: &Path) -> bool;

    /// ディレクトリを再帰的に作成
    fn create_dir_all(&self, path: &Path) -> Result<()>;

    /// 最終更新時刻を取得
    fn mtime(&self, path: &Path) -> Result<SystemTime>;

    /// ファイル内容のハッシュを計算（DefaultHasher 使用）
    fn content_hash(&self, path: &Path) -> Result<u64>;

    /// ファイル内容を読み込み
    fn read_to_string(&self, path: &Path) -> Result<String>;

    /// ファイルに書き込み
    ///
    /// - 親ディレクトリは自動作成
    /// - 既存ファイルは上書き
    /// - アトミック性は保証しない
    fn write(&self, path: &Path, content: &[u8]) -> Result<()>;

    /// ディレクトリ内のエントリを取得
    ///
    /// - FsDirEntry のベクタを返す
    /// - 順序は未定義
    /// - symlink_metadata を使用（シンボリックリンク非追従）
    /// - 引数がディレクトリでない場合は Err
    fn read_dir(&self, path: &Path) -> Result<Vec<FsDirEntry>>;
}

/// 本番用ファイルシステム実装
pub struct RealFs;

impl FileSystem for RealFs {
    fn copy_file(&self, src: &Path, dst: &Path) -> Result<()> {
        // 親ディレクトリを作成
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(src, dst)?;
        Ok(())
    }

    fn copy_dir(&self, src: &Path, dst: &Path) -> Result<()> {
        // 同一/子孫パスチェック
        if let (Ok(src_canonical), Ok(dst_canonical)) = (src.canonicalize(), dst.canonicalize()) {
            if dst_canonical.starts_with(&src_canonical) {
                return Err(crate::error::PlmError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Cannot copy directory into itself or its subdirectory",
                )));
            }
        }
        copy_dir_recursive(src, dst)
    }

    fn remove(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }
        if path.is_dir() {
            std::fs::remove_dir_all(path)?;
        } else {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }
        if path.is_dir() {
            return Err(crate::error::PlmError::Io(std::io::Error::new(
                std::io::ErrorKind::IsADirectory,
                "Cannot remove directory with remove_file",
            )));
        }
        std::fs::remove_file(path)?;
        Ok(())
    }

    fn remove_dir_all(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    fn rename(&self, src: &Path, dst: &Path) -> Result<()> {
        std::fs::rename(src, dst)?;
        Ok(())
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        std::fs::create_dir_all(path)?;
        Ok(())
    }

    fn mtime(&self, path: &Path) -> Result<SystemTime> {
        let metadata = std::fs::metadata(path)?;
        Ok(metadata.modified()?)
    }

    fn content_hash(&self, path: &Path) -> Result<u64> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;
        use std::io::Read;

        let mut file = std::fs::File::open(path)?;
        let mut hasher = DefaultHasher::new();
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.write(&buffer[..bytes_read]);
        }

        Ok(hasher.finish())
    }

    fn read_to_string(&self, path: &Path) -> Result<String> {
        Ok(std::fs::read_to_string(path)?)
    }

    fn write(&self, path: &Path, content: &[u8]) -> Result<()> {
        // 親ディレクトリを作成
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<FsDirEntry>> {
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.path().symlink_metadata()?;
            let file_type = if metadata.is_symlink() {
                FsFileType::Symlink
            } else if metadata.is_dir() {
                FsFileType::Dir
            } else {
                FsFileType::File
            };
            entries.push(FsDirEntry {
                path: entry.path(),
                file_type,
            });
        }
        Ok(entries)
    }
}

/// ディレクトリを再帰的にコピー
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

#[cfg(test)]
pub mod mock;

#[cfg(test)]
#[path = "fs_test.rs"]
mod tests;
