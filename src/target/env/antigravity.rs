//! Google Antigravity ターゲット実装（Skills / Hooks）

use crate::component::{
    Component, ComponentKind, FileOperation, PlacementContext, Scope, ScopedPath,
};
use crate::placement_names::{
    ANTIGRAVITY_HOOKS_FILE, ANTIGRAVITY_LEGACY_PERSONAL_CHILD, ANTIGRAVITY_LEGACY_PROJECT_SUBDIR,
    ANTIGRAVITY_PERSONAL_PARENT, ANTIGRAVITY_SKILLS_PERSONAL_CHILD,
    ANTIGRAVITY_SKILLS_PROJECT_SUBDIR,
};
use crate::target::paths::home_dir;
use crate::target::scope_support::capabilities;
use crate::target::{
    impl_target_layout, HookLayout, PersonalRoot, PostPlaceOutcome, SkillDirName, Target,
    TargetKind, TargetLayout,
};
use std::path::{Path, PathBuf};

pub(crate) const LAYOUT: TargetLayout = TargetLayout {
    kind: TargetKind::Antigravity,
    capabilities: capabilities!(
        Skill => Both,
        Hook => Both,
    ),
    personal_root: PersonalRoot::HomeNested {
        parent: ANTIGRAVITY_PERSONAL_PARENT,
        child: ANTIGRAVITY_SKILLS_PERSONAL_CHILD,
    },
    project_subdir: ANTIGRAVITY_SKILLS_PROJECT_SUBDIR,
    skill_dir_name: SkillDirName::Original,
    agent_files: None,
    command_files: None,
    instruction_at: None,
    hooks: HookLayout::SingleFile {
        filename: ANTIGRAVITY_HOOKS_FILE,
    },
};

/// Google Antigravity ターゲット
pub struct AntigravityTarget;

impl AntigravityTarget {
    pub fn new() -> Self {
        Self
    }

    fn legacy_skills_base_dir(scope: Scope, project_root: &Path) -> PathBuf {
        match scope {
            Scope::Personal => home_dir()
                .join(ANTIGRAVITY_PERSONAL_PARENT)
                .join(ANTIGRAVITY_LEGACY_PERSONAL_CHILD),
            Scope::Project => project_root.join(ANTIGRAVITY_LEGACY_PROJECT_SUBDIR),
        }
    }

    fn legacy_skill_path(scope: Scope, project_root: &Path, flattened_name: &str) -> PathBuf {
        Self::legacy_skills_base_dir(scope, project_root)
            .join(ComponentKind::Skill.plural())
            .join(flattened_name)
    }

    fn remove_legacy_skill_dir(
        scope: Scope,
        project_root: &Path,
        flattened_name: &str,
        current_path: &Path,
    ) -> bool {
        let legacy_path = Self::legacy_skill_path(scope, project_root, flattened_name);
        if !legacy_path.exists() || legacy_path == current_path {
            return false;
        }

        match std::fs::remove_dir_all(&legacy_path) {
            Ok(()) => true,
            Err(error) => {
                eprintln!(
                    "Warning: failed to remove legacy Antigravity skill path {}: {}",
                    legacy_path.display(),
                    error
                );
                false
            }
        }
    }

    /// Antigravity は 1 スコープにつき単一の `hooks.json` を読むため、複数 Hook を拒否する。
    pub fn hook_component_conflict_error(components: &[Component]) -> Option<String> {
        let hook_count = components
            .iter()
            .filter(|component| component.kind == ComponentKind::Hook)
            .count();

        (hook_count > 1).then(|| {
            format!(
                "Antigravity target supports a single hooks.json per scope; {} Hook components would overwrite each other. Select one Hook component or wait for merge support.",
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

    fn path_conflicts_with_unowned(target_path: &Path, plugin_root: &Path) -> bool {
        if !target_path.exists() {
            return false;
        }
        let already_owned = crate::plugin::meta::load_meta(plugin_root)
            .map(|meta| meta.manages_file("antigravity", target_path))
            .unwrap_or(false);
        !already_owned
    }

    fn skill_overwrite_error(target_path: &Path, plugin_root: &Path) -> Option<String> {
        if !Self::path_conflicts_with_unowned(target_path, plugin_root) {
            return None;
        }
        Some(format!(
            "{} already exists and is not managed by this plugin. \
             Refusing to overwrite; remove it or uninstall the owning plugin first.",
            target_path.display()
        ))
    }
}

impl Default for AntigravityTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl Target for AntigravityTarget {
    fn display_name(&self) -> &'static str {
        "Google Antigravity"
    }

    fn kind(&self) -> TargetKind {
        TargetKind::Antigravity
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
                crate::install::record_hook_file_ownership(
                    plugin_root,
                    deployed_path,
                    "antigravity",
                );
            }
            ComponentKind::Skill => {
                crate::install::record_managed_file_ownership(
                    plugin_root,
                    deployed_path,
                    "antigravity",
                );
                Self::remove_legacy_skill_dir(
                    context.scope(),
                    context.project_root(),
                    context.name(),
                    deployed_path,
                );
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

        let legacy_path =
            Self::legacy_skill_path(context.scope(), context.project_root(), context.name());
        if !legacy_path.exists() {
            return Ok(vec![]);
        }

        let scoped = ScopedPath::new(legacy_path, context.project_root())
            .map_err(|e| format!("Path validation failed: {}", e))?;
        Ok(vec![FileOperation::RemoveDir { path: scoped }])
    }
}

#[cfg(test)]
#[path = "antigravity_test.rs"]
mod tests;
