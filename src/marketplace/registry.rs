use crate::error::{PlmError, Result};
use crate::host::HostClient;
use crate::repo::Repo;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const MARKETPLACE_MANIFEST_FILE: &str = ".claude-plugin/marketplace.json";

/// マーケットプレイスオーナー情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceOwner {
    pub name: String,
    #[serde(default)]
    pub email: Option<String>,
}

/// プラグインソース
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PluginSource {
    /// 相対パス: "./plugins/plugin-a"
    Local(String),
    /// 外部GitHub: { "source": "github", "repo": "org/repo" }
    External { source: String, repo: String },
}

/// マーケットプレイス内のプラグイン定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplacePlugin {
    pub name: String,
    pub source: PluginSource,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
}

/// プラグイン検索結果（marketplace + plugin のペア）
#[derive(Debug, Clone)]
pub struct PluginMatch {
    pub marketplace: String,
    pub plugin: MarketplacePlugin,
}

/// marketplace.json のスキーマ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceManifest {
    pub name: String,
    #[serde(default)]
    pub owner: Option<MarketplaceOwner>,
    pub plugins: Vec<MarketplacePlugin>,
}

/// キャッシュされたマーケットプレイス情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceCache {
    pub name: String,
    pub fetched_at: DateTime<Utc>,
    pub source: String,
    #[serde(default)]
    pub owner: Option<MarketplaceOwner>,
    pub plugins: Vec<MarketplacePlugin>,
    /// 元の marketplace.json マニフェスト（旧キャッシュ互換のため読込のみ許可）
    #[serde(default, skip_serializing)]
    pub original_manifest: Option<MarketplaceManifest>,
}

impl MarketplaceCache {
    /// MarketplaceManifest とメタ情報から MarketplaceCache を構築する。
    ///
    /// - `source` は `"github:{owner}/{name}"` 形式で構築
    /// - `fetched_at` は現在時刻（`Utc::now()`）
    /// - `original_manifest` は `None`（呼び出し元で必要なら後から付ける）
    ///
    /// # Arguments
    ///
    /// * `manifest` - Source manifest to convert (consumes `owner` / `plugins`).
    /// * `name` - Marketplace name assigned to the resulting cache entry.
    /// * `repo` - Source repository used to compose the `source` field.
    pub fn from_manifest(manifest: MarketplaceManifest, name: &str, repo: &Repo) -> Self {
        Self {
            name: name.to_string(),
            fetched_at: Utc::now(),
            source: format!("github:{}/{}", repo.owner(), repo.name()),
            owner: manifest.owner,
            plugins: manifest.plugins,
            original_manifest: None,
        }
    }
}

/// マーケットプレイスレジストリ
pub struct MarketplaceRegistry {
    /// キャッシュディレクトリ: ~/.plm/cache/marketplaces/
    cache_dir: PathBuf,
}

impl MarketplaceRegistry {
    /// レジストリを初期化
    ///
    /// `PLM_HOME` が設定されている場合はそちらを優先し、なければ `HOME` にフォールバックする。
    pub fn new() -> Result<Self> {
        let home = crate::env::EnvVar::get("PLM_HOME")
            .or_else(|| crate::env::EnvVar::get("HOME"))
            .ok_or_else(|| {
                PlmError::Cache(
                    "PLM_HOME and HOME environment variables not set or empty".to_string(),
                )
            })?;
        let cache_dir = PathBuf::from(home)
            .join(".plm")
            .join("cache")
            .join("marketplaces");

        fs::create_dir_all(&cache_dir)?;

        Ok(Self { cache_dir })
    }

    /// カスタムキャッシュディレクトリで初期化（テスト用）
    ///
    /// # Arguments
    ///
    /// * `cache_dir` - Custom cache directory to use instead of the default.
    pub fn with_cache_dir(cache_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&cache_dir)?;
        Ok(Self { cache_dir })
    }

    /// キャッシュファイルパスを取得
    ///
    /// # Arguments
    ///
    /// * `name` - Marketplace name whose cache path is requested.
    fn cache_path(&self, name: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.json", name))
    }

    /// キャッシュを取得
    ///
    /// # Arguments
    ///
    /// * `name` - Marketplace name whose cache should be loaded.
    pub fn get(&self, name: &str) -> Result<Option<MarketplaceCache>> {
        let path = self.cache_path(name);
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path)?;
        let cache: MarketplaceCache = serde_json::from_str(&content)?;
        Ok(Some(cache))
    }

    /// キャッシュを保存
    ///
    /// # Arguments
    ///
    /// * `cache` - Marketplace cache entry to persist to disk.
    pub fn store(&self, cache: &MarketplaceCache) -> Result<()> {
        let path = self.cache_path(&cache.name);
        let content = serde_json::to_string_pretty(cache)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// キャッシュを削除
    ///
    /// # Arguments
    ///
    /// * `name` - Marketplace name whose cache file should be removed.
    pub fn remove(&self, name: &str) -> Result<()> {
        let path = self.cache_path(name);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    /// 全マーケットプレイス一覧
    pub fn list(&self) -> Result<Vec<String>> {
        let mut marketplaces = Vec::new();

        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|e| e == "json") {
                    if let Some(name) = path.file_stem() {
                        marketplaces.push(name.to_string_lossy().to_string());
                    }
                }
            }
        }

        Ok(marketplaces)
    }

    /// 全マーケットプレイスからプラグインを検索（最初の1件のみ）
    ///
    /// 注意: 競合検出には `find_plugins()` を使用してください
    ///
    /// # Arguments
    ///
    /// * `plugin_name` - Plugin name to search for across marketplaces.
    pub fn find_plugin(&self, plugin_name: &str) -> Result<Option<(String, MarketplacePlugin)>> {
        for marketplace_name in self.list()? {
            if let Some(cache) = self.get(&marketplace_name)? {
                for plugin in cache.plugins {
                    if plugin.name == plugin_name {
                        return Ok(Some((marketplace_name, plugin)));
                    }
                }
            }
        }
        Ok(None)
    }

    /// 全マーケットプレイスからプラグインを検索（全マッチを返す）
    ///
    /// 同名プラグインが複数のマーケットプレイスに存在する場合、全てを返す
    ///
    /// # Arguments
    ///
    /// * `plugin_name` - Plugin name to search for across marketplaces.
    pub fn find_plugins(&self, plugin_name: &str) -> Result<Vec<PluginMatch>> {
        let mut matches = Vec::new();

        for marketplace_name in self.list()? {
            if let Some(cache) = self.get(&marketplace_name)? {
                for plugin in cache.plugins {
                    if plugin.name == plugin_name {
                        matches.push(PluginMatch {
                            marketplace: marketplace_name.clone(),
                            plugin,
                        });
                    }
                }
            }
        }

        Ok(matches)
    }

    /// 同名プラグインが複数マーケットプレイスに存在するか確認
    ///
    /// # Arguments
    ///
    /// * `plugin_name` - Plugin name to check for conflicts.
    pub fn has_conflict(&self, plugin_name: &str) -> Result<bool> {
        Ok(self.find_plugins(plugin_name)?.len() > 1)
    }

    /// リモートリポジトリから marketplace.json を取得し、
    /// MarketplaceCache に変換して返す。永続化（store）はしない。
    ///
    /// 呼び出し元が必要なタイミングで `self.store(&cache)` を呼ぶ。
    ///
    /// # Arguments
    ///
    /// * `client` - HTTP クライアント。`HostClientFactory::create()` などで生成された実装を渡す。
    /// * `name` - 登録するマーケットプレイス名（`MarketplaceCache.name` に設定される）。
    /// * `repo` - 取得元リポジトリ。`source = "github:{owner}/{name}"` に反映される。
    /// * `source_path` - 正規化済みのサブディレクトリパス（`normalize_source_path` の結果）。
    ///   `None` のときはリポジトリ直下の `.claude-plugin/marketplace.json` を参照する。
    pub async fn fetch_cache(
        &self,
        client: &dyn HostClient,
        name: &str,
        repo: &Repo,
        source_path: Option<&str>,
    ) -> Result<MarketplaceCache> {
        let path = match source_path {
            Some(dir) => format!("{}/{}", dir, MARKETPLACE_MANIFEST_FILE),
            None => MARKETPLACE_MANIFEST_FILE.to_string(),
        };

        let content = client.fetch_file(repo, &path).await?;
        let manifest: MarketplaceManifest = serde_json::from_str(&content).map_err(|e| {
            PlmError::InvalidManifest(format!("Failed to parse marketplace.json: {}", e))
        })?;

        Ok(MarketplaceCache::from_manifest(manifest, name, repo))
    }
}

impl Default for MarketplaceRegistry {
    fn default() -> Self {
        Self::new().expect("Failed to initialize marketplace registry")
    }
}

#[cfg(test)]
#[path = "registry_test.rs"]
mod tests;
