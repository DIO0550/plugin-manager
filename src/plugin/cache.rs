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
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// テスト用のzipアーカイブを作成するヘルパー
    fn create_test_archive(entries: &[(&str, &str)]) -> Vec<u8> {
        use std::io::Write;
        let mut buf = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
            let options = zip::write::SimpleFileOptions::default();

            for (path, content) in entries {
                zip.start_file(*path, options).unwrap();
                zip.write_all(content.as_bytes()).unwrap();
            }
            zip.finish().unwrap();
        }
        buf
    }

    #[test]
    fn test_store_from_archive_with_source_path_extracts_to_root() {
        // テストケース14: source_path 指定時、そのパス配下の内容がキャッシュ直下に展開される
        let temp_dir = TempDir::new().unwrap();
        let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

        // GitHub 形式のアーカイブ（prefix + source_path）
        let archive = create_test_archive(&[
            (
                "repo-main/plugins/my-plugin/plugin.json",
                r#"{"name":"test","version":"1.0.0"}"#,
            ),
            ("repo-main/plugins/my-plugin/skills/test.md", "# Test Skill"),
            ("repo-main/other/file.txt", "should not be extracted"),
        ]);

        let result = cache.store_from_archive(
            Some("test-marketplace"),
            "my-plugin",
            &archive,
            Some("plugins/my-plugin"),
        );

        assert!(result.is_ok());
        let plugin_dir = result.unwrap();

        // サブディレクトリの内容がキャッシュ直下に展開されている
        assert!(plugin_dir.join("plugin.json").exists());
        assert!(plugin_dir.join("skills/test.md").exists());
        // 他のファイルは展開されない
        assert!(!plugin_dir.join("other").exists());
    }

    #[test]
    fn test_store_from_archive_source_path_boundary_match() {
        // テストケース15: source_path = "plugins/foo" で plugins/foo-bar が誤抽出されない
        let temp_dir = TempDir::new().unwrap();
        let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

        let archive = create_test_archive(&[
            ("repo-main/plugins/foo/file.txt", "correct"),
            ("repo-main/plugins/foo-bar/file.txt", "should not match"),
            ("repo-main/plugins/foobar/file.txt", "should not match either"),
        ]);

        let result = cache.store_from_archive(
            Some("test-marketplace"),
            "foo-plugin",
            &archive,
            Some("plugins/foo"),
        );

        assert!(result.is_ok());
        let plugin_dir = result.unwrap();

        // plugins/foo の内容のみ展開
        assert!(plugin_dir.join("file.txt").exists());
        let content = fs::read_to_string(plugin_dir.join("file.txt")).unwrap();
        assert_eq!(content, "correct");

        // plugins/foo-bar や plugins/foobar は展開されない
        // （ディレクトリ自体が存在しないことを確認）
        let entries: Vec<_> = fs::read_dir(&plugin_dir).unwrap().collect();
        assert_eq!(entries.len(), 2); // file.txt + .plm-meta.json
    }

    #[test]
    fn test_store_from_archive_source_path_not_found() {
        // テストケース16: source_path がアーカイブ内に存在しない場合 → InvalidSource エラー
        let temp_dir = TempDir::new().unwrap();
        let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

        let archive = create_test_archive(&[("repo-main/other/file.txt", "content")]);

        let result = cache.store_from_archive(
            Some("test-marketplace"),
            "my-plugin",
            &archive,
            Some("plugins/nonexistent"),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            PlmError::InvalidSource(msg) => {
                assert!(msg.contains("source_path not found"));
            }
            e => panic!("Expected InvalidSource error, got: {:?}", e),
        }
    }

    #[test]
    fn test_store_from_archive_source_path_validation_dotdot() {
        // テストケース21: source_path に .. が含まれる場合 → エラー
        let temp_dir = TempDir::new().unwrap();
        let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

        let archive = create_test_archive(&[("repo-main/plugins/foo/file.txt", "content")]);

        let result = cache.store_from_archive(
            Some("test-marketplace"),
            "my-plugin",
            &archive,
            Some("plugins/../foo"),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            PlmError::InvalidSource(msg) => {
                assert!(msg.contains("not normalized"));
            }
            e => panic!("Expected InvalidSource error, got: {:?}", e),
        }
    }

    #[test]
    fn test_store_from_archive_source_path_validation_backslash() {
        // テストケース21: source_path に \ が含まれる場合 → エラー
        let temp_dir = TempDir::new().unwrap();
        let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

        let archive = create_test_archive(&[("repo-main/plugins/foo/file.txt", "content")]);

        let result = cache.store_from_archive(
            Some("test-marketplace"),
            "my-plugin",
            &archive,
            Some("plugins\\foo"),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            PlmError::InvalidSource(msg) => {
                assert!(msg.contains("not normalized"));
            }
            e => panic!("Expected InvalidSource error, got: {:?}", e),
        }
    }

    #[test]
    fn test_store_from_archive_source_path_validation_dot_slash() {
        // テストケース21: source_path に ./ が含まれる場合 → エラー
        let temp_dir = TempDir::new().unwrap();
        let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

        let archive = create_test_archive(&[("repo-main/plugins/foo/file.txt", "content")]);

        let result = cache.store_from_archive(
            Some("test-marketplace"),
            "my-plugin",
            &archive,
            Some("./plugins/foo"),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            PlmError::InvalidSource(msg) => {
                assert!(msg.contains("not normalized"));
            }
            e => panic!("Expected InvalidSource error, got: {:?}", e),
        }
    }

    #[test]
    fn test_store_from_archive_without_source_path_extracts_all() {
        // source_path = None の場合は従来通り全ファイルを展開
        let temp_dir = TempDir::new().unwrap();
        let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

        let archive = create_test_archive(&[
            ("repo-main/plugin.json", r#"{"name":"test","version":"1.0.0"}"#),
            ("repo-main/skills/test.md", "# Test"),
            ("repo-main/other/file.txt", "content"),
        ]);

        let result = cache.store_from_archive(None, "test-plugin", &archive, None);

        assert!(result.is_ok());
        let plugin_dir = result.unwrap();

        // 全ファイルが展開される
        assert!(plugin_dir.join("plugin.json").exists());
        assert!(plugin_dir.join("skills/test.md").exists());
        assert!(plugin_dir.join("other/file.txt").exists());
    }

    #[test]
    fn test_store_from_archive_handles_backslash_entries() {
        // テストケース20: zip内の \ 区切りエントリを / に正規化後一致
        let temp_dir = TempDir::new().unwrap();
        let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

        // バックスラッシュを含むエントリ名（Windows由来のzip）
        // プレフィックスはスラッシュで書く（プレフィックス抽出は / でsplitするため）
        let archive = create_test_archive(&[(
            "repo-main/plugins\\foo\\file.txt",
            "content with backslash",
        )]);

        let result = cache.store_from_archive(
            Some("test-marketplace"),
            "foo-plugin",
            &archive,
            Some("plugins/foo"),
        );

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
        let plugin_dir = result.unwrap();
        assert!(plugin_dir.join("file.txt").exists());
    }

    #[test]
    fn test_store_from_archive_writes_installed_at_to_meta() {
        // store_from_archive 後に .plm-meta.json に installedAt が書き込まれることを確認
        let temp_dir = TempDir::new().unwrap();
        let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

        let archive = create_test_archive(&[(
            "repo-main/plugin.json",
            r#"{"name":"test","version":"1.0.0"}"#,
        )]);

        let result = cache.store_from_archive(None, "test-plugin", &archive, None);
        assert!(result.is_ok());
        let plugin_dir = result.unwrap();

        // .plm-meta.json を読み込んで installedAt を確認
        let meta_path = plugin_dir.join(".plm-meta.json");
        assert!(meta_path.exists(), ".plm-meta.json should exist");

        let content = fs::read_to_string(&meta_path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert!(json.get("installedAt").is_some());
        let installed_at = json.get("installedAt").unwrap().as_str().unwrap();
        // RFC3339 形式であることを確認（YYYY-MM-DDTHH:MM:SSZ）
        assert!(installed_at.contains("T"));
        assert!(installed_at.ends_with("Z"));
    }

    #[test]
    fn test_store_from_archive_does_not_modify_plugin_json() {
        // plugin.json が改変されないことを確認（上流成果物の保持）
        let temp_dir = TempDir::new().unwrap();
        let cache = PluginCache::with_cache_dir(temp_dir.path().to_path_buf()).unwrap();

        let original_content = r#"{"name":"test","version":"1.0.0","customField":"preserved"}"#;
        let archive = create_test_archive(&[("repo-main/plugin.json", original_content)]);

        let result = cache.store_from_archive(None, "test-plugin", &archive, None);
        assert!(result.is_ok());
        let plugin_dir = result.unwrap();

        // plugin.json が改変されていないことを確認
        let content = fs::read_to_string(plugin_dir.join("plugin.json")).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();

        // customField が保持されている
        assert_eq!(
            json.get("customField").unwrap().as_str().unwrap(),
            "preserved"
        );
        // installedAt は追加されていない（.plm-meta.json に記録される）
        assert!(
            json.get("installedAt").is_none(),
            "plugin.json should not have installedAt"
        );

        // .plm-meta.json に installedAt がある
        let meta_content = fs::read_to_string(plugin_dir.join(".plm-meta.json")).unwrap();
        let meta_json: serde_json::Value = serde_json::from_str(&meta_content).unwrap();
        assert!(meta_json.get("installedAt").is_some());
    }
}
