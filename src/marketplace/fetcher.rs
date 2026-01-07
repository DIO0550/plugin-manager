use crate::error::{PlmError, Result};
use crate::host::HostClientFactory;
use crate::marketplace::{MarketplaceCache, MarketplaceManifest};
use crate::repo::Repo;
use chrono::Utc;

/// マーケットプレイス取得クライアント
pub struct MarketplaceFetcher {
    factory: HostClientFactory,
}

impl MarketplaceFetcher {
    /// 新しいフェッチャーを作成
    pub fn new() -> Self {
        Self {
            factory: HostClientFactory::with_defaults(),
        }
    }

    /// GitHubリポジトリから marketplace.json を取得
    pub async fn fetch(&self, repo: &Repo, source_path: Option<&str>) -> Result<MarketplaceManifest> {
        let path = match source_path {
            Some(dir) => format!("{}/.claude-plugin/marketplace.json", dir),
            None => ".claude-plugin/marketplace.json".to_string(),
        };

        let client = self.factory.create(repo.host());
        let content = client.fetch_file(repo, &path).await?;

        serde_json::from_str(&content).map_err(|e| {
            PlmError::InvalidManifest(format!("Failed to parse marketplace.json: {}", e))
        })
    }

    /// マーケットプレイスを取得してキャッシュ形式に変換
    pub async fn fetch_as_cache(
        &self,
        repo: &Repo,
        name: &str,
        source_path: Option<&str>,
    ) -> Result<MarketplaceCache> {
        let manifest = self.fetch(repo, source_path).await?;

        Ok(MarketplaceCache {
            name: name.to_string(),
            fetched_at: Utc::now(),
            source: format!("github:{}/{}", repo.owner(), repo.name()),
            owner: manifest.owner,
            plugins: manifest.plugins,
        })
    }
}

impl Default for MarketplaceFetcher {
    fn default() -> Self {
        Self::new()
    }
}
