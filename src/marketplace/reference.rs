//! マーケットプレイス参照の値オブジェクト
//!
//! 「マーケットプレイス未指定 = GitHub 直接インストール」という規約を
//! `Option<&str>` の暗黙ルールではなく型で表現する。デフォルトの
//! ディレクトリ名（`"github"`）はここで一元管理する。

/// デフォルトマーケットプレイス（直接 GitHub インストール）のディレクトリ名。
///
/// キャッシュ階層 `<cache>/<marketplace>/<plugin>` やデプロイ階層の
/// marketplace セグメントとして使われる。ベア文字列 `"github"` を
/// 直接書かず、必ずこの定数（または [`MarketplaceRef`]）を経由すること。
pub const DEFAULT_MARKETPLACE: &str = "github";

/// マーケットプレイス参照
///
/// `None` / `Some("github")` の 2 表現に散っていた「デフォルト = GitHub」を
/// 1 つの正規形に畳み込む。生成時に正規化されるため、比較・分岐は
/// variant マッチだけで済む。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MarketplaceRef {
    /// デフォルト（直接 GitHub インストール）
    Github,
    /// 名前付きマーケットプレイス
    Named(String),
}

impl MarketplaceRef {
    /// マーケットプレイス名から生成する（`"github"` は `Github` に正規化）。
    ///
    /// # Arguments
    ///
    /// * `name` - Marketplace name.
    pub fn parse(name: &str) -> Self {
        if name == DEFAULT_MARKETPLACE {
            MarketplaceRef::Github
        } else {
            MarketplaceRef::Named(name.to_string())
        }
    }

    /// `Option<&str>`（`None` = デフォルト）から生成する。
    ///
    /// `cache.list()` などの既存 API が使う「`None` は github の意味」規約の
    /// 正規化入口。`Some("github")` も `Github` に畳み込む。
    ///
    /// # Arguments
    ///
    /// * `name` - Optional marketplace name; `None` means the default.
    pub fn from_option(name: Option<&str>) -> Self {
        match name {
            None => MarketplaceRef::Github,
            Some(n) => Self::parse(n),
        }
    }

    /// キャッシュ・デプロイ階層のディレクトリ名。
    pub fn dir_name(&self) -> &str {
        match self {
            MarketplaceRef::Github => DEFAULT_MARKETPLACE,
            MarketplaceRef::Named(name) => name,
        }
    }

    /// デフォルト（直接 GitHub）かどうか。
    pub fn is_github(&self) -> bool {
        matches!(self, MarketplaceRef::Github)
    }

    /// 名前付きマーケットプレイスの場合のみ名前を返す。
    ///
    /// `Github`（デフォルト）は `None`。「marketplace 経由か直接 GitHub か」で
    /// 経路が分かれる処理の分岐に使う。
    pub fn into_named(self) -> Option<String> {
        match self {
            MarketplaceRef::Github => None,
            MarketplaceRef::Named(name) => Some(name),
        }
    }
}

impl std::fmt::Display for MarketplaceRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.dir_name())
    }
}

#[cfg(test)]
#[path = "reference_test.rs"]
mod tests;
