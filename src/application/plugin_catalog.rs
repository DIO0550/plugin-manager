//! プラグインカタログ
//!
//! インストール済みプラグインの一覧取得ユースケースを提供する。

use crate::component::{ComponentKind, ComponentName, ComponentTypeCount, Scope};
use crate::error::Result;
use crate::plugin::{meta, MarketplacePackage, PluginCacheAccess};
use crate::scan::{list_placed_plugins, scan_components, ComponentScan};
use crate::target::all_targets;
use serde::Serialize;
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
pub(crate) fn list_installed(cache: &dyn PluginCacheAccess) -> Result<Vec<MarketplacePackage>> {
    let packages = cache
        .list()?
        .into_iter()
        .map(PluginCacheKey::from)
        .filter(|key| !key.name.starts_with('.'))
        .filter_map(|key| {
            cache
                .load_package(key.marketplace.as_deref(), &key.name)
                .ok()
                .map(MarketplacePackage::from)
        })
        .collect();
    Ok(packages)
}

/// プラグイン情報のサマリ（DTO）
#[derive(Debug, Clone, Serialize)]
pub struct PluginSummary {
    /// プラグイン名（表示用）
    pub name: String,
    /// キャッシュディレクトリ名（ファイル操作用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_key: Option<String>,
    /// マーケットプレイス名（マーケットプレイス経由の場合）
    pub marketplace: Option<String>,
    /// バージョン
    pub version: String,
    /// スキル名一覧
    pub skills: Vec<String>,
    /// エージェント名一覧
    pub agents: Vec<String>,
    /// コマンド名一覧
    pub commands: Vec<String>,
    /// インストラクション名一覧
    pub instructions: Vec<String>,
    /// フック名一覧
    pub hooks: Vec<String>,
    /// 有効状態（デプロイ先に配置されているか）
    pub enabled: bool,
}

impl PluginSummary {
    /// キャッシュディレクトリ名を返す（`cache_key` が `None` の場合は `name` にフォールバック）
    pub fn cache_key(&self) -> &str {
        self.cache_key.as_deref().unwrap_or(&self.name)
    }

    /// コンポーネントの総数を取得
    pub fn component_count(&self) -> usize {
        self.skills.len()
            + self.agents.len()
            + self.commands.len()
            + self.instructions.len()
            + self.hooks.len()
    }

    /// コンポーネントが存在するか
    pub fn has_components(&self) -> bool {
        self.component_count() > 0
    }

    /// コンポーネント種別ごとの件数を取得（空でないもののみ）
    pub fn component_type_counts(&self) -> Vec<ComponentTypeCount> {
        let mut counts = Vec::new();

        if !self.skills.is_empty() {
            counts.push(ComponentTypeCount {
                kind: ComponentKind::Skill,
                count: self.skills.len(),
            });
        }
        if !self.agents.is_empty() {
            counts.push(ComponentTypeCount {
                kind: ComponentKind::Agent,
                count: self.agents.len(),
            });
        }
        if !self.commands.is_empty() {
            counts.push(ComponentTypeCount {
                kind: ComponentKind::Command,
                count: self.commands.len(),
            });
        }
        if !self.instructions.is_empty() {
            counts.push(ComponentTypeCount {
                kind: ComponentKind::Instruction,
                count: self.instructions.len(),
            });
        }
        if !self.hooks.is_empty() {
            counts.push(ComponentTypeCount {
                kind: ComponentKind::Hook,
                count: self.hooks.len(),
            });
        }

        counts
    }

    /// 特定種別のコンポーネント名一覧を取得
    pub fn component_names(&self, kind: ComponentKind) -> Vec<ComponentName> {
        let names = match kind {
            ComponentKind::Skill => &self.skills,
            ComponentKind::Agent => &self.agents,
            ComponentKind::Command => &self.commands,
            ComponentKind::Instruction => &self.instructions,
            ComponentKind::Hook => &self.hooks,
        };

        names
            .iter()
            .map(|n| ComponentName { name: n.clone() })
            .collect()
    }
}

/// インストール済みプラグインの一覧を取得
///
/// キャッシュディレクトリをスキャンし、有効なプラグインの一覧を返す。
pub fn list_installed_plugins(cache: &dyn PluginCacheAccess) -> Result<Vec<PluginSummary>> {
    // デプロイ済みプラグイン集合を事前取得（パフォーマンス改善）
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let deployed = list_all_placed(&project_root);

    let plugins = list_installed(cache)?
        .into_iter()
        .map(|pkg| {
            let name = pkg.manifest().name.clone();
            let scan: ComponentScan = scan_components(pkg.path(), pkg.manifest());
            let marketplace_str = pkg.marketplace().unwrap_or("github");
            let ops_key = pkg.cache_key().unwrap_or(&name);
            let enabled = meta::is_enabled(pkg.path(), marketplace_str, ops_key, &deployed);

            PluginSummary {
                name,
                cache_key: pkg.cache_key().map(str::to_string),
                marketplace: pkg.marketplace().map(str::to_string),
                version: pkg.manifest().version.clone(),
                skills: scan.skills,
                agents: scan.agents,
                commands: scan.commands,
                instructions: scan.instructions,
                hooks: scan.hooks,
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
