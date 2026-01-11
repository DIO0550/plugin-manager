//! プラグインキャッシュマネージャ
//!
//! GitHubからダウンロードしたプラグインのキャッシュ管理を行う。

use super::manifest_resolve::resolve_manifest_path;
use super::PluginManifest;
use crate::error::{PlmError, Result};
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Component as PathComponent, Path, PathBuf};
use zip::ZipArchive;

// Re-export
pub use super::cached_plugin::CachedPlugin;
pub use super::manifest_resolve::has_manifest;

/// プラグインキャッシュマネージャ
pub struct PluginCache {
    /// キャッシュルート: ~/.plm/cache/plugins/
    cache_dir: PathBuf,
}

impl PluginCache {
    /// キャッシュマネージャを初期化（ディレクトリ作成含む）
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME")
            .map_err(|_| PlmError::Cache("HOME environment variable not set".to_string()))?;
        let cache_dir = PathBuf::from(home).join(".plm").join("cache").join("plugins");
        fs::create_dir_all(&cache_dir)?;
        Ok(Self { cache_dir })
    }

    /// カスタムキャッシュディレクトリで初期化（テスト用）
    pub fn with_cache_dir(cache_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&cache_dir)?;
        Ok(Self { cache_dir })
    }

    /// プラグインのキャッシュパスを取得（階層型: marketplace/plugin）
    ///
    /// # Arguments
    /// * `marketplace` - マーケットプレイス名（None の場合は "github" を使用）
    /// * `name` - プラグイン名またはリポジトリ識別子（owner--repo 形式）
    pub fn plugin_path(&self, marketplace: Option<&str>, name: &str) -> PathBuf {
        let marketplace_dir = marketplace.unwrap_or("github");
        self.cache_dir.join(marketplace_dir).join(name)
    }

    /// キャッシュ済みかチェック
    pub fn is_cached(&self, marketplace: Option<&str>, name: &str) -> bool {
        self.plugin_path(marketplace, name).exists()
    }

    /// zipアーカイブを展開してキャッシュに保存
    /// GitHubのzipballは `{repo}-{ref}/` というプレフィックスが付くため、それを除去する
    ///
    /// # Arguments
    /// * `marketplace` - マーケットプレイス名（None の場合は "github" を使用）
    /// * `name` - プラグイン名またはリポジトリ識別子
    /// * `archive` - zipアーカイブのバイト列
    /// * `source_path` - 抽出するソースパス（正規化済み、例: "plugins/my-plugin"）
    ///                   指定時はそのパス配下の内容のみをキャッシュ直下に展開
    pub fn store_from_archive(
        &self,
        marketplace: Option<&str>,
        name: &str,
        archive: &[u8],
        source_path: Option<&str>,
    ) -> Result<PathBuf> {
        // source_path の防御的検証（正規化は行わない、呼び出し元の責務）
        if let Some(sp) = source_path {
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
        }

        let plugin_dir = self.plugin_path(marketplace, name);

        // 既存のキャッシュがあれば削除
        if plugin_dir.exists() {
            fs::remove_dir_all(&plugin_dir)?;
        }

        // zipを展開
        let cursor = Cursor::new(archive);
        let mut zip = ZipArchive::new(cursor)?;

        // 最初のエントリからプレフィックスを取得
        let prefix = if zip.len() > 0 {
            let first = zip.by_index(0)?;
            let first_name = first.name();
            // "repo-branch/" のような形式からプレフィックスを抽出
            first_name
                .split('/')
                .next()
                .map(|s| format!("{}/", s))
                .unwrap_or_default()
        } else {
            String::new()
        };

        // source_path 抽出時のエラートラッキング
        let mut source_path_hit = false;
        let mut _files_extracted = 0usize;
        let mut entries_skipped_for_security = 0usize;

        // 各ファイルを展開
        for i in 0..zip.len() {
            let mut file = zip.by_index(i)?;
            let file_path = file.name();

            // バックスラッシュをスラッシュに正規化（zip内の\区切りエントリ対応）
            let file_path_normalized = file_path.replace('\\', "/");

            // プレフィックスを除去したパスを作成
            let relative_path =
                if !prefix.is_empty() && file_path_normalized.starts_with(&prefix) {
                    &file_path_normalized[prefix.len()..]
                } else {
                    &file_path_normalized[..]
                };

            // 空のパス（ルートディレクトリ）はスキップ
            if relative_path.is_empty() {
                continue;
            }

            // source_path が指定されている場合、そのパス配下のみを抽出
            let final_path = if let Some(sp) = source_path {
                let relative_path_obj = Path::new(relative_path);
                let source_path_obj = Path::new(sp);

                // strip_prefix でパス要素単位の一致判定
                match relative_path_obj.strip_prefix(source_path_obj) {
                    Ok(stripped) => {
                        source_path_hit = true;

                        // strip_prefix 後の空パス（ディレクトリエントリ自体）はスキップ
                        if stripped.as_os_str().is_empty() {
                            continue;
                        }

                        // zip-slip 対策: Normal コンポーネントのみ許容
                        let has_unsafe_component =
                            stripped.components().any(|c| !matches!(c, PathComponent::Normal(_)));
                        if has_unsafe_component {
                            entries_skipped_for_security += 1;
                            continue;
                        }

                        // symlink 対策（source_path 抽出時のみ）
                        #[cfg(unix)]
                        {
                            if let Some(mode) = file.unix_mode() {
                                // S_IFLNK = 0o120000
                                if (mode & 0o170000) == 0o120000 {
                                    entries_skipped_for_security += 1;
                                    continue;
                                }
                            }
                        }

                        stripped.to_path_buf()
                    }
                    Err(_) => {
                        // source_path にマッチしない → スキップ
                        continue;
                    }
                }
            } else {
                PathBuf::from(relative_path)
            };

            let target_path = plugin_dir.join(&final_path);

            if file.is_dir() {
                fs::create_dir_all(&target_path)?;
            } else {
                // 親ディレクトリを作成
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                // ファイルを書き込み
                let mut content = Vec::new();
                file.read_to_end(&mut content)?;
                fs::write(&target_path, content)?;
                _files_extracted += 1;
            }
        }

        // source_path 指定時のエラーチェック
        if source_path.is_some() {
            if entries_skipped_for_security > 0 {
                // セキュリティ理由でスキップされたエントリがあれば全体をエラー
                return Err(PlmError::InvalidSource(format!(
                    "{} entries in source_path were skipped for security reasons (possible zip-slip or symlink)",
                    entries_skipped_for_security
                )));
            }
            if !source_path_hit {
                return Err(PlmError::InvalidSource(format!(
                    "source_path not found in archive: {}",
                    source_path.unwrap()
                )));
            }
            // source_path_hit == true && files_extracted == 0 は
            // ディレクトリエントリのみの場合。後続の plugin.json 不在エラーに委ねる
        }

        // インストール日時を .plm-meta.json に記録（失敗時は警告のみ、処理は継続）
        if let Err(e) = super::meta::write_installed_at(&plugin_dir) {
            eprintln!("Warning: Failed to write installedAt: {}", e);
        }

        Ok(plugin_dir)
    }

    /// キャッシュからマニフェストを読み込み
    ///
    /// 以下の順序でマニフェストを検索:
    /// 1. `.claude-plugin/plugin.json` (推奨)
    /// 2. `plugin.json` (フォールバック)
    pub fn load_manifest(&self, marketplace: Option<&str>, name: &str) -> Result<PluginManifest> {
        let plugin_dir = self.plugin_path(marketplace, name);
        let manifest_path = resolve_manifest_path(&plugin_dir).ok_or_else(|| {
            PlmError::InvalidManifest(format!("plugin.json not found in {:?}", plugin_dir))
        })?;

        PluginManifest::load(&manifest_path)
    }

    /// キャッシュから削除
    pub fn remove(&self, marketplace: Option<&str>, name: &str) -> Result<()> {
        let plugin_dir = self.plugin_path(marketplace, name);
        if plugin_dir.exists() {
            fs::remove_dir_all(&plugin_dir)?;
        }
        Ok(())
    }

    /// 全キャッシュをクリア
    pub fn clear(&self) -> Result<()> {
        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    fs::remove_dir_all(path)?;
                }
            }
        }
        Ok(())
    }

    /// キャッシュされているプラグイン一覧を取得
    /// 階層構造を走査し、(marketplace, plugin_name) のタプルを返す
    pub fn list(&self) -> Result<Vec<(Option<String>, String)>> {
        let mut plugins = Vec::new();

        if !self.cache_dir.exists() {
            return Ok(plugins);
        }

        // marketplace ディレクトリを走査
        for mp_entry in fs::read_dir(&self.cache_dir)? {
            let mp_entry = mp_entry?;
            let mp_path = mp_entry.path();

            if !mp_path.is_dir() {
                continue;
            }

            let marketplace_name = mp_path.file_name().and_then(|n| n.to_str()).map(String::from);

            // marketplace 内のプラグインを走査
            for plugin_entry in fs::read_dir(&mp_path)? {
                let plugin_entry = plugin_entry?;
                let plugin_path = plugin_entry.path();

                if plugin_path.is_dir() {
                    if let Some(plugin_name) = plugin_path.file_name() {
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
}

impl Default for PluginCache {
    fn default() -> Self {
        Self::new().expect("Failed to initialize plugin cache")
    }
}

#[cfg(test)]
#[path = "cache_test.rs"]
mod tests;
