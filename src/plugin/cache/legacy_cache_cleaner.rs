//! 旧キャッシュレイアウトのピンポイント掃除
//!
//! 「`plugins.len() > 1` の marketplace で、`<repo.name()>` ディレクトリが存在し、
//! かつそれが現 `plugins[].name` のどれにも一致しない」場合のみ、
//! 旧 `<repo.name()>` ディレクトリ「だけ」を削除する。

use crate::error::Result;
use crate::marketplace::MarketplaceCache;
use crate::plugin::cache::PackageCacheAccess;
use std::collections::HashSet;

/// 旧キャッシュレイアウトを安全に掃除するヘルパー
pub struct LegacyCacheCleaner;

impl LegacyCacheCleaner {
    /// 旧バグ固有のレイアウトを検出し、旧 `<repo.name()>` ディレクトリだけを削除する
    ///
    /// 副作用は idempotent。
    ///
    /// 検出条件（すべて満たすときのみ削除）:
    /// 1. `mp_cache.plugins.len() > 1`
    /// 2. `<market>/<repo.name()>/` が物理的に存在する
    /// 3. `<repo.name()>` が `plugins[].name` のどれにも一致しない
    ///
    /// repo 名のパス安全性は `MarketplaceSourceRef` の不変条件として保証済み。
    ///
    /// # Arguments
    ///
    /// * `cache` - Package cache accessor for has/remove operations.
    /// * `marketplace_name` - Marketplace name being inspected.
    /// * `mp_cache` - Cached marketplace manifest.
    ///
    /// # Returns
    ///
    /// `Ok(true)` when the legacy directory was removed, `Ok(false)` when no-op.
    pub fn clean_if_legacy(
        cache: &dyn PackageCacheAccess,
        marketplace_name: &str,
        mp_cache: &MarketplaceCache,
    ) -> Result<bool> {
        if mp_cache.plugins.len() <= 1 {
            return Ok(false);
        }

        let legacy_dir_name = mp_cache.source.name();

        let plugin_names: HashSet<&str> =
            mp_cache.plugins.iter().map(|p| p.name.as_str()).collect();
        if plugin_names.contains(legacy_dir_name) {
            return Ok(false);
        }

        if !cache.has_marketplace_entry(marketplace_name, legacy_dir_name)? {
            return Ok(false);
        }

        cache.remove_marketplace_entry(marketplace_name, legacy_dir_name)?;
        Ok(true)
    }
}

#[cfg(test)]
#[path = "legacy_cache_cleaner_test.rs"]
mod tests;
