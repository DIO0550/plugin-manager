//! プラグインソースの解決とダウンロード
//!
//! ## 使い方
//!
//! ```ignore
//! let source = parse_source("owner/repo")?;
//! let plugin = source.download(false).await?;
//! ```

mod github_source;
mod marketplace_source;
mod search_source;

pub use github_source::GitHubSource;
pub use marketplace_source::MarketplaceSource;
pub use search_source::SearchSource;

use crate::error::Result;
use crate::plugin::CachedPlugin;
use crate::repo;
use std::future::Future;
use std::pin::Pin;

/// プラグインソースの抽象化
///
/// 各ソースタイプ（GitHub, Marketplace, Search）がこの trait を実装する。
/// 使う側は具体的なソースタイプを意識せず `download()` を呼ぶだけ。
pub trait PluginSource: Send + Sync {
    /// プラグインをダウンロードする
    fn download(&self, force: bool) -> Pin<Box<dyn Future<Output = Result<CachedPlugin>> + Send + '_>>;
}

/// 入力文字列をパースして適切な PluginSource を返す
pub fn parse_source(input: &str) -> Result<Box<dyn PluginSource>> {
    // "@" を含む場合
    if let Some((left, right)) = input.split_once('@') {
        // "owner/repo@ref" の場合（Gitリポジトリ）
        if left.contains('/') {
            let repo = repo::from_url(input)?;
            return Ok(Box::new(GitHubSource::new(repo)));
        }

        // "plugin@marketplace" の場合
        return Ok(Box::new(MarketplaceSource::new(left, right)));
    }

    // "/" を含む場合はGitリポジトリ
    if input.contains('/') {
        let repo = repo::from_url(input)?;
        return Ok(Box::new(GitHubSource::new(repo)));
    }

    // それ以外はMarketplace検索
    Ok(Box::new(SearchSource::new(input)))
}

#[cfg(test)]
#[path = "source_test.rs"]
mod tests;
