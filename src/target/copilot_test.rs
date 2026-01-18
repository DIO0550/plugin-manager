use super::*;

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
    assert!(!target.supports(ComponentKind::Hook));
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
        scope: PlacementScope(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    assert!(target.placement_location(&ctx).is_none());
}

#[test]
fn test_copilot_placement_location_skill_project() {
    let target = CopilotTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Skill, "my-skill"),
        origin: &origin,
        scope: PlacementScope(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_dir());
    assert_eq!(
        location.as_path(),
        Path::new("/project/.github/skills/official/my-plugin/my-skill")
    );
}

#[test]
fn test_copilot_placement_location_agent() {
    let target = CopilotTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    // Personal scope
    let ctx_personal = PlacementContext {
        component: ComponentRef::new(ComponentKind::Agent, "test"),
        origin: &origin,
        scope: PlacementScope(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    let location_personal = target.placement_location(&ctx_personal).unwrap();
    assert!(location_personal.is_file());
    assert!(location_personal
        .as_path()
        .to_string_lossy()
        .contains(".copilot/agents/official/my-plugin/test.agent.md"));

    // Project scope
    let ctx_project = PlacementContext {
        component: ComponentRef::new(ComponentKind::Agent, "test"),
        origin: &origin,
        scope: PlacementScope(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location_project = target.placement_location(&ctx_project).unwrap();
    assert!(location_project.is_file());
    assert_eq!(
        location_project.as_path(),
        Path::new("/project/.github/agents/official/my-plugin/test.agent.md")
    );
}

#[test]
fn test_copilot_placement_location_command() {
    let target = CopilotTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    // Personal scope for commands is not supported
    let ctx_personal = PlacementContext {
        component: ComponentRef::new(ComponentKind::Command, "my-command"),
        origin: &origin,
        scope: PlacementScope(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    assert!(target.placement_location(&ctx_personal).is_none());

    // Project scope
    let ctx_project = PlacementContext {
        component: ComponentRef::new(ComponentKind::Command, "my-command"),
        origin: &origin,
        scope: PlacementScope(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx_project).unwrap();
    assert!(location.is_file());
    assert_eq!(
        location.as_path(),
        Path::new("/project/.github/prompts/official/my-plugin/my-command.prompt.md")
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
        scope: PlacementScope(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_file());
    assert_eq!(
        location.as_path(),
        Path::new("/project/.github/copilot-instructions.md")
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
        scope: PlacementScope(Scope::Project),
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
        scope: PlacementScope(Scope::Project),
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
