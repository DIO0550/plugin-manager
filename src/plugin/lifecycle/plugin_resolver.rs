//! プラグイン解決ヘルパー（display name fallback 専用）
//!
//! `update_plugin` 入口では先に **`cache_id` 完全一致** を試し、
//! ヒットしなかったときだけ本 helper にフォールバックする。

use crate::error::{PlmError, Result};
use crate::plugin::cache::{CachedPackage, PackageCacheAccess};
use crate::plugin::meta;
use crate::plugin::PluginMeta;

/// 解決された 1 件分のプラグイン情報
#[derive(Debug)]
pub struct ResolvedPlugin {
    pub marketplace: Option<String>,
    pub cache_id: String,
    pub display_name: String,
    pub package: CachedPackage,
    /// 現在キャッシュされている `.plm-meta.json` の内容
    pub package_meta: PluginMeta,
}

/// プラグイン表示名（`CachedPackage.name`）でプラグインを resolve する
///
/// 解決方針 (案 A):
/// - `marketplace_hint` が `Some(m)` のときは走査時点で `marketplace == m` のエントリのみ候補にする
/// - `None` のときは全 marketplace を候補にし、複数件マッチで `AmbiguousPluginName` を返す
/// - 0 件 → `Ok(None)`
/// - 1 件 → `Ok(Some(ResolvedPlugin))`
/// - 2 件以上 → `Err(PlmError::AmbiguousPluginName { name, candidates })`
///
/// # Arguments
///
/// * `cache` - Package cache accessor used to enumerate stored plugins.
/// * `plugin_name` - Display name (`CachedPackage.name`) being searched.
/// * `marketplace_hint` - Optional marketplace name to disambiguate candidates.
pub fn find_by_plugin_name(
    cache: &dyn PackageCacheAccess,
    plugin_name: &str,
    marketplace_hint: Option<&str>,
) -> Result<Option<ResolvedPlugin>> {
    let mut matches: Vec<ResolvedPlugin> = Vec::new();
    for (marketplace, cache_id) in cache.list()? {
        if let Some(hint) = marketplace_hint {
            if marketplace.as_deref() != Some(hint) {
                continue;
            }
        }
        let pkg = match cache.load_package(marketplace.as_deref(), &cache_id) {
            Ok(p) => p,
            Err(_) => continue,
        };
        if pkg.name == plugin_name {
            let plugin_path = cache.plugin_path(marketplace.as_deref(), &cache_id);
            let package_meta = meta::load_meta(&plugin_path).unwrap_or_default();
            matches.push(ResolvedPlugin {
                marketplace,
                cache_id: cache_id.clone(),
                display_name: pkg.name.clone(),
                package: pkg,
                package_meta,
            });
        }
    }

    match matches.len() {
        0 => Ok(None),
        1 => Ok(Some(matches.into_iter().next().unwrap())),
        _ => {
            let mut candidates: Vec<String> = matches
                .iter()
                .map(|m| match &m.marketplace {
                    Some(mk) => format!("{}@{}", m.display_name, mk),
                    None => m.display_name.clone(),
                })
                .collect();
            candidates.sort();
            Err(PlmError::AmbiguousPluginName {
                name: plugin_name.to_string(),
                candidates,
            })
        }
    }
}

#[cfg(test)]
#[path = "plugin_resolver_test.rs"]
mod tests;
