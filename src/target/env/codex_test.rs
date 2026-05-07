use super::*;
use crate::component::{ComponentRef, PlacementScope, ProjectContext};
use crate::target::paths::home_dir;
use crate::target::PluginOrigin;

#[test]
fn test_codex_target_name() {
    let target = CodexTarget::new();
    assert_eq!(target.name(), "codex");
    assert_eq!(target.display_name(), "OpenAI Codex");
}

#[test]
fn test_codex_supported_components() {
    let target = CodexTarget::new();
    assert!(target.supports(ComponentKind::Skill));
    assert!(target.supports(ComponentKind::Agent));
    assert!(target.supports(ComponentKind::Instruction));
    assert!(target.supports(ComponentKind::Hook));
    assert!(!target.supports(ComponentKind::Command));
}

#[test]
fn test_codex_placement_location_skill_with_hierarchy() {
    // インストール経路では `Component.name` が `flatten_name(plugin, original)
    // = "{plugin}_{original}"` に平坦化されるため、テストもその形を使う。
    let target = CodexTarget::new();
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
        Path::new("/project/.codex/skills/my-plugin_my-skill")
    );
}

#[test]
fn test_codex_placement_location_skill_github_direct() {
    // インストール経路では `Component.name` が `flatten_name(plugin, original)
    // = "{plugin}_{original}"` に平坦化されるため、origin の種別 (github) に
    // 関わらずテストもその形を使う。
    let target = CodexTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_github("owner", "repo");

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
        Path::new("/project/.codex/skills/my-plugin_my-skill")
    );
}

#[test]
fn test_codex_placement_location_agent() {
    // インストール経路では `Component.name` が `flatten_name(plugin, original)
    // = "{plugin}_{original}"` に平坦化されるため、テストもその形を使う。
    let target = CodexTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Agent, "my-plugin_my-agent"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_file());
    assert_eq!(
        location.as_path(),
        Path::new("/project/.codex/agents/my-plugin_my-agent.agent.md")
    );
}

#[test]
fn test_codex_placement_location_instruction() {
    let target = CodexTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    // Project scope
    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Instruction, "test"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_file());
    assert_eq!(location.as_path(), Path::new("/project/AGENTS.md"));
}

#[test]
fn test_codex_placement_location_skill_with_prefixed_name() {
    let target = CodexTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Skill, "myplugin_foo"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();
    assert_eq!(
        location.as_path(),
        Path::new("/project/.codex/skills/myplugin_foo")
    );
}

#[test]
fn test_codex_placement_location_agent_with_prefixed_name() {
    let target = CodexTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Agent, "myplugin_foo"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();
    assert_eq!(
        location.as_path(),
        Path::new("/project/.codex/agents/myplugin_foo.agent.md")
    );
}

#[test]
fn test_codex_command_not_supported() {
    let target = CodexTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("test", "test");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Command, "test"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    assert!(target.placement_location(&ctx).is_none());
}

#[test]
fn test_codex_placement_location_hook_project_scope() {
    let target = CodexTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("test", "test");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Hook, "test_pre-tool-use"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_file());
    assert_eq!(location.as_path(), Path::new("/project/.codex/hooks.json"));
}

#[test]
fn test_codex_placement_location_hook_personal_scope() {
    let target = CodexTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("test", "test");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Hook, "test_pre-tool-use"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();

    assert!(location.is_file());
    assert_eq!(location.as_path(), home_dir().join(".codex/hooks.json"));
}

#[test]
fn hook_overwrite_error_returns_none_when_target_does_not_exist() {
    let temp = tempfile::TempDir::new().unwrap();
    let target_path = temp.path().join("hooks.json"); // 存在しない
    let plugin_root = temp.path();

    assert!(CodexTarget::hook_overwrite_error(&target_path, plugin_root).is_none());
}

#[test]
fn hook_overwrite_error_returns_error_when_target_exists_and_not_managed() {
    let temp = tempfile::TempDir::new().unwrap();
    let target_path = temp.path().join("hooks.json");
    std::fs::write(&target_path, "{}").unwrap();

    let plugin_root_dir = tempfile::TempDir::new().unwrap();
    // .plm-meta.json は無いので、過去 install 履歴なし → error
    let result = CodexTarget::hook_overwrite_error(&target_path, plugin_root_dir.path());
    assert!(result.is_some());
    let msg = result.unwrap();
    assert!(msg.contains("already exists"));
    assert!(msg.contains("not managed by this plugin"));
}

#[test]
fn hook_overwrite_error_returns_none_when_plugin_already_owns_codex() {
    let temp = tempfile::TempDir::new().unwrap();
    let target_path = temp.path().join("hooks.json");
    std::fs::write(&target_path, "{}").unwrap();

    let plugin_root_dir = tempfile::TempDir::new().unwrap();
    let mut meta = crate::plugin::meta::PluginMeta::default();
    meta.set_status("codex", "enabled");
    crate::plugin::meta::write_meta(plugin_root_dir.path(), &meta).unwrap();

    // 同プラグインの再 install は許可される
    assert!(
        CodexTarget::hook_overwrite_error(&target_path, plugin_root_dir.path()).is_none(),
        "re-install of the same plugin must be allowed when codex was previously enabled"
    );
}
