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
pub struct LegacyLayoutSweeper;

impl LegacyLayoutSweeper {
    /// 旧バグ固有のレイアウトを検出し、旧 `<repo.name()>` ディレクトリだけを削除する
    ///
    /// 副作用は idempotent。
    ///
    /// 検出条件（すべて満たすときのみ削除）:
    /// 1. `mp_cache.plugins.len() > 1`
    /// 2. `mp_cache.source` から repo.name() を抽出できる
    /// 3. `<market>/<repo.name()>/` が物理的に存在する
    /// 4. `<repo.name()>` が `plugins[].name` のどれにも一致しない
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
    pub fn sweep_if_legacy(
        cache: &dyn PackageCacheAccess,
        marketplace_name: &str,
        mp_cache: &MarketplaceCache,
    ) -> Result<bool> {
        if mp_cache.plugins.len() <= 1 {
            return Ok(false);
        }

        let Some(legacy_dir_name) = extract_repo_name(&mp_cache.source) else {
            return Ok(false);
        };

        let plugin_names: HashSet<&str> =
            mp_cache.plugins.iter().map(|p| p.name.as_str()).collect();
        if plugin_names.contains(legacy_dir_name.as_str()) {
            return Ok(false);
        }

        if !cache.has_marketplace_entry(marketplace_name, &legacy_dir_name)? {
            return Ok(false);
        }

        cache.remove_marketplace_entry(marketplace_name, &legacy_dir_name)?;
        Ok(true)
    }
}

/// `"github:owner/repo"` → `Some("repo")`
///
/// パストラバーサル防止のため、抽出した名前が空 / `.` / `..` /
/// パス区切り（`/`, `\`）を含む場合は `None` を返し、sweep を無効化する。
///
/// # Arguments
///
/// * `source` - Marketplace source string in `github:owner/repo` form.
fn extract_repo_name(source: &str) -> Option<String> {
    let stripped = source.strip_prefix("github:").unwrap_or(source);
    let candidate = stripped.rsplit('/').next()?;
    if candidate.is_empty()
        || candidate == "."
        || candidate == ".."
        || candidate.contains('/')
        || candidate.contains('\\')
    {
        return None;
    }
    Some(candidate.to_string())
}

#[cfg(test)]
#[path = "legacy_layout_sweeper_test.rs"]
mod tests;
