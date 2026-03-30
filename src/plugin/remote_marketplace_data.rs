//! リモートマーケットプレイスデータ（外部データ型）
//!
//! リモートから取得したマーケットプレイスデータを保持する。
//! ドメインロジック（コンポーネントスキャン・パス解決）は `MarketplacePackage` に委譲。

use crate::plugin::PluginManifest;
use std::path::PathBuf;

/// リモートから取得したマーケットプレイスデータ（外部データ型）
///
/// GitHub からダウンロード・パースした結果を保持する。
/// ドメインロジックは `MarketplacePackage` に委譲。
#[derive(Debug, Clone)]
pub struct RemoteMarketplaceData {
    pub name: String,
    /// マーケットプレイス名（marketplace経由の場合）
    /// None の場合は直接GitHubからインストール
    pub marketplace: Option<String>,
    pub path: PathBuf,
    pub manifest: PluginManifest,
    pub git_ref: String,
    pub commit_sha: String,
}

impl RemoteMarketplaceData {
    /// プラグインのバージョンを取得
    pub fn version(&self) -> &str {
        &self.manifest.version
    }

    /// プラグインの説明を取得
    pub fn description(&self) -> Option<&str> {
        self.manifest.description.as_deref()
    }
}
