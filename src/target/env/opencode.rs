//! OpenCode ターゲット実装（Skills / Agents / Commands / Instructions）
//!
//! Hooks は JS/TS Plugin モデルのため対象外。

use crate::component::{ComponentKind, PlacementContext};
use crate::placement_names::{OPENCODE_PERSONAL_CHILD, OPENCODE_PROJECT_SUBDIR};
use crate::target::paths::{home_dir, xdg_config_child};
use crate::target::scope_support::capabilities;
use crate::target::{
    impl_target_layout, FileSuffixStyle, HookLayout, InstructionPlacement, PersonalRoot,
    PostPlaceOutcome, SkillDirName, Target, TargetKind, TargetLayout,
};
use std::path::{Path, PathBuf};

pub(crate) const LAYOUT: TargetLayout = TargetLayout {
    kind: TargetKind::OpenCode,
    capabilities: capabilities!(
        Skill => Both,
        Agent => Both,
        Command => Both,
        Instruction => Both,
    ),
    personal_root: PersonalRoot::XdgConfig,
    project_subdir: OPENCODE_PROJECT_SUBDIR,
    skill_dir_name: SkillDirName::Original,
    agent_files: Some(FileSuffixStyle::PlainMarkdown),
    command_files: Some(FileSuffixStyle::PlainMarkdown),
    instruction_at: Some(InstructionPlacement::ProjectRootOrEnvRoot),
    hooks: HookLayout::Unsupported,
};

/// OpenCode ターゲット
pub struct OpenCodeTarget;

impl OpenCodeTarget {
    pub fn new() -> Self {
        Self
    }

    /// Personal ルート（`$XDG_CONFIG_HOME/opencode`、未設定時 `~/.config/opencode`）。
    pub(crate) fn personal_root() -> PathBuf {
        personal_root_from_env(&home_dir())
    }

    pub fn skill_overwrite_error(target_path: &Path, plugin_root: &Path) -> Option<String> {
        if !Self::path_conflicts_with_unowned(target_path, plugin_root) {
            return None;
        }
        Some(format!(
            "{} already exists and is not managed by this plugin. \
             Refusing to overwrite; remove it or uninstall the owning plugin first.",
            target_path.display()
        ))
    }

    fn path_conflicts_with_unowned(target_path: &Path, plugin_root: &Path) -> bool {
        if !target_path.exists() {
            return false;
        }
        let already_owned = crate::plugin::meta::load_meta(plugin_root)
            .map(|meta| meta.manages_file("opencode", target_path))
            .unwrap_or(false);
        !already_owned
    }
}

/// `$XDG_CONFIG_HOME/opencode`（未設定・空・相対パス時は `home/.config/opencode`）。
fn personal_root_from_env(home: &Path) -> PathBuf {
    xdg_config_child(home, OPENCODE_PERSONAL_CHILD)
}

impl Default for OpenCodeTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl Target for OpenCodeTarget {
    fn display_name(&self) -> &'static str {
        "OpenCode"
    }

    fn kind(&self) -> TargetKind {
        TargetKind::OpenCode
    }

    impl_target_layout!(LAYOUT);

    fn pre_place_check(
        &self,
        context: &PlacementContext,
        target_path: &Path,
        plugin_root: &Path,
    ) -> std::result::Result<(), String> {
        if context.kind() == ComponentKind::Skill {
            if let Some(error) = Self::skill_overwrite_error(target_path, plugin_root) {
                return Err(error);
            }
        }
        Ok(())
    }

    fn post_place(
        &self,
        context: &PlacementContext,
        deployed_path: &Path,
        plugin_root: &Path,
        _enable_feature_flag: bool,
    ) -> PostPlaceOutcome {
        if context.kind() == ComponentKind::Skill {
            crate::install::record_opencode_skill_ownership(plugin_root, deployed_path);
        }
        PostPlaceOutcome::default()
    }
}

#[cfg(test)]
#[path = "opencode_test.rs"]
mod tests;
