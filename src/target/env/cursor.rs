//! Cursor ターゲット実装（Skills / Agents / Commands / Instructions / Hooks）

use crate::component::{
    Component, ComponentKind, FileOperation, PlacementContext, Scope, ScopedPath,
};
use crate::placement_names::CURSOR_SUBDIR;
use crate::target::scope_support::capabilities;
use crate::target::{
    impl_target_layout, FileSuffixStyle, HookLayout, InstructionPlacement, PersonalRoot,
    PostPlaceOutcome, SkillDirName, Target, TargetKind, TargetLayout,
};
use std::path::{Path, PathBuf};

pub(crate) const LAYOUT: TargetLayout = TargetLayout {
    kind: TargetKind::Cursor,
    capabilities: capabilities!(
        Skill => Both,
        Agent => Both,
        Command => Both,
        Instruction => ProjectOnly,
        Hook => Both,
    ),
    personal_root: PersonalRoot::HomeSubdir(CURSOR_SUBDIR),
    project_subdir: CURSOR_SUBDIR,
    skill_dir_name: SkillDirName::Original,
    agent_files: Some(FileSuffixStyle::PlainMarkdown),
    command_files: Some(FileSuffixStyle::PlainMarkdown),
    instruction_at: Some(InstructionPlacement::ProjectRootOrEnvRoot),
    hooks: HookLayout::SingleFile {
        filename: "hooks.json",
    },
};

/// Cursor ターゲット
pub struct CursorTarget;

impl CursorTarget {
    pub fn new() -> Self {
        Self
    }

    /// Cursor は 1 スコープにつき単一の `hooks.json` を読むため、複数 Hook を拒否する。
    pub fn hook_component_conflict_error(components: &[Component]) -> Option<String> {
        let hook_count = components
            .iter()
            .filter(|component| component.kind == ComponentKind::Hook)
            .count();

        (hook_count > 1).then(|| {
            format!(
                "Cursor target supports a single hooks.json per scope; {} Hook components would overwrite each other. Select one Hook component or wait for merge support.",
                hook_count
            )
        })
    }

    pub fn hook_overwrite_error(target_path: &Path, plugin_root: &Path) -> Option<String> {
        if !Self::path_conflicts_with_unowned(target_path, plugin_root) {
            return None;
        }
        Some(format!(
            "{} already exists and is not managed by this plugin. \
             Refusing to overwrite; remove the file or merge it manually before re-installing.",
            target_path.display()
        ))
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
            .map(|meta| meta.manages_file("cursor", target_path))
            .unwrap_or(false);
        !already_owned
    }

    pub fn legacy_flattened_skill_path(
        scope: Scope,
        project_root: &Path,
        flattened_name: &str,
    ) -> PathBuf {
        LAYOUT
            .env_root(scope, project_root)
            .join(ComponentKind::Skill.plural())
            .join(flattened_name)
    }

    pub fn remove_legacy_flattened_skill_dir(
        scope: Scope,
        project_root: &Path,
        flattened_name: &str,
        current_path: &Path,
    ) -> bool {
        let legacy = Self::legacy_flattened_skill_path(scope, project_root, flattened_name);
        if !legacy.exists() || legacy == current_path {
            return false;
        }
        match std::fs::remove_dir_all(&legacy) {
            Ok(()) => true,
            Err(e) => {
                eprintln!(
                    "Warning: failed to remove legacy Cursor skill path {}: {}",
                    legacy.display(),
                    e
                );
                false
            }
        }
    }
}

impl Default for CursorTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl Target for CursorTarget {
    fn display_name(&self) -> &'static str {
        "Cursor"
    }

    fn kind(&self) -> TargetKind {
        TargetKind::Cursor
    }

    impl_target_layout!(LAYOUT);

    fn component_conflict_error(&self, components: &[Component]) -> Option<String> {
        Self::hook_component_conflict_error(components)
    }

    fn pre_place_check(
        &self,
        context: &PlacementContext,
        target_path: &Path,
        plugin_root: &Path,
    ) -> std::result::Result<(), String> {
        match context.kind() {
            ComponentKind::Hook => {
                if let Some(error) = Self::hook_overwrite_error(target_path, plugin_root) {
                    return Err(error);
                }
            }
            ComponentKind::Skill => {
                if let Some(error) = Self::skill_overwrite_error(target_path, plugin_root) {
                    return Err(error);
                }
            }
            _ => {}
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
        match context.kind() {
            ComponentKind::Hook => {
                crate::install::record_hook_file_ownership(plugin_root, deployed_path, "cursor");
            }
            ComponentKind::Skill => {
                crate::install::record_cursor_skill_ownership(plugin_root, deployed_path);
                if let Some(original) = context.original_name() {
                    if context.name() != original {
                        Self::remove_legacy_flattened_skill_dir(
                            context.scope(),
                            context.project_root(),
                            context.name(),
                            deployed_path,
                        );
                    }
                }
            }
            _ => {}
        }
        PostPlaceOutcome::default()
    }

    fn legacy_cleanup_operations(
        &self,
        context: &PlacementContext,
    ) -> std::result::Result<Vec<FileOperation>, String> {
        if context.kind() != ComponentKind::Skill {
            return Ok(vec![]);
        }

        let Some(original) = context.original_name() else {
            return Ok(vec![]);
        };

        if context.name() == original {
            return Ok(vec![]);
        }

        let legacy_path = Self::legacy_flattened_skill_path(
            context.scope(),
            context.project_root(),
            context.name(),
        );
        if !legacy_path.exists() {
            return Ok(vec![]);
        }

        let scoped = ScopedPath::new(legacy_path, context.project_root())
            .map_err(|e| format!("Path validation failed: {}", e))?;

        Ok(vec![FileOperation::RemoveDir { path: scoped }])
    }
}

#[cfg(test)]
#[path = "cursor_test.rs"]
mod tests;
