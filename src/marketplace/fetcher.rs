use crate::error::{PlmError, Result};
use crate::github::{GitHubClient, GitRepo};
use crate::marketplace::{MarketplaceCache, MarketplaceManifest};
use chrono::Utc;

/// マーケットプレイス取得クライアント
pub struct MarketplaceFetcher {
    github: GitHubClient,
}

impl MarketplaceFetcher {
    /// 新しいフェッチャーを作成
    pub fn new() -> Self {
        Self {
            github: GitHubClient::new(),
        }
    }

    /// GitHubリポジトリから marketplace.json を取得
    pub async fn fetch(
        &self,
        repo: &GitRepo,
        subdir: Option<&str>,
    ) -> Result<MarketplaceManifest> {
        let path = match subdir {
            Some(dir) => format!("{}/.claude-plugin/marketplace.json", dir),
            None => ".claude-plugin/marketplace.json".to_string(),
        };

        let content = self.github.fetch_file(repo, &path).await?;

        serde_json::from_str(&content).map_err(|e| {
            PlmError::InvalidManifest(format!("Failed to parse marketplace.json: {}", e))
        })
    }

    /// マーケットプレイスを取得してキャッシュ形式に変換
    pub async fn fetch_as_cache(
        &self,
        repo: &GitRepo,
        name: &str,
        subdir: Option<&str>,
    ) -> Result<MarketplaceCache> {
        let manifest = self.fetch(repo, subdir).await?;

        Ok(MarketplaceCache {
            name: name.to_string(),
            fetched_at: Utc::now(),
            source: format!("github:{}/{}", repo.owner, repo.repo),
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
