//! 直接 GitHub インストールのキャッシュ ID 値オブジェクト
//!
//! 直接 GitHub からインストールしたプラグインのキャッシュディレクトリ名
//! （`{owner}--{repo}` 形式）のエンコード・デコードを一元化する。

use crate::repo::Repo;
use std::fmt;

/// owner と repo の区切り文字
///
/// GitHub のユーザー名・組織名は連続ハイフンを含められないため、
/// 最初に現れる `--` が常に owner / repo の境界になる。
const SEPARATOR: &str = "--";

/// 直接 GitHub インストールのキャッシュ ID（`{owner}--{repo}` 形式）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubCacheId(String);

impl GithubCacheId {
    /// `Repo` からキャッシュ ID を生成する
    ///
    /// # Arguments
    ///
    /// * `repo` - Repository whose owner / name compose the cache ID.
    pub fn from_repo(repo: &Repo) -> Self {
        Self::from_parts(repo.owner(), repo.name())
    }

    /// owner / repo 名からキャッシュ ID を生成する
    ///
    /// # Arguments
    ///
    /// * `owner` - GitHub owner (user or organization).
    /// * `name` - GitHub repository name.
    pub fn from_parts(owner: &str, name: &str) -> Self {
        Self(format!("{}{}{}", owner, SEPARATOR, name))
    }

    /// 既存のキャッシュディレクトリ名（プラグイン名）をラップする
    ///
    /// `{owner}--{repo}` 形式でない名前も受け付ける（その場合 `parts()` は `None`）。
    ///
    /// # Arguments
    ///
    /// * `name` - Cache directory name to interpret as a cache ID.
    pub fn from_cache_name(name: &str) -> Self {
        Self(name.to_string())
    }

    /// `{owner}--{repo}` 形式なら `(owner, repo)` を返す
    ///
    /// 形式に合致しない（区切りがない、owner / repo が空）場合は `None`。
    pub fn parts(&self) -> Option<(&str, &str)> {
        let (owner, name) = self.0.split_once(SEPARATOR)?;
        if owner.is_empty() || name.is_empty() {
            return None;
        }
        Some((owner, name))
    }

    /// キャッシュディレクトリ名としての文字列表現
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 文字列へ変換する（キャッシュディレクトリ名）
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for GithubCacheId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
#[path = "github_cache_id_test.rs"]
mod tests;
