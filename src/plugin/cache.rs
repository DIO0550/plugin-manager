//! プラグインキャッシュマネージャ
//!
//! GitHubからダウンロードしたプラグインのキャッシュ管理を行う。

use super::manifest_resolve::resolve_manifest_path;
use super::PluginManifest;
use crate::error::{PlmError, Result};
use crate::fs::{FileSystem, RealFs};
use std::io::{Cursor, Read};
use std::path::{Component as PathComponent, Path, PathBuf};
use zip::ZipArchive;

// Re-export
pub use super::cached_plugin::CachedPlugin;
pub use super::manifest_resolve::has_manifest;

/// プラグインキャッシュアクセスの抽象化トレイト
///
/// 消費者はこの trait 経由でキャッシュ操作を行う。
/// テスト時は `PluginCache::with_cache_dir(tempdir)` で tempdir ベースの
/// 本番実装を注入する。
pub trait PluginCacheAccess: Send + Sync {
    /// プラグインのキャッシュパスを取得
    fn plugin_path(&self, marketplace: Option<&str>, name: &str) -> PathBuf;

    /// キャッシュ済みかチェック
    fn is_cached(&self, marketplace: Option<&str>, name: &str) -> bool;

    /// zipアーカイブを展開してキャッシュに保存
    fn store_from_archive(
        &self,
        marketplace: Option<&str>,
        name: &str,
        archive: &[u8],
        source_path: Option<&str>,
    ) -> Result<PathBuf>;

    /// キャッシュからマニフェストを読み込み
    fn load_manifest(&self, marketplace: Option<&str>, name: &str) -> Result<PluginManifest>;

    /// キャッシュから削除
    fn remove(&self, marketplace: Option<&str>, name: &str) -> Result<()>;

    /// キャッシュされているプラグイン一覧を取得
    fn list(&self) -> Result<Vec<(Option<String>, String)>>;

    /// プラグインをバックアップ
    fn backup(&self, marketplace: Option<&str>, name: &str) -> Result<PathBuf>;

    /// バックアップからリストア
    fn restore(&self, marketplace: Option<&str>, name: &str) -> Result<()>;

    /// バックアップを削除
    fn remove_backup(&self, marketplace: Option<&str>, name: &str) -> Result<()>;

    /// アトミック更新（temp展開 → 検証 → リネーム）
    fn atomic_update(
        &self,
        marketplace: Option<&str>,
        name: &str,
        archive: &[u8],
    ) -> Result<PathBuf>;
}

/// プラグインキャッシュマネージャ
pub struct PluginCache {
    /// キャッシュルート: ~/.plm/cache/plugins/
    cache_dir: PathBuf,
}

impl PluginCache {
    /// キャッシュマネージャを初期化（ディレクトリ作成含む）
    pub fn new() -> Result<Self> {
        let fs = RealFs;
        let home = std::env::var("HOME")
            .map_err(|_| PlmError::Cache("HOME environment variable not set".to_string()))?;
        let cache_dir = PathBuf::from(home)
            .join(".plm")
            .join("cache")
            .join("plugins");
        fs.create_dir_all(&cache_dir)?;
        Ok(Self { cache_dir })
    }

    /// カスタムキャッシュディレクトリで初期化（テスト用）
    pub fn with_cache_dir(cache_dir: PathBuf) -> Result<Self> {
        let fs = RealFs;
        fs.create_dir_all(&cache_dir)?;
        Ok(Self { cache_dir })
    }

    /// 全キャッシュをクリア
    pub fn clear(&self) -> Result<()> {
        let fs = RealFs;
        if fs.exists(&self.cache_dir) {
            for entry in fs.read_dir(&self.cache_dir)? {
                if entry.is_dir() {
                    fs.remove_dir_all(&entry.path)?;
                }
            }
        }
        Ok(())
    }

    /// バックアップパスを取得
    fn backup_path(&self, marketplace: Option<&str>, name: &str) -> PathBuf {
        let marketplace_dir = marketplace.unwrap_or("github");
        self.cache_dir
            .join(".backup")
            .join(marketplace_dir)
            .join(name)
    }

    /// 一時パスを取得
    fn temp_path(&self, marketplace: Option<&str>, name: &str) -> PathBuf {
        let marketplace_dir = marketplace.unwrap_or("github");
        self.cache_dir
            .join(".temp")
            .join(marketplace_dir)
            .join(name)
    }
}

impl PluginCacheAccess for PluginCache {
    fn plugin_path(&self, marketplace: Option<&str>, name: &str) -> PathBuf {
        let marketplace_dir = marketplace.unwrap_or("github");
        self.cache_dir.join(marketplace_dir).join(name)
    }

    fn is_cached(&self, marketplace: Option<&str>, name: &str) -> bool {
        let fs = RealFs;
        let plugin_path = self.plugin_path(marketplace, name);
        fs.exists(&plugin_path)
    }

    fn store_from_archive(
        &self,
        marketplace: Option<&str>,
        name: &str,
        archive: &[u8],
        source_path: Option<&str>,
    ) -> Result<PathBuf> {
        let fs = RealFs;

        // source_path の防御的検証
        validate_source_path(source_path)?;

        let plugin_dir = self.plugin_path(marketplace, name);

        // 既存のキャッシュがあれば削除
        if fs.exists(&plugin_dir) {
            fs.remove_dir_all(&plugin_dir)?;
        }

        // アーカイブを展開
        extract_archive_with_source_path(&plugin_dir, archive, source_path)?;

        // インストール日時を .plm-meta.json に記録（失敗時は警告のみ、処理は継続）
        if let Err(e) = super::meta::write_installed_at(&plugin_dir) {
            eprintln!("Warning: Failed to write installedAt: {}", e);
        }

        Ok(plugin_dir)
    }

    fn load_manifest(&self, marketplace: Option<&str>, name: &str) -> Result<PluginManifest> {
        let plugin_dir = self.plugin_path(marketplace, name);
        let manifest_path = resolve_manifest_path(&plugin_dir).ok_or_else(|| {
            PlmError::InvalidManifest(format!("plugin.json not found in {:?}", plugin_dir))
        })?;

        PluginManifest::load(&manifest_path)
    }

    fn remove(&self, marketplace: Option<&str>, name: &str) -> Result<()> {
        let fs = RealFs;
        let plugin_dir = self.plugin_path(marketplace, name);
        if fs.exists(&plugin_dir) {
            fs.remove_dir_all(&plugin_dir)?;
        }
        Ok(())
    }

    fn list(&self) -> Result<Vec<(Option<String>, String)>> {
        let fs = RealFs;
        let mut plugins = Vec::new();

        if !fs.exists(&self.cache_dir) {
            return Ok(plugins);
        }

        // marketplace ディレクトリを走査
        for mp_entry in fs.read_dir(&self.cache_dir)? {
            if !mp_entry.is_dir() {
                continue;
            }

            let mp_path = &mp_entry.path;
            let marketplace_name = mp_path
                .file_name()
                .and_then(|n| n.to_str())
                .map(String::from);

            // marketplace 内のプラグインを走査
            for plugin_entry in fs.read_dir(mp_path)? {
                if plugin_entry.is_dir() {
                    if let Some(plugin_name) = plugin_entry.path.file_name() {
                        let plugin_name = plugin_name.to_string_lossy().to_string();
                        // "github" marketplace は None として扱う
                        let mp = if marketplace_name.as_deref() == Some("github") {
                            None
                        } else {
                            marketplace_name.clone()
                        };
                        plugins.push((mp, plugin_name));
                    }
                }
            }
        }

        Ok(plugins)
    }

    fn backup(&self, marketplace: Option<&str>, name: &str) -> Result<PathBuf> {
        let fs = RealFs;
        let source = self.plugin_path(marketplace, name);
        let backup_dir = self.backup_path(marketplace, name);

        if !fs.exists(&source) {
            return Err(PlmError::Cache(format!(
                "Plugin not found: {}",
                source.display()
            )));
        }

        // 既存バックアップがあれば削除
        if fs.exists(&backup_dir) {
            fs.remove_dir_all(&backup_dir)?;
        }

        // 親ディレクトリを作成
        if let Some(parent) = backup_dir.parent() {
            fs.create_dir_all(parent)?;
        }

        fs.copy_dir(&source, &backup_dir)?;
        Ok(backup_dir)
    }

    fn restore(&self, marketplace: Option<&str>, name: &str) -> Result<()> {
        let fs = RealFs;
        let backup_dir = self.backup_path(marketplace, name);
        let target = self.plugin_path(marketplace, name);

        if !fs.exists(&backup_dir) {
            return Err(PlmError::Cache("Backup not found".to_string()));
        }

        // 現在のディレクトリを削除
        if fs.exists(&target) {
            fs.remove_dir_all(&target)?;
        }

        // バックアップからリストア
        fs.copy_dir(&backup_dir, &target)?;

        // バックアップを削除
        fs.remove_dir_all(&backup_dir)?;

        Ok(())
    }

    fn remove_backup(&self, marketplace: Option<&str>, name: &str) -> Result<()> {
        let fs = RealFs;
        let backup_dir = self.backup_path(marketplace, name);
        if fs.exists(&backup_dir) {
            fs.remove_dir_all(&backup_dir)?;
        }
        Ok(())
    }

    fn atomic_update(
        &self,
        marketplace: Option<&str>,
        name: &str,
        archive: &[u8],
    ) -> Result<PathBuf> {
        let fs = RealFs;
        let target = self.plugin_path(marketplace, name);
        let temp_dir = self.temp_path(marketplace, name);

        // temp ディレクトリをクリーンアップ
        if fs.exists(&temp_dir) {
            fs.remove_dir_all(&temp_dir)?;
        }

        // 親ディレクトリを作成
        if let Some(parent) = temp_dir.parent() {
            fs.create_dir_all(parent)?;
        }

        // temp に展開（source_path なし）
        extract_archive_with_source_path(&temp_dir, archive, None)?;

        // 検証: plugin.json の存在確認
        if !has_manifest(&temp_dir) {
            fs.remove_dir_all(&temp_dir)?;
            return Err(PlmError::InvalidManifest("plugin.json not found".into()));
        }

        // 旧キャッシュ削除 → temp をリネーム
        if fs.exists(&target) {
            fs.remove_dir_all(&target)?;
        }

        // リネーム（同一ファイルシステム上でのアトミック操作）
        fs.rename(&temp_dir, &target)?;

        Ok(target)
    }
}

/// source_path の防御的検証
fn validate_source_path(source_path: Option<&str>) -> Result<()> {
    let sp = match source_path {
        Some(s) => s,
        None => return Ok(()),
    };

    if sp.contains("..") {
        return Err(PlmError::InvalidSource(
            "source_path is not normalized: contains '..'".into(),
        ));
    }
    if sp.contains('\\') {
        return Err(PlmError::InvalidSource(
            "source_path is not normalized: contains backslash".into(),
        ));
    }
    if sp.contains("./") || sp.starts_with('.') {
        return Err(PlmError::InvalidSource(
            "source_path is not normalized: contains './' or starts with '.'".into(),
        ));
    }
    if Path::new(sp).is_absolute() {
        return Err(PlmError::InvalidSource(
            "source_path is not normalized: absolute path".into(),
        ));
    }

    Ok(())
}

/// zipアーカイブのプレフィックス（トップディレクトリ）を取得
fn get_archive_prefix(zip: &mut ZipArchive<Cursor<&[u8]>>) -> Result<String> {
    if zip.is_empty() {
        return Ok(String::new());
    }

    let first = zip.by_index(0)?;
    let first_name = first.name();
    Ok(first_name
        .split('/')
        .next()
        .map(|s| format!("{}/", s))
        .unwrap_or_default())
}

/// アーカイブを展開（source_path 指定対応）
fn extract_archive_with_source_path(
    dest: &Path,
    archive: &[u8],
    source_path: Option<&str>,
) -> Result<()> {
    let cursor = Cursor::new(archive);
    let mut zip = ZipArchive::new(cursor)?;

    let prefix = get_archive_prefix(&mut zip)?;

    let mut source_path_hit = false;
    let mut entries_skipped_for_security = 0usize;

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let file_path = file.name();

        // バックスラッシュをスラッシュに正規化
        let file_path_normalized = file_path.replace('\\', "/");

        // プレフィックスを除去
        let relative_path = if !prefix.is_empty() && file_path_normalized.starts_with(&prefix) {
            &file_path_normalized[prefix.len()..]
        } else {
            &file_path_normalized[..]
        };

        // 空のパス（ルートディレクトリ）はスキップ
        if relative_path.is_empty() {
            continue;
        }

        // source_path が指定されている場合、そのパス配下のみを抽出
        let final_path = match source_path {
            Some(sp) => {
                match extract_with_source_path_filter(
                    relative_path,
                    sp,
                    &file,
                    &mut source_path_hit,
                    &mut entries_skipped_for_security,
                ) {
                    Some(path) => path,
                    None => continue,
                }
            }
            None => PathBuf::from(relative_path),
        };

        write_zip_entry(&mut file, &dest.join(&final_path))?;
    }

    // source_path 指定時のエラーチェック
    if let Some(sp) = source_path {
        if entries_skipped_for_security > 0 {
            return Err(PlmError::InvalidSource(format!(
                "{} entries in source_path were skipped for security reasons (possible zip-slip or symlink)",
                entries_skipped_for_security
            )));
        }
        if !source_path_hit {
            return Err(PlmError::InvalidSource(format!(
                "source_path not found in archive: {}",
                sp
            )));
        }
    }

    Ok(())
}

/// source_path フィルタを適用し、展開すべきパスを返す（None = スキップ）
fn extract_with_source_path_filter(
    relative_path: &str,
    source_path: &str,
    file: &zip::read::ZipFile,
    source_path_hit: &mut bool,
    entries_skipped: &mut usize,
) -> Option<PathBuf> {
    let relative_path_obj = Path::new(relative_path);
    let source_path_obj = Path::new(source_path);

    let stripped = relative_path_obj.strip_prefix(source_path_obj).ok()?;
    *source_path_hit = true;

    // strip_prefix 後の空パス（ディレクトリエントリ自体）はスキップ
    if stripped.as_os_str().is_empty() {
        return None;
    }

    // zip-slip 対策: Normal コンポーネントのみ許容
    let has_unsafe_component = stripped
        .components()
        .any(|c| !matches!(c, PathComponent::Normal(_)));
    if has_unsafe_component {
        *entries_skipped += 1;
        return None;
    }

    // symlink 対策（source_path 抽出時のみ）
    #[cfg(unix)]
    {
        if let Some(mode) = file.unix_mode() {
            // S_IFLNK = 0o120000
            if (mode & 0o170000) == 0o120000 {
                *entries_skipped += 1;
                return None;
            }
        }
    }
    #[cfg(not(unix))]
    let _ = file;

    Some(stripped.to_path_buf())
}

/// zipエントリをファイルシステムに書き込み
fn write_zip_entry(file: &mut zip::read::ZipFile, target_path: &Path) -> Result<()> {
    let fs = RealFs;
    if file.is_dir() {
        fs.create_dir_all(target_path)?;
    } else {
        if let Some(parent) = target_path.parent() {
            fs.create_dir_all(parent)?;
        }
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;
        fs.write(target_path, &content)?;
    }
    Ok(())
}

impl Default for PluginCache {
    fn default() -> Self {
        Self::new().expect("Failed to initialize plugin cache")
    }
}

#[cfg(test)]
#[path = "cache_test.rs"]
mod tests;
