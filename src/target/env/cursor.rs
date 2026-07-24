//! Cursor ターゲット実装（Skills / Agents / Commands / Instructions / Hooks）

use crate::component::{
    Component, ComponentKind, FileOperation, PlacementContext, PlacementLocation, Scope, ScopedPath,
};
use crate::error::Result;
use crate::target::filter::{filter_exact_file, filter_plain_markdown, filter_skill_dir};
use crate::target::list_helpers::{list_instruction_at, scan_and_filter, scan_and_filter_in};
use crate::target::paths::base_dir;
use crate::target::placement_helpers::{named_file, skill_dir};
use crate::target::scope_support::{allows_scope, ScopeSupport};
use crate::target::{PostPlaceOutcome, Target, TargetKind};
use std::path::{Path, PathBuf};

struct CursorLayout {
    subdir: &'static str,
    instruction_file: &'static str,
    hooks_file: &'static str,
}

const LAYOUT: CursorLayout = CursorLayout {
    subdir: ".cursor",
    instruction_file: "AGENTS.md",
    hooks_file: "hooks.json",
};

const SUPPORTED: &[ComponentKind] = &[
    ComponentKind::Skill,
    ComponentKind::Agent,
    ComponentKind::Command,
    ComponentKind::Instruction,
    ComponentKind::Hook,
];

/// Instructions は Project のみ。それ以外は両スコープ。
const CAPABILITIES: &[(ComponentKind, ScopeSupport)] = &[
    (ComponentKind::Skill, ScopeSupport::Both),
    (ComponentKind::Agent, ScopeSupport::Both),
    (ComponentKind::Command, ScopeSupport::Both),
    (ComponentKind::Instruction, ScopeSupport::ProjectOnly),
    (ComponentKind::Hook, ScopeSupport::Both),
];

/// Cursor ターゲット
pub struct CursorTarget;

impl CursorTarget {
    pub fn new() -> Self {
        Self
    }

    fn base_dir(scope: Scope, project_root: &Path) -> PathBuf {
        base_dir(scope, project_root, LAYOUT.subdir, LAYOUT.subdir)
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
        Self::base_dir(scope, project_root)
            .join("skills")
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

        let project_root = context.project_root();
        let base = Self::base_dir(scope, project_root);
        let name = context.name();

        Some(match kind {
            // Cursor は frontmatter `name` と親フォルダ名の一致を要求するため、
            // Skill のみ元名で配置する（Issue #377）。`original_name` 未設定なら配置不可。
            ComponentKind::Skill => {
                let dir_name = context.original_name().filter(|n| !n.is_empty())?;
                skill_dir(&base, dir_name)
            }
            ComponentKind::Agent => named_file(&base, "agents", name, ".md"),
            ComponentKind::Command => named_file(&base, "commands", name, ".md"),
            ComponentKind::Instruction => {
                PlacementLocation::file(project_root.join(LAYOUT.instruction_file))
            }
            ComponentKind::Hook => PlacementLocation::file(base.join(LAYOUT.hooks_file)),
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

    fn list_placed(
        &self,
        kind: ComponentKind,
        scope: Scope,
        project_root: &Path,
    ) -> Result<Vec<String>> {
        if !self.can_place_scope(kind, scope) {
            return Ok(vec![]);
        }

        if kind == ComponentKind::Instruction {
            return Ok(list_instruction_at(
                &project_root.join(LAYOUT.instruction_file),
                LAYOUT.instruction_file,
            ));
        }

        let base = Self::base_dir(scope, project_root);
        match kind {
            ComponentKind::Skill => scan_and_filter(&base, "skills", filter_skill_dir),
            ComponentKind::Agent => scan_and_filter(&base, "agents", filter_plain_markdown),
            ComponentKind::Command => scan_and_filter(&base, "commands", filter_plain_markdown),
            ComponentKind::Hook => {
                scan_and_filter_in(&base, |c| filter_exact_file(c, LAYOUT.hooks_file, "hooks"))
            }
            _ => Ok(vec![]),
        }
    }
}

#[cfg(test)]
#[path = "cursor_test.rs"]
mod tests;
