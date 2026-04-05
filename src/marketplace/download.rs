use crate::error::Result;
use crate::plugin::{MarketplacePackage, PackageCache, PackageCacheAccess};
use crate::source::{MarketplaceSource, PluginSource};

/// マーケットプレイス経由のプラグインダウンロード
///
/// デフォルトの `PackageCache` を使用する CLI/TUI 向け便利関数。
pub(crate) async fn download_marketplace_plugin(
    plugin_name: &str,
    marketplace_name: &str,
    force: bool,
) -> Result<MarketplacePackage> {
    let cache = PackageCache::new()?;
    download_marketplace_plugin_with_cache(plugin_name, marketplace_name, force, &cache).await
}

/// キャッシュを注入可能なマーケットプレイス経由のプラグインダウンロード
///
/// テストや DI が必要な場面で使用する。
pub async fn download_marketplace_plugin_with_cache(
    plugin_name: &str,
    marketplace_name: &str,
    force: bool,
    cache: &dyn PackageCacheAccess,
) -> Result<MarketplacePackage> {
    let source = MarketplaceSource::new(plugin_name, marketplace_name);
    let cached = source.download(cache, force).await?;
    Ok(MarketplacePackage::from(cached))
}

#[cfg(test)]
#[path = "download_test.rs"]
mod download_test;
