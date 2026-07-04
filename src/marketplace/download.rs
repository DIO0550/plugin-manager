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
///
/// # Arguments
///
/// * `plugin_name` - Name of the plugin to download.
/// * `marketplace_name` - Name of the marketplace that hosts the plugin.
/// * `force` - Whether to bypass the cache and force a fresh download.
/// * `cache` - Package cache accessor used to read and write cached downloads.
pub async fn download_marketplace_plugin_with_cache(
    plugin_name: &str,
    marketplace_name: &str,
    force: bool,
    cache: &dyn PackageCacheAccess,
) -> Result<MarketplaceContent> {
    let source = MarketplaceSource::new(plugin_name, marketplace_name);
    let cached = source.download(cache, force).await?;
    MarketplaceContent::try_from(cached)
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
            let repo = mp_cache.source.to_repo();
            let source_path: PluginSourcePath = path.parse()?;

            GitHubSource::with_marketplace_plugin(
                repo,
                marketplace_name.to_string(),
                Some(source_path.into()),
                plugin_entry.name.clone(),
            )
            .download(cache, force)
            .await?
        }
        MpPluginSource::External { repo: repo_url, .. } => {
            let repo = crate::repo::from_url(repo_url)?;
            GitHubSource::with_marketplace_plugin(
                repo,
                marketplace_name.to_string(),
                None,
                plugin_entry.name.clone(),
            )
            .download(cache, force)
            .await?
        }
    };

    MarketplaceContent::try_from(cached)
}

#[cfg(test)]
#[path = "download_test.rs"]
mod download_test;
