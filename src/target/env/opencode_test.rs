//! OpenCodeTarget unit tests（Skills / Agents / Commands / Instructions）

use super::*;
use crate::component::{ComponentRef, PlacementScope, ProjectContext};
use crate::target::{CodexTarget, CursorTarget, PluginOrigin};
use std::ffi::OsStr;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use tempfile::TempDir;

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct EnvGuard {
    saved: Vec<(&'static str, Option<std::ffi::OsString>)>,
}

impl EnvGuard {
    fn clear(keys: &[&'static str]) -> Self {
        let saved = keys
            .iter()
            .map(|&key| {
                let prev = std::env::var_os(key);
                std::env::remove_var(key);
                (key, prev)
            })
            .collect();
        Self { saved }
    }

    fn set(&self, key: &'static str, value: impl AsRef<OsStr>) {
        std::env::set_var(key, value);
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, prev) in self.saved.drain(..) {
            match prev {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
    }
}

#[test]
fn test_opencode_name_and_kind() {
    let target = OpenCodeTarget::new();
    assert_eq!(target.name(), "opencode");
    assert_eq!(target.display_name(), "OpenCode");
    assert_eq!(target.kind(), TargetKind::OpenCode);
}

#[test]
fn test_opencode_supported_components() {
    let target = OpenCodeTarget::new();
    let supported = target.supported_components();
    assert_eq!(
        supported,
        &[
            ComponentKind::Skill,
            ComponentKind::Agent,
            ComponentKind::Command,
            ComponentKind::Instruction,
        ]
    );
    assert!(!target.supports(ComponentKind::Hook));
}

#[test]
fn test_opencode_supports_instruction() {
    let target = OpenCodeTarget::new();
    assert!(target.supports(ComponentKind::Instruction));
}

#[test]
fn test_opencode_supports_agent_and_command() {
    let target = OpenCodeTarget::new();
    assert!(target.supports(ComponentKind::Agent));
    assert!(target.supports(ComponentKind::Command));
}

#[test]
fn test_opencode_supports_scope_skill_both() {
    let target = OpenCodeTarget::new();
    assert!(target.supports_scope(ComponentKind::Skill, Scope::Personal));
    assert!(target.supports_scope(ComponentKind::Skill, Scope::Project));
}

#[test]
fn test_opencode_supports_scope_instruction_both() {
    let target = OpenCodeTarget::new();
    assert!(target.supports_scope(ComponentKind::Instruction, Scope::Personal));
    assert!(target.supports_scope(ComponentKind::Instruction, Scope::Project));
}

#[test]
fn test_opencode_supports_scope_agent_both() {
    let target = OpenCodeTarget::new();
    assert!(target.supports_scope(ComponentKind::Agent, Scope::Personal));
    assert!(target.supports_scope(ComponentKind::Agent, Scope::Project));
}

#[test]
fn test_opencode_supports_scope_command_both() {
    let target = OpenCodeTarget::new();
    assert!(target.supports_scope(ComponentKind::Command, Scope::Personal));
    assert!(target.supports_scope(ComponentKind::Command, Scope::Project));
}

#[test]
fn test_opencode_placement_skill_project_uses_original_name() {
    let target = OpenCodeTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::with_names(
            ComponentKind::Skill,
            "my-plugin_my-skill",
            "my-skill",
            "my-plugin",
        ),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();
    assert!(location.is_dir());
    assert_eq!(
        location.as_path(),
        Path::new("/project/.opencode/skills/my-skill")
    );
}

#[test]
fn test_opencode_placement_skill_personal_default_home() {
    let _lock = env_lock().lock().unwrap();
    let guard = EnvGuard::clear(&["XDG_CONFIG_HOME", "HOME"]);
    guard.set("HOME", "/home/u");

    let target = OpenCodeTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::with_names(
            ComponentKind::Skill,
            "my-plugin_my-skill",
            "my-skill",
            "my-plugin",
        ),
        origin: &origin,
        scope: PlacementScope::new(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();
    assert_eq!(
        location.as_path(),
        Path::new("/home/u/.config/opencode/skills/my-skill")
    );
}

#[test]
fn test_opencode_placement_skill_personal_respects_xdg_config_home() {
    let _lock = env_lock().lock().unwrap();
    let guard = EnvGuard::clear(&["XDG_CONFIG_HOME", "HOME"]);
    guard.set("HOME", "/home/u");
    guard.set("XDG_CONFIG_HOME", "/xdg/config");

    let target = OpenCodeTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::with_names(
            ComponentKind::Skill,
            "my-plugin_my-skill",
            "my-skill",
            "my-plugin",
        ),
        origin: &origin,
        scope: PlacementScope::new(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();
    assert_eq!(
        location.as_path(),
        Path::new("/xdg/config/opencode/skills/my-skill")
    );
}

#[test]
fn test_opencode_placement_skill_without_original_name_returns_none() {
    let target = OpenCodeTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Skill, "my-plugin_my-skill"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    assert!(target.placement_location(&ctx).is_none());
}

#[test]
fn test_opencode_list_placed_with_skills() {
    let target = OpenCodeTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    let skill_path = project_root
        .join(".opencode")
        .join("skills")
        .join("skill-1");
    std::fs::create_dir_all(&skill_path).unwrap();
    std::fs::write(skill_path.join("SKILL.md"), "# Skill 1").unwrap();

    let result = target
        .list_placed(ComponentKind::Skill, Scope::Project, project_root)
        .unwrap();
    assert_eq!(result, vec!["skill-1".to_string()]);
}

#[test]
fn test_opencode_list_placed_empty_when_no_skill_md() {
    let target = OpenCodeTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    let skill_path = project_root
        .join(".opencode")
        .join("skills")
        .join("empty-skill");
    std::fs::create_dir_all(&skill_path).unwrap();

    let result = target
        .list_placed(ComponentKind::Skill, Scope::Project, project_root)
        .unwrap();
    assert!(result.is_empty());
}

#[test]
fn skill_overwrite_error_returns_none_when_target_does_not_exist() {
    let temp = TempDir::new().unwrap();
    let target_path = temp.path().join("skills").join("my-skill");
    assert!(OpenCodeTarget::skill_overwrite_error(&target_path, temp.path()).is_none());
}

#[test]
fn skill_overwrite_error_returns_error_when_unowned() {
    let temp = TempDir::new().unwrap();
    let target_path = temp.path().join("skills").join("my-skill");
    std::fs::create_dir_all(&target_path).unwrap();
    std::fs::write(target_path.join("SKILL.md"), "---\nname: my-skill\n---\n").unwrap();

    let plugin_root = TempDir::new().unwrap();
    let result = OpenCodeTarget::skill_overwrite_error(&target_path, plugin_root.path());
    assert!(result.is_some());
    assert!(result.unwrap().contains("already exists"));
}

#[test]
fn skill_overwrite_error_returns_none_when_owned() {
    let temp = TempDir::new().unwrap();
    let target_path = temp.path().join("skills").join("my-skill");
    std::fs::create_dir_all(&target_path).unwrap();

    let plugin_root = TempDir::new().unwrap();
    let mut meta = crate::plugin::meta::PluginMeta::default();
    meta.add_managed_file("opencode", &target_path);
    crate::plugin::meta::write_meta(plugin_root.path(), &meta).unwrap();

    assert!(OpenCodeTarget::skill_overwrite_error(&target_path, plugin_root.path()).is_none());
}

#[test]
fn personal_root_prefers_xdg_over_home() {
    let _lock = env_lock().lock().unwrap();
    let guard = EnvGuard::clear(&["XDG_CONFIG_HOME", "HOME"]);
    guard.set("HOME", "/home/u");
    guard.set("XDG_CONFIG_HOME", "/custom/xdg");

    assert_eq!(
        OpenCodeTarget::personal_root(),
        Path::new("/custom/xdg/opencode")
    );
}

#[test]
fn test_opencode_placement_agent_project_uses_flattened_md() {
    let target = OpenCodeTarget::new();
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
        Path::new("/project/.opencode/agents/my-plugin_my-agent.md")
    );
}

#[test]
fn test_opencode_placement_agent_personal_default_home() {
    let _lock = env_lock().lock().unwrap();
    let guard = EnvGuard::clear(&["XDG_CONFIG_HOME", "HOME"]);
    guard.set("HOME", "/home/u");

    let target = OpenCodeTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Agent, "my-plugin_my-agent"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();
    assert_eq!(
        location.as_path(),
        Path::new("/home/u/.config/opencode/agents/my-plugin_my-agent.md")
    );
}

#[test]
fn test_opencode_placement_agent_personal_respects_xdg_config_home() {
    let _lock = env_lock().lock().unwrap();
    let guard = EnvGuard::clear(&["XDG_CONFIG_HOME", "HOME"]);
    guard.set("HOME", "/home/u");
    guard.set("XDG_CONFIG_HOME", "/xdg/config");

    let target = OpenCodeTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Agent, "my-plugin_my-agent"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();
    assert_eq!(
        location.as_path(),
        Path::new("/xdg/config/opencode/agents/my-plugin_my-agent.md")
    );
}

#[test]
fn test_opencode_placement_command_project_uses_flattened_md() {
    let target = OpenCodeTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Command, "my-plugin_my-command"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();
    assert!(location.is_file());
    assert_eq!(
        location.as_path(),
        Path::new("/project/.opencode/commands/my-plugin_my-command.md")
    );
}

#[test]
fn test_opencode_placement_command_personal_default_home() {
    let _lock = env_lock().lock().unwrap();
    let guard = EnvGuard::clear(&["XDG_CONFIG_HOME", "HOME"]);
    guard.set("HOME", "/home/u");

    let target = OpenCodeTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Command, "my-plugin_my-command"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();
    assert_eq!(
        location.as_path(),
        Path::new("/home/u/.config/opencode/commands/my-plugin_my-command.md")
    );
}

#[test]
fn test_opencode_list_placed_with_agents() {
    let target = OpenCodeTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    let agents_dir = project_root.join(".opencode").join("agents");
    std::fs::create_dir_all(&agents_dir).unwrap();
    std::fs::write(agents_dir.join("plugin_agent-1.md"), "# Agent 1").unwrap();
    // `.agent.md` はプレーン Markdown ではないため列挙しない
    std::fs::write(agents_dir.join("legacy.agent.md"), "# Legacy").unwrap();

    let result = target
        .list_placed(ComponentKind::Agent, Scope::Project, project_root)
        .unwrap();
    assert_eq!(result, vec!["plugin_agent-1".to_string()]);
}

#[test]
fn test_opencode_list_placed_with_commands() {
    let target = OpenCodeTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    let commands_dir = project_root.join(".opencode").join("commands");
    std::fs::create_dir_all(&commands_dir).unwrap();
    std::fs::write(commands_dir.join("plugin_cmd-1.md"), "# Command 1").unwrap();
    // `.prompt.md` はプレーン Markdown ではないため列挙しない
    std::fs::write(commands_dir.join("legacy.prompt.md"), "# Legacy").unwrap();

    let result = target
        .list_placed(ComponentKind::Command, Scope::Project, project_root)
        .unwrap();
    assert_eq!(result, vec!["plugin_cmd-1".to_string()]);
}

#[test]
fn test_opencode_placement_instruction_project() {
    let target = OpenCodeTarget::new();
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
    assert_eq!(location.as_path(), Path::new("/project/AGENTS.md"));
}

#[test]
fn test_opencode_placement_instruction_personal_default_home() {
    let _lock = env_lock().lock().unwrap();
    let guard = EnvGuard::clear(&["XDG_CONFIG_HOME", "HOME"]);
    guard.set("HOME", "/home/u");

    let target = OpenCodeTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Instruction, "test"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();
    assert!(location.is_file());
    assert_eq!(
        location.as_path(),
        Path::new("/home/u/.config/opencode/AGENTS.md")
    );
}

#[test]
fn test_opencode_placement_instruction_personal_respects_xdg_config_home() {
    let _lock = env_lock().lock().unwrap();
    let guard = EnvGuard::clear(&["XDG_CONFIG_HOME", "HOME"]);
    guard.set("HOME", "/home/u");
    guard.set("XDG_CONFIG_HOME", "/xdg/config");

    let target = OpenCodeTarget::new();
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Instruction, "test"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Personal),
        project: ProjectContext::new(project_root),
    };
    let location = target.placement_location(&ctx).unwrap();
    assert!(location.is_file());
    assert_eq!(
        location.as_path(),
        Path::new("/xdg/config/opencode/AGENTS.md")
    );
}

#[test]
fn test_opencode_list_placed_instruction_project_exists() {
    let target = OpenCodeTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    std::fs::write(project_root.join("AGENTS.md"), "# Agents").unwrap();

    let result = target
        .list_placed(ComponentKind::Instruction, Scope::Project, project_root)
        .unwrap();
    assert_eq!(result, vec!["AGENTS.md".to_string()]);
}

#[test]
fn test_opencode_list_placed_instruction_project_missing() {
    let target = OpenCodeTarget::new();
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    let result = target
        .list_placed(ComponentKind::Instruction, Scope::Project, project_root)
        .unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_opencode_list_placed_instruction_personal_exists() {
    let _lock = env_lock().lock().unwrap();
    let home = TempDir::new().unwrap();
    let guard = EnvGuard::clear(&["XDG_CONFIG_HOME", "HOME"]);
    guard.set("HOME", home.path());

    let agents_path = home
        .path()
        .join(".config")
        .join("opencode")
        .join("AGENTS.md");
    std::fs::create_dir_all(agents_path.parent().unwrap()).unwrap();
    std::fs::write(&agents_path, "# Personal Agents").unwrap();

    let target = OpenCodeTarget::new();
    let project_root = Path::new("/project");
    let result = target
        .list_placed(ComponentKind::Instruction, Scope::Personal, project_root)
        .unwrap();
    assert_eq!(result, vec!["AGENTS.md".to_string()]);
}

#[test]
fn test_opencode_project_instruction_path_matches_codex_and_cursor() {
    let project_root = Path::new("/project");
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");

    let ctx = PlacementContext {
        component: ComponentRef::new(ComponentKind::Instruction, "test"),
        origin: &origin,
        scope: PlacementScope::new(Scope::Project),
        project: ProjectContext::new(project_root),
    };

    let opencode = OpenCodeTarget::new().placement_location(&ctx).unwrap();
    let codex = CodexTarget::new().placement_location(&ctx).unwrap();
    let cursor = CursorTarget::new().placement_location(&ctx).unwrap();

    assert_eq!(opencode.as_path(), Path::new("/project/AGENTS.md"));
    assert_eq!(opencode.as_path(), codex.as_path());
    assert_eq!(opencode.as_path(), cursor.as_path());
}
