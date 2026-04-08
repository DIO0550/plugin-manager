//! キャッシュ済みパッケージデータ（外部データ型）
//!
//! ダウンロード結果またはキャッシュ読み出し結果を保持する。
//! ドメインロジック（コンポーネントスキャン・パス解決）は `MarketplaceContent` に委譲。

#[cfg(test)]
#[path = "cached_package_test.rs"]
mod tests;

use crate::marketplace::MarketplaceManifest;
use crate::plugin::PluginManifest;
use std::path::PathBuf;

/// git_ref / commit_sha が不明な場合のデフォルト値
pub const UNKNOWN_GIT_VALUE: &str = "unknown";

/// キャッシュ上のプラグインデータ（外部データ型）
///
/// ダウンロード結果またはキャッシュ読み出し結果を保持する。
/// ドメインロジックは `MarketplaceContent` に委譲。
///
/// **`name` フィールド**: 表示用のプラグイン名（`manifest.name`）。
/// **`cache_key` フィールド**: ファイル操作用のキャッシュディレクトリ名。
/// `cache_key` が `None` の場合は `name` にフォールバックする。
#[derive(Debug, Clone)]
pub struct CachedPackage {
    pub name: String,
    /// キャッシュディレクトリ名（ファイル操作用）
    /// `None` の場合は `name` にフォールバック
    pub cache_key: Option<String>,
    /// マーケットプレイス名（marketplace経由の場合）
    /// None の場合は直接GitHubからインストール
    pub marketplace: Option<String>,
    pub path: PathBuf,
    pub manifest: PluginManifest,
    pub git_ref: String,
    pub commit_sha: String,
    /// ダウンロード時のマーケットプレイスマニフェスト（ダウンロード直後のみ有効、キャッシュ再読込時は None）
    pub marketplace_manifest: Option<MarketplaceManifest>,
}

impl CachedPackage {
    /// キャッシュディレクトリ名を返す（`cache_key` が `None` の場合は `name` にフォールバック）
    pub fn cache_key(&self) -> &str {
        super::resolve_cache_key(self.cache_key.as_deref(), &self.name)
    }

    /// プラグインのバージョンを取得
    pub fn version(&self) -> &str {
        &self.manifest.version
    }

    /// プラグインの説明を取得
    pub fn description(&self) -> Option<&str> {
        self.manifest.description.as_deref()
    }
}
