//! `--marketplace` オプション用の共通 Args 部品。

use clap::Args as ClapArgs;

#[derive(Debug, Clone, ClapArgs)]
pub struct MarketplaceArgs {
    /// Marketplace identifier (defaults to "github" if omitted)
    #[arg(long, short = 'm')]
    pub marketplace: Option<String>,
}

impl MarketplaceArgs {
    /// run 側で使うフォールバック付きアクセッサ。
    /// 既存 enable/disable の `default_value = "github"` 互換を保つために提供する。
    pub fn marketplace_or_default(&self) -> &str {
        self.marketplace.as_deref().unwrap_or("github")
    }
}

#[cfg(test)]
#[path = "marketplace_test.rs"]
mod tests;
