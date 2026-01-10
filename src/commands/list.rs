//! plm list コマンド
//!
//! インストール済みプラグインの一覧を表示する。

use crate::application::{list_installed_plugins, PluginSummary};
use crate::component::ComponentKind;
use crate::target::TargetKind;
use clap::Parser;
use comfy_table::{presets::UTF8_FULL, Table};

#[derive(Debug, Parser)]
pub struct Args {
    /// Filter by component type
    #[arg(long = "type", value_enum)]
    pub component_type: Option<ComponentKind>,

    /// Filter by target environment (currently filters by enabled status)
    #[arg(long, value_enum)]
    pub target: Option<TargetKind>,

    /// Output in JSON format
    #[arg(long, conflicts_with = "simple")]
    pub json: bool,

    /// Output only plugin names
    #[arg(long, conflicts_with = "json")]
    pub simple: bool,
}

pub async fn run(args: Args) -> Result<(), String> {
    // 1. プラグイン一覧を取得
    let mut plugins = list_installed_plugins().map_err(|e| e.to_string())?;

    let total_count = plugins.len();

    // 2. ソート（name昇順）
    plugins.sort_by(|a, b| a.name.cmp(&b.name));

    // 3. フィルタリング
    let filtered = filter_plugins(plugins, &args);

    // 4. 出力
    if args.json {
        print_json(&filtered)?;
    } else if args.simple {
        print_simple(&filtered, total_count);
    } else {
        print_table(&filtered, total_count);
    }

    Ok(())
}

fn filter_plugins(plugins: Vec<PluginSummary>, args: &Args) -> Vec<PluginSummary> {
    plugins
        .into_iter()
        .filter(|p| filter_by_type(p, args.component_type.as_ref()))
        .filter(|p| filter_by_target(p, args.target.as_ref()))
        .collect()
}

fn filter_by_type(plugin: &PluginSummary, component_type: Option<&ComponentKind>) -> bool {
    match component_type {
        None => true,
        Some(ComponentKind::Skill) => !plugin.skills.is_empty(),
        Some(ComponentKind::Agent) => !plugin.agents.is_empty(),
        Some(ComponentKind::Command) => !plugin.commands.is_empty(),
        Some(ComponentKind::Instruction) => !plugin.instructions.is_empty(),
        Some(ComponentKind::Hook) => !plugin.hooks.is_empty(),
    }
}

fn filter_by_target(plugin: &PluginSummary, target: Option<&TargetKind>) -> bool {
    // Phase 1: シンプルにenabled状態でフィルタ
    // ターゲット指定時は、そのターゲットで有効なプラグインのみ表示
    // 現状の PluginSummary にはターゲット別のデプロイ情報がないため、
    // enabled = true のプラグインを「ターゲットにデプロイ済み」とみなす
    match target {
        None => true,
        Some(_) => plugin.enabled,
    }
}

fn print_table(plugins: &[PluginSummary], total_count: usize) {
    if plugins.is_empty() {
        if total_count == 0 {
            println!("No plugins installed");
        } else {
            println!("No plugins matched");
        }
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        "Name",
        "Version",
        "Components",
        "Status",
        "Marketplace",
    ]);

    for plugin in plugins {
        let status = if plugin.enabled {
            "enabled"
        } else {
            "disabled"
        };
        let marketplace = plugin.marketplace.as_deref().unwrap_or("-");
        let components = format_components(plugin);

        table.add_row(vec![
            plugin.name.as_str(),
            plugin.version.as_str(),
            components.as_str(),
            status,
            marketplace,
        ]);
    }

    println!("{table}");
}

fn print_json(plugins: &[PluginSummary]) -> Result<(), String> {
    // 空の場合も [] を出力
    serde_json::to_string_pretty(plugins)
        .map(|json| println!("{json}"))
        .map_err(|e| format!("Failed to serialize plugins: {}", e))
}

fn print_simple(plugins: &[PluginSummary], total_count: usize) {
    if plugins.is_empty() {
        if total_count == 0 {
            println!("No plugins installed");
        } else {
            println!("No plugins matched");
        }
        return;
    }
    for plugin in plugins {
        println!("{}", plugin.name);
    }
}

fn format_components(plugin: &PluginSummary) -> String {
    let counts = plugin.component_type_counts();
    if counts.is_empty() {
        return "-".to_string();
    }
    // component_type_counts() は固定順序: Skill → Agent → Command → Instruction → Hook
    counts
        .iter()
        .map(|c| format!("{} {}", c.count, c.kind.plural()))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_empty_plugin(name: &str) -> PluginSummary {
        PluginSummary {
            name: name.to_string(),
            marketplace: None,
            version: "1.0.0".to_string(),
            skills: vec![],
            agents: vec![],
            commands: vec![],
            instructions: vec![],
            hooks: vec![],
            enabled: false,
        }
    }

    fn create_plugin_with_skills(name: &str, skill_count: usize, enabled: bool) -> PluginSummary {
        PluginSummary {
            name: name.to_string(),
            marketplace: Some("github".to_string()),
            version: "1.0.0".to_string(),
            skills: (0..skill_count).map(|i| format!("skill{}", i)).collect(),
            agents: vec![],
            commands: vec![],
            instructions: vec![],
            hooks: vec![],
            enabled,
        }
    }

    fn create_full_plugin(name: &str, enabled: bool) -> PluginSummary {
        PluginSummary {
            name: name.to_string(),
            marketplace: Some("github".to_string()),
            version: "2.0.0".to_string(),
            skills: vec!["skill1".to_string(), "skill2".to_string()],
            agents: vec!["agent1".to_string()],
            commands: vec!["cmd1".to_string()],
            instructions: vec!["inst1".to_string()],
            hooks: vec!["hook1".to_string()],
            enabled,
        }
    }

    // ========================================
    // filter_by_type tests
    // ========================================

    #[test]
    fn test_filter_by_type_none_passes_all() {
        let empty = create_empty_plugin("empty");
        let with_skills = create_plugin_with_skills("with-skills", 2, true);

        assert!(filter_by_type(&empty, None));
        assert!(filter_by_type(&with_skills, None));
    }

    #[test]
    fn test_filter_by_type_skill_matches() {
        let plugin = create_plugin_with_skills("test", 2, true);
        assert!(filter_by_type(&plugin, Some(&ComponentKind::Skill)));
    }

    #[test]
    fn test_filter_by_type_skill_no_match() {
        let plugin = create_empty_plugin("test");
        assert!(!filter_by_type(&plugin, Some(&ComponentKind::Skill)));
    }

    #[test]
    fn test_filter_by_type_agent_matches() {
        let plugin = create_full_plugin("test", true);
        assert!(filter_by_type(&plugin, Some(&ComponentKind::Agent)));
    }

    #[test]
    fn test_filter_by_type_command_matches() {
        let plugin = create_full_plugin("test", true);
        assert!(filter_by_type(&plugin, Some(&ComponentKind::Command)));
    }

    #[test]
    fn test_filter_by_type_instruction_matches() {
        let plugin = create_full_plugin("test", true);
        assert!(filter_by_type(&plugin, Some(&ComponentKind::Instruction)));
    }

    #[test]
    fn test_filter_by_type_hook_matches() {
        let plugin = create_full_plugin("test", true);
        assert!(filter_by_type(&plugin, Some(&ComponentKind::Hook)));
    }

    #[test]
    fn test_filter_by_type_hook_no_match() {
        let plugin = create_plugin_with_skills("test", 1, true);
        assert!(!filter_by_type(&plugin, Some(&ComponentKind::Hook)));
    }

    // ========================================
    // filter_by_target tests
    // ========================================

    #[test]
    fn test_filter_by_target_none_passes_all() {
        let enabled = create_plugin_with_skills("enabled", 1, true);
        let disabled = create_plugin_with_skills("disabled", 1, false);

        assert!(filter_by_target(&enabled, None));
        assert!(filter_by_target(&disabled, None));
    }

    #[test]
    fn test_filter_by_target_enabled_only() {
        let enabled = create_plugin_with_skills("enabled", 1, true);
        let disabled = create_plugin_with_skills("disabled", 1, false);

        assert!(filter_by_target(&enabled, Some(&TargetKind::Codex)));
        assert!(!filter_by_target(&disabled, Some(&TargetKind::Codex)));

        assert!(filter_by_target(&enabled, Some(&TargetKind::Copilot)));
        assert!(!filter_by_target(&disabled, Some(&TargetKind::Copilot)));
    }

    // ========================================
    // format_components tests
    // ========================================

    #[test]
    fn test_format_components_empty() {
        let plugin = create_empty_plugin("empty");
        assert_eq!(format_components(&plugin), "-");
    }

    #[test]
    fn test_format_components_single() {
        let plugin = create_plugin_with_skills("test", 2, true);
        assert_eq!(format_components(&plugin), "2 skills");
    }

    #[test]
    fn test_format_components_multiple() {
        let plugin = create_full_plugin("test", true);
        // 固定順序: Skill → Agent → Command → Instruction → Hook
        assert_eq!(
            format_components(&plugin),
            "2 skills, 1 agents, 1 commands, 1 instructions, 1 hooks"
        );
    }

    // ========================================
    // filter_plugins tests
    // ========================================

    #[test]
    fn test_filter_plugins_combined() {
        let plugins = vec![
            create_plugin_with_skills("enabled-with-skills", 2, true),
            create_plugin_with_skills("disabled-with-skills", 1, false),
            create_empty_plugin("enabled-empty"),
            create_full_plugin("enabled-full", true),
        ];

        // --type skill --target codex
        let args = Args {
            component_type: Some(ComponentKind::Skill),
            target: Some(TargetKind::Codex),
            json: false,
            simple: false,
        };

        let filtered = filter_plugins(plugins, &args);

        // enabled かつ skills を持つもののみ
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].name, "enabled-with-skills");
        assert_eq!(filtered[1].name, "enabled-full");
    }
}
