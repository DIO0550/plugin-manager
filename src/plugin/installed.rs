//! インストール済みプラグイン DTO
//!
//! `Plugin`（manifest + path + components）を内部に所有し、
//! 起源情報（marketplace / install_id）とデプロイ状態（enabled）を追加で保持する。
//! serde 属性は持たず、wire format は commands 層が責任を持つ。

use crate::component::{Component, ComponentKind};
use crate::plugin::{Author, Plugin};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct InstalledPlugin {
    plugin: Plugin,
    install_id: Option<String>,
    marketplace: Option<String>,
    enabled: bool,
}

impl InstalledPlugin {
    /// Plugin（キャッシュ済みパッケージ）と起源情報から InstalledPlugin を構築する
    ///
    /// # Arguments
    ///
    /// * `plugin` - cached plugin data (manifest, path, components)
    /// * `install_id` - optional install identifier (falls back to `plugin.name()`)
    /// * `marketplace` - optional marketplace name of origin
    /// * `enabled` - whether the plugin is currently deployed
    pub(crate) fn from_cached_package(
        plugin: Plugin,
        install_id: Option<String>,
        marketplace: Option<String>,
        enabled: bool,
    ) -> Self {
        Self {
            plugin,
            install_id,
            marketplace,
            enabled,
        }
    }

    /// プラグイン名
    pub fn name(&self) -> &str {
        self.plugin.name()
    }

    /// バージョン
    pub fn version(&self) -> &str {
        &self.plugin.manifest().version
    }

    /// コンポーネント一覧
    pub fn components(&self) -> &[Component] {
        self.plugin.components()
    }

    /// マーケットプレイス名
    pub fn marketplace(&self) -> Option<&str> {
        self.marketplace.as_deref()
    }

    /// 有効状態（デプロイ先に配置されているか）
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// 内部的な有効状態の設定（TUI からの状態更新用）
    ///
    /// # Arguments
    ///
    /// * `enabled` - new enabled state to assign
    pub(crate) fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// インストール識別子（`install_id` が `None` の場合は `name` にフォールバック）
    pub fn install_id(&self) -> &str {
        crate::plugin::resolve_cache_key(self.install_id.as_deref(), self.plugin.name())
    }

    /// コンポーネント種別ごとの件数を取得（空でないもののみ）
    pub fn component_type_counts(&self) -> Vec<(ComponentKind, usize)> {
        ComponentKind::all()
            .iter()
            .filter_map(|&kind| {
                let count = self.components().iter().filter(|c| c.kind == kind).count();
                (count > 0).then_some((kind, count))
            })
            .collect()
    }

    /// 特定種別のコンポーネント名一覧を取得
    ///
    /// # Arguments
    ///
    /// * `kind` - component kind to filter by
    pub fn component_names(&self, kind: ComponentKind) -> Vec<String> {
        self.components()
            .iter()
            .filter(|c| c.kind == kind)
            .map(|c| c.name.clone())
            .collect()
    }

    /// プラグインの説明文
    pub fn description(&self) -> Option<&str> {
        self.plugin.manifest().description.as_deref()
    }

    /// 作者情報を返す。
    /// 空 name の author は `None` として扱う（正規化責務をここに集約）。
    pub fn author(&self) -> Option<&Author> {
        self.plugin
            .manifest()
            .author
            .as_ref()
            .filter(|a| !a.name.is_empty())
    }

    /// キャッシュ上のプラグインパス
    pub fn cache_path(&self) -> &Path {
        self.plugin.path()
    }

    /// テスト専用: FS スキャンをバイパスして InstalledPlugin を構築する
    #[cfg(test)]
    pub(crate) fn new_for_test(
        name: &str,
        version: &str,
        components: Vec<Component>,
        install_id: Option<String>,
        marketplace: Option<String>,
        enabled: bool,
    ) -> Self {
        use crate::plugin::PluginManifest;
        use std::path::PathBuf;
        let manifest = PluginManifest {
            name: name.to_string(),
            version: version.to_string(),
            description: None,
            author: None,
            homepage: None,
            repository: None,
            license: None,
            keywords: None,
            commands: None,
            agents: None,
            skills: None,
            instructions: None,
            hooks: None,
            mcp_servers: None,
            lsp_servers: None,
            installed_at: None,
        };
        let plugin = Plugin::new_for_test(manifest, PathBuf::from("/test"), components);
        Self {
            plugin,
            install_id,
            marketplace,
            enabled,
        }
    }

    /// テスト専用: 任意の manifest / cache_path を指定して InstalledPlugin を構築する
    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_for_test_full(
        manifest: crate::plugin::PluginManifest,
        cache_path: std::path::PathBuf,
        components: Vec<Component>,
        install_id: Option<String>,
        marketplace: Option<String>,
        enabled: bool,
    ) -> Self {
        let plugin = Plugin::new_for_test(manifest, cache_path, components);
        Self {
            plugin,
            install_id,
            marketplace,
            enabled,
        }
    }
}

#[cfg(test)]
#[path = "installed_test.rs"]
mod tests;
