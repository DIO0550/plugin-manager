//! プラグインカタログ
//!
//! インストール済みプラグインの一覧取得ユースケースを提供する。

use crate::component::{ComponentKind, Scope};
use crate::error::Result;
use crate::plugin::{meta, InstalledPlugin, MarketplaceContent, PackageCacheAccess, Plugin};
use crate::scan::list_placed_plugins;
use crate::target::all_targets;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// cache.list() の生タプルからのマッピング型（変更吸収層）
struct PluginCacheKey {
    marketplace: Option<String>,
    name: String,
}

impl From<(Option<String>, String)> for PluginCacheKey {
    fn from((marketplace, name): (Option<String>, String)) -> Self {
        Self { marketplace, name }
    }
}

/// キャッシュ内のマーケットプレイスパッケージを列挙
pub(crate) fn list_installed(cache: &dyn PackageCacheAccess) -> Result<Vec<MarketplaceContent>> {
    let packages = cache
        .list()?
        .into_iter()
        .map(PluginCacheKey::from)
        .filter(|key| !key.name.starts_with('.'))
        .filter_map(|key| {
            cache
                .load_package(key.marketplace.as_deref(), &key.name)
                .ok()
                .map(MarketplaceContent::from)
        })
        .collect();
    Ok(packages)
}

/// インストール済みプラグインの一覧を取得
///
/// キャッシュディレクトリをスキャンし、有効なプラグインの一覧を返す。
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

/// 全ターゲットから配置済みプラグインを収集
///
/// 全ターゲット・全コンポーネント種別のデプロイ済みコンポーネントを走査し、
/// プラグインの (marketplace, plugin_name) の集合を返す。
///
/// `plugin_info.rs` からも使用されるため `pub(crate)` で公開。
pub(crate) fn list_all_placed(project_root: &Path) -> HashSet<(String, String)> {
    let targets = all_targets();
    let mut all_items = Vec::new();

    for target in &targets {
        for kind in ComponentKind::all() {
            if !target.supports(*kind) {
                continue;
            }
            // エラー時は黙殺（保守的に deployed とみなさない）
            if let Ok(placed) = target.list_placed(*kind, Scope::Project, project_root) {
                all_items.extend(placed);
            }
        }
    }

    list_placed_plugins(&all_items)
}

#[cfg(test)]
#[path = "plugin_catalog_test.rs"]
mod tests;
