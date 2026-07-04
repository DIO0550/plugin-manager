//! マーケットプレイスソース参照の値オブジェクト
//!
//! 「マーケットプレイスの取得元 GitHub リポジトリへの参照」を単一の型で表現する。
//! 保存（内部）形式は `github:owner/repo` に正規化し、読み込みは
//! プレフィックスなし（`owner/repo`）や URL 形式も受け付ける。

use crate::error::PlmError;
use crate::host::HostKind;
use crate::repo::{self, Repo};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// 内部（保存）形式のプレフィックス
const GITHUB_PREFIX: &str = "github:";

/// マーケットプレイスの取得元 GitHub リポジトリへの参照
///
/// # 不変条件
///
/// - `owner` / `name` は空でない
/// - `owner` / `name` はパス安全（`.` / `..` / パス区切りを含まない）
///
/// # 形式
///
/// - 内部（保存）形式: `github:owner/repo`（`Display` / `Serialize`）
/// - 表示形式: `owner/repo`（`full_name()`）
/// - パース: `github:` プレフィックスあり/なし、URL 形式のいずれも受理
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketplaceSourceRef {
    owner: String,
    name: String,
}

impl MarketplaceSourceRef {
    /// `Repo` から参照を作成する（git ref は保持しない）
    ///
    /// # Arguments
    ///
    /// * `repo` - Source repository whose owner / name are captured.
    pub fn from_repo(repo: &Repo) -> Self {
        Self {
            owner: repo.owner().to_string(),
            name: repo.name().to_string(),
        }
    }

    /// オーナー名
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// リポジトリ名
    pub fn name(&self) -> &str {
        &self.name
    }

    /// ユーザー表示用の `owner/repo` 形式
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    /// GitHub の `Repo` へ変換する（git ref なし）
    pub fn to_repo(&self) -> Repo {
        self.to_repo_with_ref(None)
    }

    /// git ref を指定して `Repo` へ変換する
    ///
    /// マーケットプレイスソースは ref を持たないため、archive 取得などで
    /// ref を確定させたい場合に呼び出し側から渡す。
    ///
    /// # Arguments
    ///
    /// * `git_ref` - Git reference to embed into the resulting `Repo`.
    pub fn to_repo_with_ref(&self, git_ref: Option<String>) -> Repo {
        Repo::new(
            HostKind::GitHub,
            self.owner.clone(),
            self.name.clone(),
            git_ref,
        )
    }
}

impl FromStr for MarketplaceSourceRef {
    type Err = PlmError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let stripped = s.strip_prefix(GITHUB_PREFIX).unwrap_or(s);
        // 正規パーサー（repo::from_url）へ委譲し、パースを一元化する
        let parsed = repo::from_url(stripped)?;

        // キャッシュディレクトリ名等に使われるため、パス安全性を保証する
        for part in [parsed.owner(), parsed.name()] {
            if part == "." || part == ".." || part.contains('\\') {
                return Err(PlmError::InvalidSource(format!(
                    "path-unsafe repository reference: {}",
                    s
                )));
            }
        }

        Ok(Self::from_repo(&parsed))
    }
}

impl fmt::Display for MarketplaceSourceRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}/{}", GITHUB_PREFIX, self.owner, self.name)
    }
}

impl Serialize for MarketplaceSourceRef {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for MarketplaceSourceRef {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
#[path = "source_ref_test.rs"]
mod tests;
