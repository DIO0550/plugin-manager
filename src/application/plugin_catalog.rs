//! プラグインカタログ
//!
//! インストール済みプラグインの一覧取得ユースケースを提供する。

use crate::component::{serialize_components, Component, ComponentKind, Scope};
use crate::error::Result;
use crate::plugin::{meta, MarketplaceContent, PackageCacheAccess, Plugin};
use crate::scan::list_placed_plugins;
use crate::target::all_targets;
use serde::Serialize;
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

/// プラグイン情報のサマリ（DTO）
#[derive(Debug, Clone, Serialize)]
pub struct PluginSummary {
    /// プラグイン名（表示用）
    pub name: String,
    /// キャッシュディレクトリ名（ファイル操作用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_key: Option<String>,
    /// マーケットプレイス名（マーケットプレイス経由の場合）
    pub marketplace: Option<String>,
    /// バージョン
    pub version: String,
    /// コンポーネント一覧
    #[serde(flatten, serialize_with = "serialize_components")]
    pub components: Vec<Component>,
    /// 有効状態（デプロイ先に配置されているか）
    pub enabled: bool,
}

impl PluginSummary {
    /// キャッシュディレクトリ名を返す（`cache_key` が `None` の場合は `name` にフォールバック）
    pub fn cache_key(&self) -> &str {
        crate::plugin::resolve_cache_key(self.cache_key.as_deref(), &self.name)
    }

    /// コンポーネント種別ごとの件数を取得（空でないもののみ）
    pub fn component_type_counts(&self) -> Vec<(ComponentKind, usize)> {
        ComponentKind::all()
            .iter()
            .filter_map(|&kind| {
                let count = self.components.iter().filter(|c| c.kind == kind).count();
                if count > 0 {
                    Some((kind, count))
                } else {
                    None
                }
            })
            .collect()
    }

    /// 特定種別のコンポーネント名一覧を取得
    pub fn component_names(&self, kind: ComponentKind) -> Vec<String> {
        self.components
            .iter()
            .filter(|c| c.kind == kind)
            .map(|c| c.name.clone())
            .collect()
    }
}

/// インストール済みプラグインの一覧を取得
///
/// キャッシュディレクトリをスキャンし、有効なプラグインの一覧を返す。
pub fn list_installed_plugins(cache: &dyn PackageCacheAccess) -> Result<Vec<PluginSummary>> {
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

            PluginSummary {
                name,
                cache_key: pkg.cache_key().map(str::to_string),
                marketplace: pkg.marketplace().map(str::to_string),
                version: pkg.manifest().version.clone(),
                components: plugin.components().to_vec(),
                enabled,
            }
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
