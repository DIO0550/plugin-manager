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
    let packages = list_installed(cache)?;

    // 「プラグイン数 × 配置済みコンポーネント数」の線形走査を避けるため、
    // 既知 plugin_name 集合と deployed から「配置済みコンポーネントを 1 件以上
    // 持つ plugin_name 集合」を 1 度だけ構築し、各 is_enabled 判定を O(1) に。
    let known_plugin_names: HashSet<String> = packages
        .iter()
        .map(|pkg| pkg.manifest().name.clone())
        .collect();
    let deployed_plugins = meta::build_deployed_plugin_set(&deployed, &known_plugin_names);

    let plugins = packages
        .into_iter()
        .filter_map(|pkg| {
            let name = pkg.manifest().name.clone();
            let marketplace_str = pkg.marketplace().unwrap_or("github");
            let ops_key = pkg.id().unwrap_or(&name);
            let origin = PluginOrigin::from_marketplace(marketplace_str, ops_key);
            let plugin =
                Plugin::new(pkg.manifest().clone(), pkg.path().to_path_buf(), origin).ok()?;
            // flatten_name の prefix は manifest.name に基づくため
            // is_enabled_indexed には manifest.name を渡す。
            let enabled = meta::is_enabled_indexed(pkg.path(), name.as_str(), &deployed_plugins);

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
