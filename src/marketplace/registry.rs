use crate::error::{PlmError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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

/// マーケットプレイス内のプラグインエントリ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplacePluginEntry {
    pub name: String,
    pub source: PluginSource,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
}

/// marketplace.json のスキーマ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceManifest {
    pub name: String,
    #[serde(default)]
    pub owner: Option<MarketplaceOwner>,
    pub plugins: Vec<MarketplacePluginEntry>,
}

/// キャッシュされたマーケットプレイス情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceCache {
    pub name: String,
    pub fetched_at: DateTime<Utc>,
    pub source: String,
    #[serde(default)]
    pub owner: Option<MarketplaceOwner>,
    pub plugins: Vec<MarketplacePluginEntry>,
}

/// マーケットプレイスレジストリ
pub struct MarketplaceRegistry {
    /// キャッシュディレクトリ: ~/.plm/cache/marketplaces/
    cache_dir: PathBuf,
}

impl MarketplaceRegistry {
    /// レジストリを初期化
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME")
            .map_err(|_| PlmError::Cache("HOME environment variable not set".to_string()))?;
        let cache_dir = PathBuf::from(home)
            .join(".plm")
            .join("cache")
            .join("marketplaces");

        fs::create_dir_all(&cache_dir)?;

        Ok(Self { cache_dir })
    }

    /// カスタムキャッシュディレクトリで初期化（テスト用）
    pub fn with_cache_dir(cache_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&cache_dir)?;
        Ok(Self { cache_dir })
    }

    /// キャッシュファイルパスを取得
    fn cache_path(&self, name: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.json", name))
    }

    /// キャッシュを取得
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
    pub fn store(&self, cache: &MarketplaceCache) -> Result<()> {
        let path = self.cache_path(&cache.name);
        let content = serde_json::to_string_pretty(cache)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// キャッシュを削除
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
                if path.is_file() && path.extension().map_or(false, |e| e == "json") {
                    if let Some(name) = path.file_stem() {
                        marketplaces.push(name.to_string_lossy().to_string());
                    }
                }
            }
        }

        Ok(marketplaces)
    }

    /// 全マーケットプレイスからプラグインを検索
    pub fn find_plugin(
        &self,
        plugin_name: &str,
    ) -> Result<Option<(String, MarketplacePluginEntry)>> {
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
}

impl Default for MarketplaceRegistry {
    fn default() -> Self {
        Self::new().expect("Failed to initialize marketplace registry")
    }
}
