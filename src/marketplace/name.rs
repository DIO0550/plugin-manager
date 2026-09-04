//! マーケットプレイス登録名の値オブジェクト。

use super::config::normalize_name;
use std::fmt;

/// 正規化・検証済みのマーケットプレイス登録名。
///
/// 外部入力は [`MarketplaceName::parse`] でこの型へ変換し、参照処理には
/// 小文字化済みの値だけを渡す。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MarketplaceName(String);

impl MarketplaceName {
    /// 生のマーケットプレイス名を小文字化し、使用可能な文字を検証する。
    ///
    /// # Arguments
    ///
    /// * `name` - CLI やソース指定から受け取ったマーケットプレイス名。
    pub fn parse(name: &str) -> Result<Self, String> {
        normalize_name(name).map(Self)
    }

    /// 正規化済みの名前を文字列として返す。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MarketplaceName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
#[path = "name_test.rs"]
mod tests;
