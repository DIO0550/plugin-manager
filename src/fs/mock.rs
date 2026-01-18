//! テスト用モックファイルシステム

use super::*;
use std::collections::HashMap;
use std::sync::RwLock;

/// テスト用モックファイルシステム
pub struct MockFs {
    files: RwLock<HashMap<String, MockFile>>,
}

struct MockFile {
    content: Vec<u8>,
    mtime: SystemTime,
    file_type: FsFileType,
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
                content: content.as_bytes().to_vec(),
                mtime: SystemTime::now(),
                file_type: FsFileType::File,
            },
        );
    }

    /// バイナリファイルを追加
    pub fn add_file_bytes(&self, path: &str, content: &[u8]) {
        self.files.write().unwrap().insert(
            path.to_string(),
            MockFile {
                content: content.to_vec(),
                mtime: SystemTime::now(),
                file_type: FsFileType::File,
            },
        );
    }

    /// ディレクトリを追加
    pub fn add_dir(&self, path: &str) {
        self.files.write().unwrap().insert(
            path.to_string(),
            MockFile {
                content: Vec::new(),
                mtime: SystemTime::now(),
                file_type: FsFileType::Dir,
            },
        );
    }

    /// シンボリックリンクを追加
    pub fn add_symlink(&self, path: &str) {
        self.files.write().unwrap().insert(
            path.to_string(),
            MockFile {
                content: Vec::new(),
                mtime: SystemTime::now(),
                file_type: FsFileType::Symlink,
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
                file_type: FsFileType::File,
            },
        );
        Ok(())
    }

    fn copy_dir(&self, src: &Path, dst: &Path) -> Result<()> {
        let src_str = src.to_string_lossy().to_string();
        let dst_str = dst.to_string_lossy().to_string();

        // 同一/子孫パスチェック
        if dst_str.starts_with(&src_str) {
            return Err(crate::error::PlmError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Cannot copy directory into itself or its subdirectory",
            )));
        }

        let files = self.files.read().unwrap();
        let entries_to_copy: Vec<_> = files
            .iter()
            .filter(|(path, _)| path.starts_with(&src_str))
            .map(|(path, file)| {
                let relative = path.strip_prefix(&src_str).unwrap_or(path);
                let new_path = format!("{}{}", dst_str, relative);
                (new_path, file.content.clone(), file.file_type)
            })
            .collect();
        drop(files);

        let mut files = self.files.write().unwrap();
        for (new_path, content, file_type) in entries_to_copy {
            files.insert(
                new_path,
                MockFile {
                    content,
                    mtime: SystemTime::now(),
                    file_type,
                },
            );
        }

        Ok(())
    }

    fn remove(&self, path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy().to_string();
        let mut files = self.files.write().unwrap();

        // パスで始まるすべてのエントリを削除（再帰削除）
        files.retain(|k, _| !k.starts_with(&path_str));
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy().to_string();
        let mut files = self.files.write().unwrap();

        if let Some(file) = files.get(&path_str) {
            if file.file_type == FsFileType::Dir {
                return Err(crate::error::PlmError::Io(std::io::Error::new(
                    std::io::ErrorKind::IsADirectory,
                    "Cannot remove directory with remove_file",
                )));
            }
        }

        files.remove(&path_str);
        Ok(())
    }

    fn remove_dir_all(&self, path: &Path) -> Result<()> {
        self.remove(path)
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
            .map(|f| f.file_type == FsFileType::Dir)
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
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "not found").into())
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
            .map(|f| String::from_utf8_lossy(&f.content).to_string())
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "not found").into())
    }

    fn write(&self, path: &Path, content: &[u8]) -> Result<()> {
        self.files.write().unwrap().insert(
            path.to_string_lossy().to_string(),
            MockFile {
                content: content.to_vec(),
                mtime: SystemTime::now(),
                file_type: FsFileType::File,
            },
        );
        Ok(())
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<FsDirEntry>> {
        let path_str = path.to_string_lossy().to_string();
        let files = self.files.read().unwrap();

        // パスがディレクトリとして存在するかチェック
        if let Some(file) = files.get(&path_str) {
            if file.file_type != FsFileType::Dir {
                return Err(crate::error::PlmError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotADirectory,
                    "Not a directory",
                )));
            }
        }

        let prefix = if path_str.ends_with('/') {
            path_str.clone()
        } else {
            format!("{}/", path_str)
        };

        let entries: Vec<_> = files
            .iter()
            .filter(|(k, _)| {
                if !k.starts_with(&prefix) {
                    return false;
                }
                // 直接の子のみ（サブディレクトリの中身は除外）
                let remainder = &k[prefix.len()..];
                !remainder.contains('/')
            })
            .map(|(k, v)| FsDirEntry {
                path: PathBuf::from(k),
                file_type: v.file_type,
            })
            .collect();

        Ok(entries)
    }
}
