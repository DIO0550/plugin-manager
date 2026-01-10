//! プラグインカタログ
//!
//! インストール済みプラグインの一覧取得ユースケースを提供する。

use crate::component::{ComponentKind, ComponentName, ComponentTypeCount, Scope};
use crate::error::Result;
use crate::plugin::{has_manifest, PluginCache, PluginManifest};
use crate::scan::{scan_components, ComponentScan};
use crate::target::{all_targets, PluginOrigin};
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
    let deployed = collect_deployed_plugins(&project_root);

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

        plugins.push(build_summary(name, marketplace, &plugin_path, &manifest, &deployed));
    }

    Ok(plugins)
}

/// デプロイ済みプラグインの集合を取得
///
/// 全ターゲット・全コンポーネント種別のデプロイ済みコンポーネントを走査し、
/// プラグインの (marketplace, plugin_name) の集合を返す。
fn collect_deployed_plugins(project_root: &Path) -> HashSet<(String, String)> {
    let mut deployed = HashSet::new();
    let targets = all_targets();

    for target in &targets {
        for kind in ComponentKind::all() {
            if !target.supports(*kind) {
                continue;
            }
            // エラー時は黙殺（保守的に deployed とみなさない）
            if let Ok(placed) = target.list_placed(*kind, Scope::Project, project_root) {
                for item in placed {
                    if let Some((mp, plugin)) = parse_placed_item(&item) {
                        deployed.insert((mp, plugin));
                    }
                }
            }
        }
    }
    deployed
}

/// "marketplace/plugin/component" 形式をパース
///
/// 戻り値: Some((marketplace, plugin_name)) または None
fn parse_placed_item(item: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = item.split('/').collect();
    if parts.len() >= 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
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

    // デプロイ状態をチェック
    let origin = PluginOrigin::from_cached_plugin(marketplace.as_deref(), &name);
    let enabled = deployed.contains(&(origin.marketplace, origin.plugin));

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
mod tests {
    use super::*;

    fn create_empty_summary() -> PluginSummary {
        PluginSummary {
            name: "test-plugin".to_string(),
            marketplace: None,
            version: "1.0.0".to_string(),
            skills: vec![],
            agents: vec![],
            commands: vec![],
            instructions: vec![],
            hooks: vec![],
            enabled: true,
        }
    }

    fn create_full_summary() -> PluginSummary {
        PluginSummary {
            name: "full-plugin".to_string(),
            marketplace: Some("awesome-marketplace".to_string()),
            version: "2.0.0".to_string(),
            skills: vec!["skill1".to_string(), "skill2".to_string()],
            agents: vec!["agent1".to_string()],
            commands: vec!["cmd1".to_string(), "cmd2".to_string(), "cmd3".to_string()],
            instructions: vec!["inst1".to_string()],
            hooks: vec!["hook1".to_string(), "hook2".to_string()],
            enabled: true,
        }
    }

    // ========================================
    // component_count tests
    // ========================================

    #[test]
    fn test_component_count_empty() {
        let summary = create_empty_summary();
        assert_eq!(summary.component_count(), 0);
    }

    #[test]
    fn test_component_count_full() {
        let summary = create_full_summary();
        // 2 skills + 1 agent + 3 commands + 1 instruction + 2 hooks = 9
        assert_eq!(summary.component_count(), 9);
    }

    #[test]
    fn test_component_count_partial() {
        let summary = PluginSummary {
            name: "partial".to_string(),
            marketplace: None,
            version: "1.0.0".to_string(),
            skills: vec!["s1".to_string()],
            agents: vec![],
            commands: vec!["c1".to_string(), "c2".to_string()],
            instructions: vec![],
            hooks: vec![],
            enabled: true,
        };
        assert_eq!(summary.component_count(), 3);
    }

    // ========================================
    // has_components tests
    // ========================================

    #[test]
    fn test_has_components_empty() {
        let summary = create_empty_summary();
        assert!(!summary.has_components());
    }

    #[test]
    fn test_has_components_with_skills() {
        let mut summary = create_empty_summary();
        summary.skills = vec!["skill".to_string()];
        assert!(summary.has_components());
    }

    #[test]
    fn test_has_components_with_hooks_only() {
        let mut summary = create_empty_summary();
        summary.hooks = vec!["hook".to_string()];
        assert!(summary.has_components());
    }

    // ========================================
    // component_type_counts tests
    // ========================================

    #[test]
    fn test_component_type_counts_empty() {
        let summary = create_empty_summary();
        let counts = summary.component_type_counts();
        assert!(counts.is_empty());
    }

    #[test]
    fn test_component_type_counts_full() {
        let summary = create_full_summary();
        let counts = summary.component_type_counts();

        assert_eq!(counts.len(), 5);

        // Check each type
        let skill_count = counts.iter().find(|c| c.kind == ComponentKind::Skill).unwrap();
        assert_eq!(skill_count.count, 2);

        let agent_count = counts.iter().find(|c| c.kind == ComponentKind::Agent).unwrap();
        assert_eq!(agent_count.count, 1);

        let cmd_count = counts.iter().find(|c| c.kind == ComponentKind::Command).unwrap();
        assert_eq!(cmd_count.count, 3);

        let inst_count = counts.iter().find(|c| c.kind == ComponentKind::Instruction).unwrap();
        assert_eq!(inst_count.count, 1);

        let hook_count = counts.iter().find(|c| c.kind == ComponentKind::Hook).unwrap();
        assert_eq!(hook_count.count, 2);
    }

    #[test]
    fn test_component_type_counts_partial() {
        let summary = PluginSummary {
            name: "partial".to_string(),
            marketplace: None,
            version: "1.0.0".to_string(),
            skills: vec![],
            agents: vec!["a1".to_string(), "a2".to_string()],
            commands: vec![],
            instructions: vec![],
            hooks: vec!["h1".to_string()],
            enabled: true,
        };

        let counts = summary.component_type_counts();

        assert_eq!(counts.len(), 2);
        assert!(counts.iter().any(|c| c.kind == ComponentKind::Agent && c.count == 2));
        assert!(counts.iter().any(|c| c.kind == ComponentKind::Hook && c.count == 1));
    }

    #[test]
    fn test_component_type_counts_order() {
        let summary = create_full_summary();
        let counts = summary.component_type_counts();

        // Should be in order: Skill, Agent, Command, Instruction, Hook
        assert_eq!(counts[0].kind, ComponentKind::Skill);
        assert_eq!(counts[1].kind, ComponentKind::Agent);
        assert_eq!(counts[2].kind, ComponentKind::Command);
        assert_eq!(counts[3].kind, ComponentKind::Instruction);
        assert_eq!(counts[4].kind, ComponentKind::Hook);
    }

    // ========================================
    // component_names tests
    // ========================================

    #[test]
    fn test_component_names_skills() {
        let summary = create_full_summary();
        let names = summary.component_names(ComponentKind::Skill);

        assert_eq!(names.len(), 2);
        assert_eq!(names[0].name, "skill1");
        assert_eq!(names[1].name, "skill2");
    }

    #[test]
    fn test_component_names_agents() {
        let summary = create_full_summary();
        let names = summary.component_names(ComponentKind::Agent);

        assert_eq!(names.len(), 1);
        assert_eq!(names[0].name, "agent1");
    }

    #[test]
    fn test_component_names_commands() {
        let summary = create_full_summary();
        let names = summary.component_names(ComponentKind::Command);

        assert_eq!(names.len(), 3);
    }

    #[test]
    fn test_component_names_instructions() {
        let summary = create_full_summary();
        let names = summary.component_names(ComponentKind::Instruction);

        assert_eq!(names.len(), 1);
        assert_eq!(names[0].name, "inst1");
    }

    #[test]
    fn test_component_names_hooks() {
        let summary = create_full_summary();
        let names = summary.component_names(ComponentKind::Hook);

        assert_eq!(names.len(), 2);
    }

    #[test]
    fn test_component_names_empty() {
        let summary = create_empty_summary();

        for kind in ComponentKind::all() {
            let names = summary.component_names(*kind);
            assert!(names.is_empty(), "{:?} should be empty", kind);
        }
    }
}
