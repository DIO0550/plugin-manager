//! Google Antigravity ターゲット実装（Skills / Hooks）

use crate::component::{
    Component, ComponentKind, FileOperation, PlacementContext, PlacementLocation, Scope, ScopedPath,
};
use crate::error::Result;
use crate::placement_names::{
    ANTIGRAVITY_HOOKS_FILE, ANTIGRAVITY_HOOKS_PERSONAL_CHILD, ANTIGRAVITY_HOOKS_PROJECT_SUBDIR,
    ANTIGRAVITY_LEGACY_PERSONAL_CHILD, ANTIGRAVITY_LEGACY_PROJECT_SUBDIR,
    ANTIGRAVITY_PERSONAL_PARENT, ANTIGRAVITY_SKILLS_PERSONAL_CHILD,
    ANTIGRAVITY_SKILLS_PROJECT_SUBDIR,
};
use crate::target::filter::{filter_exact_file, filter_skill_dir};
use crate::target::list_helpers::{scan_and_filter, scan_and_filter_in};
use crate::target::paths::home_dir;
use crate::target::placement_helpers::skill_dir;
use crate::target::scope_support::{allows_scope, ScopeSupport};
use crate::target::{PostPlaceOutcome, Target, TargetKind};
use std::path::{Path, PathBuf};

/// Antigravity のパス定数（#339: placement_names を正とする）。
/// Skills と Hooks でルートが異なる（公式仕様）。
struct AntigravityLayout {
    personal_parent: &'static str,
    skills_personal_child: &'static str,
    skills_project_subdir: &'static str,
    legacy_skills_personal_child: &'static str,
    legacy_skills_project_subdir: &'static str,
    hooks_personal_child: &'static str,
    hooks_project_subdir: &'static str,
    hooks_file: &'static str,
}

const LAYOUT: AntigravityLayout = AntigravityLayout {
    personal_parent: ANTIGRAVITY_PERSONAL_PARENT,
    skills_personal_child: ANTIGRAVITY_SKILLS_PERSONAL_CHILD,
    skills_project_subdir: ANTIGRAVITY_SKILLS_PROJECT_SUBDIR,
    legacy_skills_personal_child: ANTIGRAVITY_LEGACY_PERSONAL_CHILD,
    legacy_skills_project_subdir: ANTIGRAVITY_LEGACY_PROJECT_SUBDIR,
    hooks_personal_child: ANTIGRAVITY_HOOKS_PERSONAL_CHILD,
    hooks_project_subdir: ANTIGRAVITY_HOOKS_PROJECT_SUBDIR,
    hooks_file: ANTIGRAVITY_HOOKS_FILE,
};

const SUPPORTED: &[ComponentKind] = &[ComponentKind::Skill, ComponentKind::Hook];

const CAPABILITIES: &[(ComponentKind, ScopeSupport)] = &[
    (ComponentKind::Skill, ScopeSupport::Both),
    (ComponentKind::Hook, ScopeSupport::Both),
];

/// Google Antigravity ターゲット
pub struct AntigravityTarget;

impl AntigravityTarget {
    pub fn new() -> Self {
        Self
    }

    fn skills_base_dir(scope: Scope, project_root: &Path) -> PathBuf {
        match scope {
            Scope::Personal => home_dir()
                .join(LAYOUT.personal_parent)
                .join(LAYOUT.skills_personal_child),
            Scope::Project => project_root.join(LAYOUT.skills_project_subdir),
        }
    }

    fn hooks_base_dir(scope: Scope, project_root: &Path) -> PathBuf {
        match scope {
            Scope::Personal => home_dir()
                .join(LAYOUT.personal_parent)
                .join(LAYOUT.hooks_personal_child),
            Scope::Project => project_root.join(LAYOUT.hooks_project_subdir),
        }
    }

    fn legacy_skills_base_dir(scope: Scope, project_root: &Path) -> PathBuf {
        match scope {
            Scope::Personal => home_dir()
                .join(LAYOUT.personal_parent)
                .join(LAYOUT.legacy_skills_personal_child),
            Scope::Project => project_root.join(LAYOUT.legacy_skills_project_subdir),
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

    fn supported_components(&self) -> &[ComponentKind] {
        SUPPORTED
    }

    fn can_place_scope(&self, kind: ComponentKind, scope: Scope) -> bool {
        allows_scope(CAPABILITIES, kind, scope)
    }

    fn placement_location(&self, context: &PlacementContext) -> Option<PlacementLocation> {
        let kind = context.kind();
        let scope = context.scope();
        if !self.can_place_scope(kind, scope) {
            return None;
        }

        Some(match kind {
            ComponentKind::Skill => {
                let base = Self::skills_base_dir(scope, context.project_root());
                // Antigravity は `<skills>/<skill-folder>/SKILL.md` の 1 階層を読む。
                // frontmatter の name と対応するスキャン時の元名で配置する。
                let dir_name = context.original_name().filter(|name| !name.is_empty())?;
                skill_dir(&base, dir_name)
            }
            ComponentKind::Hook => {
                let base = Self::hooks_base_dir(scope, context.project_root());
                PlacementLocation::file(base.join(LAYOUT.hooks_file))
            }
            _ => return None,
        })
    }

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

    fn list_placed(
        &self,
        kind: ComponentKind,
        scope: Scope,
        project_root: &Path,
    ) -> Result<Vec<String>> {
        if !self.can_place_scope(kind, scope) {
            return Ok(vec![]);
        }

        match kind {
            ComponentKind::Skill => {
                let base = Self::skills_base_dir(scope, project_root);
                scan_and_filter(&base, ComponentKind::Skill.plural(), filter_skill_dir)
            }
            ComponentKind::Hook => {
                let base = Self::hooks_base_dir(scope, project_root);
                scan_and_filter_in(&base, |c| {
                    filter_exact_file(c, LAYOUT.hooks_file, ComponentKind::Hook.plural())
                })
            }
            _ => Ok(vec![]),
        }
    }
}

#[cfg(test)]
#[path = "antigravity_test.rs"]
mod tests;
