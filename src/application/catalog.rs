//! プラグインカタログ
//!
//! インストール済みプラグインの一覧取得ユースケースを提供する。

use crate::error::Result;
use crate::plugin::{list_installed, meta, InstalledPlugin, PackageCacheAccess, Plugin};
use crate::target::list_all_placed;
use std::path::PathBuf;

/// インストール済みプラグインの一覧を取得
///
/// キャッシュディレクトリをスキャンし、有効なプラグインの一覧を返す。
///
/// # Arguments
///
/// * `cache` - Package cache accessor used to enumerate installed plugins.
pub fn list_installed_plugins(cache: &dyn PackageCacheAccess) -> Result<Vec<InstalledPlugin>> {
    // デプロイ済みプラグイン集合を事前取得（パフォーマンス改善）
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let deployed = list_all_placed(&project_root);

    let plugins = list_installed(cache)?
        .into_iter()
        .map(|pkg| {
            let name = pkg.manifest().name.clone();
            let plugin = Plugin::new(pkg.manifest().clone(), pkg.path().to_path_buf());
            let marketplace_str = pkg.marketplace().unwrap_or("github");
            let ops_key = pkg.cache_key().unwrap_or(&name);
            let enabled = meta::is_enabled(pkg.path(), marketplace_str, ops_key, &deployed);

            InstalledPlugin::from_cached_package(
                plugin,
                pkg.cache_key().map(str::to_string),
                pkg.marketplace().map(str::to_string),
                enabled,
            )
        })
        .collect();

    Ok(plugins)
}

#[cfg(test)]
#[path = "catalog_test.rs"]
mod tests;
