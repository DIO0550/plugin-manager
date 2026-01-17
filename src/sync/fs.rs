//! ファイルシステム抽象化

use crate::error::Result;
use std::path::Path;
use std::time::SystemTime;

/// ファイルシステム操作を抽象化するトレイト
///
/// テスト時に MockFs を注入してファイル操作をモック化できる
pub trait FileSystem: Send + Sync {
    /// ファイルをコピー
    fn copy_file(&self, src: &Path, dst: &Path) -> Result<()>;

    /// ディレクトリを再帰的にコピー
    fn copy_dir(&self, src: &Path, dst: &Path) -> Result<()>;

    /// ファイルまたはディレクトリを削除
    fn remove(&self, path: &Path) -> Result<()>;

    /// ファイルまたはディレクトリを移動（リネーム）
    fn rename(&self, src: &Path, dst: &Path) -> Result<()>;

    /// パスが存在するか
    fn exists(&self, path: &Path) -> bool;

    /// ディレクトリかどうか
    fn is_dir(&self, path: &Path) -> bool;

    /// ディレクトリを再帰的に作成
    fn create_dir_all(&self, path: &Path) -> Result<()>;

    /// 最終更新時刻を取得
    fn mtime(&self, path: &Path) -> Result<SystemTime>;

    /// ファイル内容のハッシュを計算
    fn content_hash(&self, path: &Path) -> Result<u64>;

    /// ファイル内容を読み込み
    fn read_to_string(&self, path: &Path) -> Result<String>;
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
        copy_dir_recursive(src, dst)
    }

    fn remove(&self, path: &Path) -> Result<()> {
        if path.is_dir() {
            std::fs::remove_dir_all(path)?;
        } else {
            std::fs::remove_file(path)?;
        }
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
pub mod mock {
    use super::*;
    use std::collections::HashMap;
    use std::sync::RwLock;

    /// テスト用モックファイルシステム
    pub struct MockFs {
        files: RwLock<HashMap<String, MockFile>>,
    }

    struct MockFile {
        content: String,
        mtime: SystemTime,
        is_dir: bool,
    }

    impl MockFs {
        pub fn new() -> Self {
            Self {
                files: RwLock::new(HashMap::new()),
            }
        }

        /// ファイルを追加
        pub fn add_file(&self, path: &str, content: &str) {
            self.files.write().unwrap().insert(
                path.to_string(),
                MockFile {
                    content: content.to_string(),
                    mtime: SystemTime::now(),
                    is_dir: false,
                },
            );
        }

        /// ディレクトリを追加
        pub fn add_dir(&self, path: &str) {
            self.files.write().unwrap().insert(
                path.to_string(),
                MockFile {
                    content: String::new(),
                    mtime: SystemTime::now(),
                    is_dir: true,
                },
            );
        }
    }

    impl Default for MockFs {
        fn default() -> Self {
            Self::new()
        }
    }

    impl FileSystem for MockFs {
        fn copy_file(&self, src: &Path, dst: &Path) -> Result<()> {
            let files = self.files.read().unwrap();
            let src_file = files
                .get(src.to_string_lossy().as_ref())
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "not found"))?;
            let content = src_file.content.clone();
            drop(files);

            self.files.write().unwrap().insert(
                dst.to_string_lossy().to_string(),
                MockFile {
                    content,
                    mtime: SystemTime::now(),
                    is_dir: false,
                },
            );
            Ok(())
        }

        fn copy_dir(&self, _src: &Path, _dst: &Path) -> Result<()> {
            // 簡易実装
            Ok(())
        }

        fn remove(&self, path: &Path) -> Result<()> {
            self.files
                .write()
                .unwrap()
                .remove(path.to_string_lossy().as_ref());
            Ok(())
        }

        fn rename(&self, src: &Path, dst: &Path) -> Result<()> {
            let mut files = self.files.write().unwrap();
            if let Some(file) = files.remove(src.to_string_lossy().as_ref()) {
                files.insert(dst.to_string_lossy().to_string(), file);
            }
            Ok(())
        }

        fn exists(&self, path: &Path) -> bool {
            self.files
                .read()
                .unwrap()
                .contains_key(path.to_string_lossy().as_ref())
        }

        fn is_dir(&self, path: &Path) -> bool {
            self.files
                .read()
                .unwrap()
                .get(path.to_string_lossy().as_ref())
                .map(|f| f.is_dir)
                .unwrap_or(false)
        }

        fn create_dir_all(&self, path: &Path) -> Result<()> {
            self.add_dir(&path.to_string_lossy());
            Ok(())
        }

        fn mtime(&self, path: &Path) -> Result<SystemTime> {
            self.files
                .read()
                .unwrap()
                .get(path.to_string_lossy().as_ref())
                .map(|f| f.mtime)
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::NotFound, "not found").into()
                })
        }

        fn content_hash(&self, path: &Path) -> Result<u64> {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let files = self.files.read().unwrap();
            let file = files
                .get(path.to_string_lossy().as_ref())
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "not found"))?;

            let mut hasher = DefaultHasher::new();
            file.content.hash(&mut hasher);
            Ok(hasher.finish())
        }

        fn read_to_string(&self, path: &Path) -> Result<String> {
            self.files
                .read()
                .unwrap()
                .get(path.to_string_lossy().as_ref())
                .map(|f| f.content.clone())
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::NotFound, "not found").into()
                })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::MockFs;
    use super::*;

    #[test]
    fn test_mock_fs_file_operations() {
        let fs = MockFs::new();

        // ファイル追加
        fs.add_file("/test.txt", "hello");
        assert!(fs.exists(Path::new("/test.txt")));
        assert!(!fs.is_dir(Path::new("/test.txt")));

        // 内容読み込み
        let content = fs.read_to_string(Path::new("/test.txt")).unwrap();
        assert_eq!(content, "hello");

        // コピー
        fs.copy_file(Path::new("/test.txt"), Path::new("/copy.txt"))
            .unwrap();
        assert!(fs.exists(Path::new("/copy.txt")));

        // 削除
        fs.remove(Path::new("/test.txt")).unwrap();
        assert!(!fs.exists(Path::new("/test.txt")));
    }

    #[test]
    fn test_mock_fs_content_hash() {
        let fs = MockFs::new();

        fs.add_file("/a.txt", "same content");
        fs.add_file("/b.txt", "same content");
        fs.add_file("/c.txt", "different");

        let hash_a = fs.content_hash(Path::new("/a.txt")).unwrap();
        let hash_b = fs.content_hash(Path::new("/b.txt")).unwrap();
        let hash_c = fs.content_hash(Path::new("/c.txt")).unwrap();

        assert_eq!(hash_a, hash_b);
        assert_ne!(hash_a, hash_c);
    }
}
