//! TargetKind 配置パス API のテスト

use super::*;
use crate::component::ComponentKind;
use crate::placement_names::COPILOT_COMMAND_SUBDIR;
use std::path::Path;

#[test]
fn instruction_filenames_match_all_constant() {
    assert_instruction_filenames_consistent();
}

#[test]
fn instruction_filename_per_target() {
    assert_eq!(TargetKind::Codex.instruction_filename(), Some("AGENTS.md"));
    assert_eq!(TargetKind::Cursor.instruction_filename(), Some("AGENTS.md"));
    assert_eq!(
        TargetKind::Copilot.instruction_filename(),
        Some("copilot-instructions.md")
    );
    assert_eq!(
        TargetKind::GeminiCli.instruction_filename(),
        Some("GEMINI.md")
    );
    assert_eq!(TargetKind::Antigravity.instruction_filename(), None);
    assert_eq!(
        TargetKind::OpenCode.instruction_filename(),
        Some("AGENTS.md")
    );
}

#[test]
fn opencode_bases() {
    use std::sync::{Mutex, OnceLock};
    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }
    let _lock = env_lock().lock().unwrap();
    let prev_xdg = std::env::var_os("XDG_CONFIG_HOME");
    std::env::remove_var("XDG_CONFIG_HOME");

    let home = Path::new("/home/u");
    let root = Path::new("/proj");
    assert_eq!(
        TargetKind::OpenCode.personal_base(home),
        home.join(".config").join("opencode")
    );
    assert_eq!(
        TargetKind::OpenCode.project_base(root),
        root.join(".opencode")
    );

    std::env::set_var("XDG_CONFIG_HOME", "/xdg/config");
    assert_eq!(
        TargetKind::OpenCode.personal_base(home),
        Path::new("/xdg/config/opencode")
    );

    match prev_xdg {
        Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
        None => std::env::remove_var("XDG_CONFIG_HOME"),
    }
}

#[test]
fn cleanup_specs_opencode() {
    use std::sync::{Mutex, OnceLock};
    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }
    let _lock = env_lock().lock().unwrap();
    let prev_xdg = std::env::var_os("XDG_CONFIG_HOME");
    std::env::remove_var("XDG_CONFIG_HOME");

    let home = Path::new("/home/u");
    let root = Path::new("/proj");
    let specs = TargetKind::OpenCode.cleanup_specs(Some(home), root);
    let base = home.join(".config").join("opencode");
    assert!(specs.contains(&(base.clone(), "skills")));
    assert!(specs.contains(&(base.clone(), "agents")));
    assert!(specs.contains(&(base, "commands")));
    assert!(specs.contains(&(root.join(".opencode"), "skills")));
    assert!(specs.contains(&(root.join(".opencode"), "agents")));
    assert!(specs.contains(&(root.join(".opencode"), "commands")));

    match prev_xdg {
        Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
        None => std::env::remove_var("XDG_CONFIG_HOME"),
    }
}

#[test]
fn placement_subdir_copilot_command_is_prompts() {
    assert_eq!(
        TargetKind::Copilot.placement_subdir(ComponentKind::Command),
        Some(COPILOT_COMMAND_SUBDIR)
    );
    assert_eq!(
        TargetKind::Cursor.placement_subdir(ComponentKind::Command),
        ComponentKind::Command.default_subdir()
    );
    assert_eq!(
        TargetKind::Codex.placement_subdir(ComponentKind::Skill),
        ComponentKind::Skill.default_subdir()
    );
    assert_eq!(
        TargetKind::Codex.placement_subdir(ComponentKind::Instruction),
        None
    );
}

#[test]
fn cleanup_specs_codex_without_home() {
    let root = Path::new("/proj");
    let specs = TargetKind::Codex.cleanup_specs(None, root);
    assert_eq!(
        specs,
        vec![
            (root.join(".codex"), "agents"),
            (root.join(".codex"), "skills"),
        ]
    );
}

#[test]
fn cleanup_specs_copilot_with_home() {
    let home = Path::new("/home/u");
    let root = Path::new("/proj");
    let specs = TargetKind::Copilot.cleanup_specs(Some(home), root);
    assert!(specs.contains(&(home.join(".copilot"), "agents")));
    assert!(specs.contains(&(home.join(".copilot"), "hooks")));
    assert!(specs.contains(&(root.join(".github"), "prompts")));
    assert!(specs.contains(&(root.join(".github"), "skills")));
}

#[test]
fn cleanup_specs_antigravity_personal_nested() {
    let home = Path::new("/home/u");
    let root = Path::new("/proj");
    let specs = TargetKind::Antigravity.cleanup_specs(Some(home), root);
    assert!(specs.contains(&(home.join(".gemini").join("antigravity"), "skills")));
    assert!(specs.contains(&(root.join(".agent"), "skills")));
}
