//! プラグインソースの解決とダウンロード
//!
//! ## 使い方
//!
//! ```ignore
//! let source = parse_source("owner/repo")?;
//! let cache = PackageCache::new()?;
//! let plugin = source.download(&cache, false).await?;
//! ```

mod github_source;
mod marketplace_source;
mod search_source;

pub use github_source::GitHubSource;
pub use marketplace_source::MarketplaceSource;
pub use search_source::SearchSource;

use crate::error::Result;
use crate::plugin::{CachedPackage, PackageCacheAccess};
use crate::repo;
use std::future::Future;
use std::pin::Pin;

/// プラグインソースの抽象化
///
/// 各ソースタイプ（GitHub, Marketplace, Search）がこの trait を実装する。
/// 使う側は具体的なソースタイプを意識せず `download()` を呼ぶだけ。
pub trait PackageSource: Send + Sync {
    /// プラグインをダウンロードする
    ///
    /// # Arguments
    /// * `cache` - Package cache accessor used to read or store the downloaded package.
    /// * `force` - When `true`, bypass any cached copy and re-download.
    fn download<'a>(
        &'a self,
        cache: &'a dyn PackageCacheAccess,
        force: bool,
    ) -> Pin<Box<dyn Future<Output = Result<CachedPackage>> + Send + 'a>>;
}

/// 入力文字列をパースして適切な PackageSource を返す
///
/// # Arguments
/// * `input` - Source specifier such as `owner/repo`, `owner/repo@ref`, `plugin@marketplace`, or a bare plugin name.
pub fn parse_source(input: &str) -> Result<Box<dyn PackageSource>> {
    if let Some((left, right)) = input.split_once('@') {
        if left.contains('/') {
            let repo = repo::from_url(input)?;
            return Ok(Box::new(GitHubSource::new(repo)));
        }

        return Ok(Box::new(MarketplaceSource::new(left, right)));
    }

    if input.contains('/') {
        let repo = repo::from_url(input)?;
        return Ok(Box::new(GitHubSource::new(repo)));
    }

    Ok(Box::new(SearchSource::new(input)))
}

#[cfg(test)]
#[path = "source_test.rs"]
mod tests;
