#[cfg(test)]
use crate::error::PlmError;
use crate::error::Result;
#[cfg(test)]
use crate::marketplace::{MarketplaceRegistry, PluginSourcePath};
use crate::plugin::{MarketplaceContent, PackageCacheAccess};
#[cfg(test)]
use crate::source::GitHubSource;
use crate::source::{MarketplaceSource, PackageSource};

/// キャッシュを注入可能なマーケットプレイス経由のプラグインダウンロード
///
/// テストや DI が必要な場面で使用する。
pub async fn download_marketplace_plugin_with_cache(
    plugin_name: &str,
    marketplace_name: &str,
    force: bool,
    cache: &dyn PackageCacheAccess,
) -> Result<MarketplaceContent> {
    let source = MarketplaceSource::new(plugin_name, marketplace_name);
    let cached = source.download(cache, force).await?;
    Ok(MarketplaceContent::from(cached))
}

/// レジストリを注入可能なマーケットプレイス経由のプラグインダウンロード（テスト用）
///
/// `MarketplaceRegistry` を外部から渡せるため、環境変数に依存しない。
#[cfg(test)]
async fn download_marketplace_plugin_with_registry(
    plugin_name: &str,
    marketplace_name: &str,
    force: bool,
    cache: &dyn PackageCacheAccess,
    registry: &MarketplaceRegistry,
) -> Result<MarketplaceContent> {
    use crate::marketplace::PluginSource as MpPluginSource;

    let mp_cache = registry
        .get(marketplace_name)?
        .ok_or_else(|| PlmError::MarketplaceNotFound(marketplace_name.to_string()))?;

    let plugin_entry = mp_cache
        .plugins
        .iter()
        .find(|p| p.name == plugin_name)
        .ok_or_else(|| PlmError::PluginNotFound(plugin_name.to_string()))?;

    let cached = match &plugin_entry.source {
        MpPluginSource::Local(path) => {
            let parts: Vec<&str> = mp_cache
                .source
                .strip_prefix("github:")
                .unwrap_or(&mp_cache.source)
                .split('/')
                .collect();

            if parts.len() < 2 {
                return Err(PlmError::InvalidRepoFormat(mp_cache.source.clone()));
            }

            let owner = parts[0];
            let repo_name = parts[1];
            let repo = crate::repo::from_url(&format!("{}/{}", owner, repo_name))?;
            let source_path: PluginSourcePath = path.parse()?;

            GitHubSource::with_marketplace_and_source_path(
                repo,
                marketplace_name.to_string(),
                source_path.into(),
            )
            .download(cache, force)
            .await?
        }
        MpPluginSource::External { repo: repo_url, .. } => {
            let repo = crate::repo::from_url(repo_url)?;
            GitHubSource::with_marketplace(repo, marketplace_name.to_string())
                .download(cache, force)
                .await?
        }
    };

    Ok(MarketplaceContent::from(cached))
}

#[cfg(test)]
#[path = "download_test.rs"]
mod download_test;
