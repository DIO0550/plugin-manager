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

    /// スキルが含まれているか
    pub fn has_skills(&self) -> bool {
        self.manifest.has_skills()
    }

    /// スキルのパスを取得
    pub fn skills(&self) -> Option<&str> {
        self.manifest.skills.as_deref()
    }

    /// エージェントが含まれているか
    pub fn has_agents(&self) -> bool {
        self.manifest.has_agents()
    }

    /// エージェントのパスを取得
    pub fn agents(&self) -> Option<&str> {
        self.manifest.agents.as_deref()
    }

    /// コマンドが含まれているか
    pub fn has_commands(&self) -> bool {
        self.manifest.has_commands()
    }

    /// コマンドのパスを取得
    pub fn commands(&self) -> Option<&str> {
        self.manifest.commands.as_deref()
    }

    /// インストラクションが含まれているか
    pub fn has_instructions(&self) -> bool {
        self.manifest.has_instructions()
    }

    /// インストラクションのパスを取得
    pub fn instructions(&self) -> Option<&str> {
        self.manifest.instructions.as_deref()
    }

    /// フックが含まれているか
    pub fn has_hooks(&self) -> bool {
        self.manifest.hooks.is_some()
    }

    /// フックのパスを取得
    pub fn hooks(&self) -> Option<&str> {
        self.manifest.hooks.as_deref()
    }
}
