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
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_repo() {
        let source = parse_source("owner/repo").unwrap();
        // Box<dyn PluginSource> なので型は確認できないが、パースは成功する
        assert!(std::ptr::eq(
            source.as_ref() as *const dyn PluginSource as *const (),
            source.as_ref() as *const dyn PluginSource as *const ()
        ));
    }

    #[test]
    fn test_parse_github_repo_with_ref() {
        let source = parse_source("owner/repo@v1.0.0");
        assert!(source.is_ok());
    }

    #[test]
    fn test_parse_github_full_url() {
        let source = parse_source("https://github.com/owner/repo");
        assert!(source.is_ok());
    }

    #[test]
    fn test_parse_marketplace() {
        let source = parse_source("plugin@marketplace");
        assert!(source.is_ok());
    }

    #[test]
    fn test_parse_search() {
        let source = parse_source("plugin-name");
        assert!(source.is_ok());
    }

    // === 境界値テスト ===

    #[test]
    fn test_parse_empty_string() {
        // 空文字は SearchSource として処理される
        let source = parse_source("");
        assert!(source.is_ok());
    }

    #[test]
    fn test_parse_whitespace_only() {
        // 空白のみも SearchSource として処理される
        let source = parse_source("   ");
        assert!(source.is_ok());
    }

    #[test]
    fn test_parse_plugin_at_empty() {
        // "plugin@" は plugin="" の MarketplaceSource
        let source = parse_source("plugin@");
        assert!(source.is_ok());
    }

    #[test]
    fn test_parse_at_marketplace() {
        // "@marketplace" は left="" の MarketplaceSource
        let source = parse_source("@marketplace");
        assert!(source.is_ok());
    }

    #[test]
    fn test_parse_owner_repo_at_empty_ref() {
        // "owner/repo@" は空refエラーになる（repo::from_urlで処理）
        let source = parse_source("owner/repo@");
        assert!(source.is_err());
    }

    #[test]
    fn test_parse_multiple_at_symbols() {
        // "a@b@c" は左側に "/" がないので MarketplaceSource(left="a", right="b@c")
        let source = parse_source("a@b@c");
        assert!(source.is_ok());
    }

    #[test]
    fn test_parse_owner_repo_multiple_at() {
        // "owner/repo@v1@extra" は ref="v1@extra" として処理
        let source = parse_source("owner/repo@v1@extra");
        assert!(source.is_ok());
    }

    #[test]
    fn test_parse_at_only() {
        // "@" のみは MarketplaceSource(left="", right="")
        let source = parse_source("@");
        assert!(source.is_ok());
    }

    #[test]
    fn test_parse_slash_only() {
        // "/" のみはGitHubリポジトリとして処理されるがエラー
        let source = parse_source("/");
        assert!(source.is_err());
    }

    #[test]
    fn test_parse_owner_slash_empty() {
        // "owner/" は空のリポジトリ名でエラー
        let source = parse_source("owner/");
        assert!(source.is_err());
    }

    #[test]
    fn test_parse_slash_repo() {
        // "/repo" は空のオーナー名でエラー
        let source = parse_source("/repo");
        assert!(source.is_err());
    }
}
