//! `--marketplace` オプション用の共通 Args 部品。

use crate::marketplace::DEFAULT_MARKETPLACE;
use clap::Args as ClapArgs;

#[derive(Debug, Clone, ClapArgs)]
pub struct MarketplaceArgs {
    /// Marketplace identifier (defaults to "github" if omitted)
    #[arg(long, short = 'm')]
    pub marketplace: Option<String>,
}

impl MarketplaceArgs {
    /// CLI 境界でのデフォルト解決アクセッサ。
    ///
    /// 「未指定 = github」の解決はここで 1 回だけ行い、ドメイン層には
    /// 確定値を渡す。
    pub fn marketplace_or_default(&self) -> &str {
        self.marketplace.as_deref().unwrap_or(DEFAULT_MARKETPLACE)
    }
}

#[cfg(test)]
#[path = "marketplace_test.rs"]
mod tests;
