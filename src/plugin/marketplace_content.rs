//! マーケットプレイスコンテンツ（内部ドメイン型）
//!
//! コンポーネント群（skills, agents, commands, instructions, hooks）を含む
//! マーケットプレイスのパッケージ。コンポーネントスキャン・パス解決を担う。

use crate::component::{AgentFormat, CommandFormat, Component};
use crate::marketplace::MarketplaceManifest;
use crate::plugin::PluginManifest;
use std::path::{Path, PathBuf};

use super::cached_package::CachedPackage;
use super::plugin_content::Plugin;

/// マーケットプレイスコンテンツ（内部ドメイン型）
///
/// コンポーネント群（skills, agents, commands, instructions, hooks）を含む
/// マーケットプレイスのパッケージ。コンポーネントスキャン・パス解決を担う。
#[derive(Debug, Clone)]
pub struct MarketplaceContent {
    /// キャッシュディレクトリ名（ファイル操作用、CachedPackage から伝搬）
    pub(crate) cache_key: Option<String>,
    pub(crate) marketplace: Option<String>,
    pub(crate) marketplace_manifest: Option<MarketplaceManifest>,
    /// 代表プラグイン（型レベルで非空を保証）
    primary: Plugin,
    /// 追加プラグイン（将来の複数プラグイン対応用）
    extra_plugins: Vec<Plugin>,
}

impl MarketplaceContent {
    /// プラグイン名を取得
    pub fn name(&self) -> &str {
        self.primary.name()
    }

    /// キャッシュキーを取得
    pub fn cache_key(&self) -> Option<&str> {
        self.cache_key.as_deref()
    }

    /// マーケットプレイス名を取得
    pub fn marketplace(&self) -> Option<&str> {
        self.marketplace.as_deref()
    }

    /// パスを取得
    pub fn path(&self) -> &Path {
        self.primary.path()
    }

    /// マニフェストを取得
    pub fn manifest(&self) -> &PluginManifest {
        self.primary.manifest()
    }

    /// マーケットプレイスマニフェストを取得
    pub fn marketplace_manifest(&self) -> Option<&MarketplaceManifest> {
        self.marketplace_manifest.as_ref()
    }

    /// Command コンポーネントのソース形式を取得
    pub fn command_format(&self) -> CommandFormat {
        match self.marketplace.as_deref() {
            Some("claude") => CommandFormat::ClaudeCode,
            Some("copilot") => CommandFormat::Copilot,
            Some("codex") => CommandFormat::Codex,
            _ => CommandFormat::ClaudeCode,
        }
    }

    /// Agent コンポーネントのソース形式を取得
    pub fn agent_format(&self) -> AgentFormat {
        match self.marketplace.as_deref() {
            Some("claude") => AgentFormat::ClaudeCode,
            Some("copilot") => AgentFormat::Copilot,
            Some("codex") => AgentFormat::Codex,
            _ => AgentFormat::ClaudeCode,
        }
    }

    // =========================================================================
    // パス解決メソッド（Plugin に委譲）
    // =========================================================================

    /// スキルディレクトリのパスを解決
    pub fn skills_dir(&self) -> PathBuf {
        self.primary.skills_dir()
    }

    /// エージェントディレクトリのパスを解決
    pub fn agents_dir(&self) -> PathBuf {
        self.primary.agents_dir()
    }

    /// コマンドディレクトリのパスを解決
    pub fn commands_dir(&self) -> PathBuf {
        self.primary.commands_dir()
    }

    /// インストラクションパスを解決
    pub fn instructions_path(&self) -> PathBuf {
        self.primary.instructions_path()
    }

    /// フックディレクトリのパスを解決
    pub fn hooks_dir(&self) -> PathBuf {
        self.primary.hooks_dir()
    }

    // =========================================================================
    // スキャンメソッド
    // =========================================================================

    /// プラグイン内のコンポーネントを取得
    pub fn components(&self) -> Vec<Component> {
        std::iter::once(&self.primary)
            .chain(self.extra_plugins.iter())
            .flat_map(|p| p.components())
            .cloned()
            .collect()
    }
}

impl From<CachedPackage> for MarketplaceContent {
    fn from(cached: CachedPackage) -> Self {
        let primary = Plugin::new(cached.manifest, cached.path);
        Self {
            cache_key: cached.cache_key,
            marketplace: cached.marketplace,
            marketplace_manifest: cached.marketplace_manifest,
            primary,
            extra_plugins: Vec::new(),
        }
    }
}

impl From<&CachedPackage> for MarketplaceContent {
    fn from(cached: &CachedPackage) -> Self {
        let primary = Plugin::new(cached.manifest.clone(), cached.path.clone());
        Self {
            cache_key: cached.cache_key.clone(),
            marketplace: cached.marketplace.clone(),
            marketplace_manifest: cached.marketplace_manifest.clone(),
            primary,
            extra_plugins: Vec::new(),
        }
    }
}

#[cfg(test)]
#[path = "marketplace_content_test.rs"]
mod tests;
