//! プラグインカタログ
//!
//! インストール済みプラグインの一覧取得ユースケースを提供する。

use crate::component::{Component, ComponentKind, Scope};
use crate::error::Result;
use crate::plugin::{meta, MarketplaceContent, PackageCacheAccess, Plugin};
use crate::scan::list_placed_plugins;
use crate::target::all_targets;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// cache.list() の生タプルからのマッピング型（変更吸収層）
struct PluginCacheKey {
    marketplace: Option<String>,
    name: String,
}

impl From<(Option<String>, String)> for PluginCacheKey {
    fn from((marketplace, name): (Option<String>, String)) -> Self {
        Self { marketplace, name }
    }
}

/// キャッシュ内のマーケットプレイスパッケージを列挙
pub(crate) fn list_installed(cache: &dyn PackageCacheAccess) -> Result<Vec<MarketplaceContent>> {
    let packages = cache
        .list()?
        .into_iter()
        .map(PluginCacheKey::from)
        .filter(|key| !key.name.starts_with('.'))
        .filter_map(|key| {
            cache
                .load_package(key.marketplace.as_deref(), &key.name)
                .ok()
                .map(MarketplaceContent::from)
        })
        .collect();
    Ok(packages)
}

/// インストール済みプラグイン（`list_installed_plugins()` が返す DTO）
///
/// `Plugin`（manifest + path + components）を内部に所有し、
/// 起源情報（marketplace / install_id）とデプロイ状態（enabled）を追加で保持する。
#[derive(Debug, Clone)]
pub struct PluginSummary {
    plugin: Plugin,
    install_id: Option<String>,
    marketplace: Option<String>,
    enabled: bool,
}

impl PluginSummary {
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
                if count > 0 {
                    Some((kind, count))
                } else {
                    None
                }
            })
            .collect()
    }

    /// 特定種別のコンポーネント名一覧を取得
    pub fn component_names(&self, kind: ComponentKind) -> Vec<String> {
        self.components()
            .iter()
            .filter(|c| c.kind == kind)
            .map(|c| c.name.clone())
            .collect()
    }

    /// テスト専用: FS スキャンをバイパスして PluginSummary を構築する
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
}

impl Serialize for PluginSummary {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let field_count = if self.marketplace.is_some() { 6 } else { 5 };
        let mut state = serializer.serialize_struct("PluginSummary", field_count)?;
        state.serialize_field("name", self.name())?;
        state.serialize_field("version", self.version())?;
        state.serialize_field("install_id", self.install_id())?;
        if let Some(marketplace) = self.marketplace.as_deref() {
            state.serialize_field("marketplace", marketplace)?;
        }
        state.serialize_field("enabled", &self.enabled)?;
        state.serialize_field("components", &ComponentsByKind(self.components()))?;
        state.end()
    }
}

/// 手書き Serialize 用: components を kind 別にネストオブジェクトとしてシリアライズ
struct ComponentsByKind<'a>(&'a [Component]);

impl Serialize for ComponentsByKind<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Groups<'a> {
            skills: Vec<&'a str>,
            agents: Vec<&'a str>,
            commands: Vec<&'a str>,
            instructions: Vec<&'a str>,
            hooks: Vec<&'a str>,
        }

        let mut groups = Groups {
            skills: Vec::new(),
            agents: Vec::new(),
            commands: Vec::new(),
            instructions: Vec::new(),
            hooks: Vec::new(),
        };
        for c in self.0 {
            match c.kind {
                ComponentKind::Skill => groups.skills.push(c.name.as_str()),
                ComponentKind::Agent => groups.agents.push(c.name.as_str()),
                ComponentKind::Command => groups.commands.push(c.name.as_str()),
                ComponentKind::Instruction => groups.instructions.push(c.name.as_str()),
                ComponentKind::Hook => groups.hooks.push(c.name.as_str()),
            }
        }
        groups.serialize(serializer)
    }
}

/// インストール済みプラグインの一覧を取得
///
/// キャッシュディレクトリをスキャンし、有効なプラグインの一覧を返す。
pub fn list_installed_plugins(cache: &dyn PackageCacheAccess) -> Result<Vec<PluginSummary>> {
    // デプロイ済みプラグイン集合を事前取得（パフォーマンス改善）
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let deployed = list_all_placed(&project_root);

    let plugins = list_installed(cache)?
        .into_iter()
        .map(|pkg| {
            let name = pkg.manifest().name.clone();
            let plugin = Plugin::new(pkg.manifest().clone(), pkg.path().to_path_buf());
            let marketplace_str = pkg.marketplace().unwrap_or("github");
            let ops_key = pkg.cache_key().unwrap_or(&name);
            let enabled = meta::is_enabled(pkg.path(), marketplace_str, ops_key, &deployed);

            PluginSummary {
                plugin,
                install_id: pkg.cache_key().map(str::to_string),
                marketplace: pkg.marketplace().map(str::to_string),
                enabled,
            }
        })
        .collect();

    Ok(plugins)
}

/// 全ターゲットから配置済みプラグインを収集
///
/// 全ターゲット・全コンポーネント種別のデプロイ済みコンポーネントを走査し、
/// プラグインの (marketplace, plugin_name) の集合を返す。
///
/// `plugin_info.rs` からも使用されるため `pub(crate)` で公開。
pub(crate) fn list_all_placed(project_root: &Path) -> HashSet<(String, String)> {
    let targets = all_targets();
    let mut all_items = Vec::new();

    for target in &targets {
        for kind in ComponentKind::all() {
            if !target.supports(*kind) {
                continue;
            }
            // エラー時は黙殺（保守的に deployed とみなさない）
            if let Ok(placed) = target.list_placed(*kind, Scope::Project, project_root) {
                all_items.extend(placed);
            }
        }
    }

    list_placed_plugins(&all_items)
}

#[cfg(test)]
#[path = "plugin_catalog_test.rs"]
mod tests;
