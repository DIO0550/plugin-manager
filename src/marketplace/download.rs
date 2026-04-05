use crate::plugin::{MarketplacePackage, PackageCache, PackageCacheAccess};
use crate::source::{MarketplaceSource, PluginSource};

/// マーケットプレイス経由のプラグインダウンロード
///
/// デフォルトの `PackageCache` を使用する CLI/TUI 向け便利関数。
pub(crate) async fn download_marketplace_plugin(
    plugin_name: &str,
    marketplace_name: &str,
    force: bool,
) -> Result<MarketplacePackage, String> {
    let cache = PackageCache::new().map_err(|e| format!("Failed to access cache: {e}"))?;
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
) -> Result<MarketplacePackage, String> {
    let source = MarketplaceSource::new(plugin_name, marketplace_name);
    let cached = source
        .download(cache, force)
        .await
        .map_err(|e| e.to_string())?;
    Ok(MarketplacePackage::from(cached))
}

#[cfg(test)]
#[path = "download_test.rs"]
mod download_test;
