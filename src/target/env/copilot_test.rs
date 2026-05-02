use super::*;
use crate::component::{ComponentRef, PlacementScope, ProjectContext};
use crate::target::PluginOrigin;

#[test]
fn test_copilot_target_name() {
    let target = CopilotTarget::new();
    assert_eq!(target.name(), "copilot");
    assert_eq!(target.display_name(), "GitHub Copilot");
}

#[test]
fn test_copilot_supported_components() {
    let target = CopilotTarget::new();
    assert!(target.supports(ComponentKind::Skill));
    assert!(target.supports(ComponentKind::Agent));
    assert!(target.supports(ComponentKind::Command));
    assert!(target.supports(ComponentKind::Instruction));
    assert!(target.supports(ComponentKind::Hook));
}

#[test]
fn test_copilot_skill_personal_not_supported() {
    let target = CopilotTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    // Personal scope for skills is not supported
    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Skill, "my-skill"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    assert!(target.placement_location(&ctx).is_none());
}

#[test]
fn test_copilot_placement_location_skill_project() {
    // インストール経路では `Component.name` が `flatten_name(plugin, original)
    // = "{plugin}_{original}"` に平坦化されるため、テストもその形を使う。
    let target = CopilotTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Skill, "my-plugin_my-skill"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_dir());
    assert_eq!(
        location.as_path(),
        Path::new("/project/.github/skills/my-plugin_my-skill")
    );
}

#[test]
fn test_copilot_placement_location_agent() {
    // インストール経路では Agent も `flatten_name(plugin, original)
    // = "{plugin}_{original}"` に平坦化されるため、テストもその形を使う。
    let target = CopilotTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    // Personal scope
    let ctx_personal = PlacementContext {
        component: ComponentRef::new(ComponentKind::Agent, "my-plugin_test"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    let location_personal = target.placement_location(&ctx_personal).unwrap();
    assert!(location_personal.is_file());
    assert!(location_personal.as_path().ends_with(
        Path::new(".copilot")
            .join("agents")
            .join("my-plugin_test.agent.md")
    ));

    // Project scope
    let ctx_project = PlacementContext {
        component: ComponentRef::new(ComponentKind::Agent, "my-plugin_test"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location_project = target.placement_location(&ctx_project).unwrap();
    assert!(location_project.is_file());
    assert_eq!(
        location_project.as_path(),
        Path::new("/project/.github/agents/my-plugin_test.agent.md")
    );
}

#[test]
fn test_copilot_placement_location_command() {
    // インストール経路では Command も `flatten_name(plugin, original)
    // = "{plugin}_{original}"` に平坦化されるため、テストもその形を使う。
    let target = CopilotTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    // Personal scope for commands is not supported
    let ctx_personal = PlacementContext {
        component: ComponentRef::new(ComponentKind::Command, "my-plugin_my-command"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    assert!(target.placement_location(&ctx_personal).is_none());

    // Project scope
    let ctx_project = PlacementContext {
        component: ComponentRef::new(ComponentKind::Command, "my-plugin_my-command"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx_project).unwrap();
    assert!(location.is_file());
    assert_eq!(
        location.as_path(),
        Path::new("/project/.github/prompts/my-plugin_my-command.prompt.md")
    );
}

#[test]
fn test_copilot_placement_location_instruction() {
    let target = CopilotTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Instruction, "test"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_file());
    assert_eq!(
        location.as_path(),
        Path::new("/project/.github/copilot-instructions.md")
    );
}

#[test]
fn test_copilot_placement_location_with_prefixed_name() {
    let target = CopilotTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let skill_ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Skill, "myplugin_foo"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    assert_eq!(
        target.placement_location(&skill_ctx).unwrap().as_path(),
        Path::new("/project/.github/skills/myplugin_foo")
    );

    let cmd_ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Command, "myplugin_foo"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    assert_eq!(
        target.placement_location(&cmd_ctx).unwrap().as_path(),
        Path::new("/project/.github/prompts/myplugin_foo.prompt.md")
    );
}

// =============================================================================
// Claude Code → Copilot 変換テスト
// =============================================================================

/// Claude Code形式のエージェント名（.md由来）がCopilot形式（.agent.md）で配置される
#[test]
fn test_copilot_agent_md_extension_transform() {
    let target = CopilotTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("claude-plugins", "example");

    // Claude Code プラグインのエージェント: agents/my-agent.md
    // list_agent_names() により名前は "my-agent" に正規化される
    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Agent, "my-agent"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };

    let location = target.placement_location(&ctx).unwrap();
    let path_str = location.as_path().to_string_lossy();

    // Copilot形式: .agent.md 拡張子で配置
    assert!(path_str.ends_with(".agent.md"), "Should end with .agent.md");
    assert!(
        path_str.contains("my-agent.agent.md"),
        "Should contain my-agent.agent.md"
    );
}

/// 単一ファイル指定のエッジケース: 名前に .agent が残る場合
/// 注意: この場合 my-agent.agent.agent.md になる（二重拡張）
/// これは既知の制限であり、ディレクトリ形式を推奨
#[test]
fn test_copilot_agent_single_file_edge_case() {
    let target = CopilotTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("test", "plugin");

    // 単一ファイル指定時: file_stem() により "my-agent.agent" になり得る
    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Agent, "my-agent.agent"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };

    let location = target.placement_location(&ctx).unwrap();
    let path_str = location.as_path().to_string_lossy();

    // 二重拡張子になる（既知の制限）
    assert!(
        path_str.ends_with(".agent.agent.md"),
        "Single file edge case: double extension"
    );
}

// =============================================================================
// Hook サポートテスト
// =============================================================================

#[test]
fn test_copilot_placement_location_hook_project() {
    // フラット化後は Hook も `flatten_name(plugin, original) = "{plugin}_{stem}"`
    // で配置されるため、テストもプリフィックス済みの名前を使う。
    let target = CopilotTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Hook, "my-plugin_pre-commit"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_file());
    assert_eq!(
        location.as_path(),
        Path::new("/project/.github/hooks/my-plugin_pre-commit.json")
    );
}

#[test]
fn test_copilot_placement_location_hook_personal() {
    // フラット化後は Hook も `flatten_name(plugin, original) = "{plugin}_{stem}"`
    // で配置されるため、テストもプリフィックス済みの名前を使う。
    let target = CopilotTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Hook, "my-plugin_pre-commit"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_file());
    assert!(location.as_path().ends_with(
        Path::new(".copilot")
            .join("hooks")
            .join("my-plugin_pre-commit.json")
    ));
}

#[test]
fn test_copilot_list_placed_hooks() {
    use std::fs;
    use tempfile::TempDir;

    let temp = TempDir::new().unwrap();
    let project_root = temp.path();

    // フラット 2 階層構造: .github/hooks/<flattened_name>.json
    let hooks_dir = project_root.join(".github").join("hooks");
    fs::create_dir_all(&hooks_dir).unwrap();
    fs::write(hooks_dir.join("my-plugin_pre-commit.json"), "{}").unwrap();

    let target = CopilotTarget::new();
    let result = target
        .list_placed(ComponentKind::Hook, Scope::Project, project_root)
        .unwrap();

    assert_eq!(result.len(), 1);
    // フラット化以降は `name` 単独 (= flattened_name) を識別子とする。
    assert_eq!(result[0], "my-plugin_pre-commit");
}

#[test]
fn test_copilot_filter_component_skill_requires_skill_md() {
    use std::fs;
    use tempfile::TempDir;

    let temp = TempDir::new().unwrap();
    let project_root = temp.path();

    // SKILL.md なしの Skill ディレクトリ -> Skill 扱いされない
    let skill_dir = project_root
        .join(".github")
        .join("skills")
        .join("plugin_no-skill-md");
    fs::create_dir_all(&skill_dir).unwrap();

    let target = CopilotTarget::new();
    let result = target
        .list_placed(ComponentKind::Skill, Scope::Project, project_root)
        .unwrap();
    assert!(result.is_empty(), "Skill without SKILL.md must be ignored");

    // SKILL.md ありの Skill ディレクトリ -> Skill として認識
    let valid_skill = project_root
        .join(".github")
        .join("skills")
        .join("plugin_my-skill");
    fs::create_dir_all(&valid_skill).unwrap();
    fs::write(valid_skill.join("SKILL.md"), "# Skill").unwrap();

    let result = target
        .list_placed(ComponentKind::Skill, Scope::Project, project_root)
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], "plugin_my-skill");
}
