//! キャッシュ済みパッケージデータ（外部データ型）
//!
//! ダウンロード結果またはキャッシュ読み出し結果を保持する。
//! ドメインロジック（コンポーネントスキャン・パス解決）は `MarketplacePackage` に委譲。

use crate::plugin::PluginManifest;
use std::path::PathBuf;

/// キャッシュ上のプラグインデータ（外部データ型）
///
/// ダウンロード結果またはキャッシュ読み出し結果を保持する。
/// ドメインロジックは `MarketplacePackage` に委譲。
///
/// **`name` フィールドの意味**: 経路により異なる値が入る（新規ダウンロード時は `manifest.name`、
/// キャッシュヒット時やキャッシュ読み出し時はキャッシュキー名）。
/// この不整合の解消（`cache_key` フィールド導入）は次ステップで対応。
#[derive(Debug, Clone)]
pub struct CachedPackage {
    pub name: String,
    /// マーケットプレイス名（marketplace経由の場合）
    /// None の場合は直接GitHubからインストール
    pub marketplace: Option<String>,
    pub path: PathBuf,
    pub manifest: PluginManifest,
    pub git_ref: String,
    pub commit_sha: String,
}

impl CachedPackage {
    /// プラグインのバージョンを取得
    pub fn version(&self) -> &str {
        &self.manifest.version
    }

    /// プラグインの説明を取得
    pub fn description(&self) -> Option<&str> {
        self.manifest.description.as_deref()
    }
}
