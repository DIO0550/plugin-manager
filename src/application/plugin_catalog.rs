//! プラグインカタログ
//!
//! インストール済みプラグインの一覧取得ユースケースを提供する。

use crate::component::{ComponentKind, ComponentName, ComponentTypeCount, Scope};
use crate::error::Result;
use crate::plugin::{has_manifest, meta, PluginCache, PluginManifest};
use crate::scan::{list_placed_plugins, scan_components, ComponentScan};
use crate::target::all_targets;
use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// プラグイン情報のサマリ（DTO）
#[derive(Debug, Clone, Serialize)]
pub struct PluginSummary {
    /// プラグイン名
    pub name: String,
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
pub fn list_installed_plugins() -> Result<Vec<PluginSummary>> {
    let cache = PluginCache::new()?;
    let plugin_list = cache.list()?;

    // デプロイ済みプラグイン集合を事前取得（パフォーマンス改善）
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let deployed = list_all_placed(&project_root);

    let mut plugins = Vec::new();

    for (marketplace, name) in plugin_list {
        // 隠しディレクトリやメタデータディレクトリは除外
        if name.starts_with('.') {
            continue;
        }

        let plugin_path = cache.plugin_path(marketplace.as_deref(), &name);

        // plugin.json が存在するもののみをプラグインとして扱う
        if !has_manifest(&plugin_path) {
            continue;
        }

        let manifest = match cache.load_manifest(marketplace.as_deref(), &name) {
            Ok(m) => m,
            Err(_) => continue,
        };

        plugins.push(build_summary(
            name,
            marketplace,
            &plugin_path,
            &manifest,
            &deployed,
        ));
    }

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

/// PluginSummary を構築
fn build_summary(
    name: String,
    marketplace: Option<String>,
    plugin_path: &Path,
    manifest: &PluginManifest,
    deployed: &HashSet<(String, String)>,
) -> PluginSummary {
    // 統一スキャンAPIを使用
    let scan: ComponentScan = scan_components(plugin_path, manifest);

    // デプロイ状態をチェック（meta::is_enabled に委譲）
    let marketplace_str = marketplace.as_deref().unwrap_or("github");
    let enabled = meta::is_enabled(plugin_path, marketplace_str, &name, deployed);

    PluginSummary {
        name,
        marketplace,
        version: manifest.version.clone(),
        skills: scan.skills,
        agents: scan.agents,
        commands: scan.commands,
        instructions: scan.instructions,
        hooks: scan.hooks,
        enabled,
    }
}

#[cfg(test)]
#[path = "plugin_catalog_test.rs"]
mod tests;
