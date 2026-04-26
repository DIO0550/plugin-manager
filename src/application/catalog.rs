//! プラグインカタログ
//!
//! インストール済みプラグインの一覧取得ユースケースを提供する。

use crate::error::Result;
use crate::plugin::{list_installed, meta, InstalledPlugin, PackageCacheAccess, Plugin};
use crate::target::{list_all_placed, PluginOrigin};
use std::collections::HashSet;
use std::path::PathBuf;

/// インストール済みプラグインの一覧を取得
///
/// キャッシュディレクトリをスキャンし、有効なプラグインの一覧を返す。
///
/// # Arguments
///
/// * `cache` - インストール済みプラグインを列挙するためのパッケージキャッシュアクセサ
pub fn list_installed_plugins(cache: &dyn PackageCacheAccess) -> Result<Vec<InstalledPlugin>> {
    // デプロイ済みコンポーネントの flattened_name 集合を事前取得（パフォーマンス改善）
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let deployed: HashSet<String> = list_all_placed(&project_root);

    // 一覧取得経路ではスキャン失敗（重複検出など）は握りつぶして列挙を続行する。
    // TUI 経由でも呼ばれるため stderr への直接出力は避ける。
    // 厳密検証が必要な経路（install 等）は `Plugin::new` を直接呼ぶこと。
    let plugins = list_installed(cache)?
        .into_iter()
        .filter_map(|pkg| {
            let name = pkg.manifest().name.clone();
            let marketplace_str = pkg.marketplace().unwrap_or("github");
            let ops_key = pkg.id().unwrap_or(&name);
            let origin = PluginOrigin::from_marketplace(marketplace_str, ops_key);
            let plugin =
                Plugin::new(pkg.manifest().clone(), pkg.path().to_path_buf(), origin).ok()?;
            // flatten_name の prefix は manifest.name に基づくため
            // is_enabled には manifest.name を渡す。
            let enabled = meta::is_enabled(pkg.path(), marketplace_str, name.as_str(), &deployed);

            Some(InstalledPlugin::from_cached_package(
                plugin,
                pkg.id().map(str::to_string),
                pkg.marketplace().map(str::to_string),
                enabled,
            ))
        })
        .collect();

    Ok(plugins)
}

#[cfg(test)]
#[path = "catalog_test.rs"]
mod tests;
