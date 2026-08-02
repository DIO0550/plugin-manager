//! OpenCode ターゲット実装（Skills 配置 — Phase 2 / #418）
//!
//! Agents / Commands / Instructions は後続 Issue（#419 / #420）で追加する。
//! Hooks は JS/TS Plugin モデルのため対象外。

use crate::component::{ComponentKind, PlacementContext, PlacementLocation, Scope};
use crate::env::EnvVar;
use crate::error::Result;
use crate::placement_names::{
    OPENCODE_PERSONAL_CHILD, OPENCODE_PERSONAL_PARENT, OPENCODE_PROJECT_SUBDIR,
};
use crate::target::filter::filter_skill_dir;
use crate::target::list_helpers::scan_and_filter;
use crate::target::paths::home_dir;
use crate::target::placement_helpers::skill_dir;
use crate::target::scope_support::{allows_scope, ScopeSupport};
use crate::target::{PostPlaceOutcome, Target, TargetKind};
use std::path::{Path, PathBuf};

const SUPPORTED: &[ComponentKind] = &[ComponentKind::Skill];

const CAPABILITIES: &[(ComponentKind, ScopeSupport)] =
    &[(ComponentKind::Skill, ScopeSupport::Both)];

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

    fn base_dir(scope: Scope, project_root: &Path) -> PathBuf {
        match scope {
            Scope::Personal => Self::personal_root(),
            Scope::Project => project_root.join(OPENCODE_PROJECT_SUBDIR),
        }
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

/// `$XDG_CONFIG_HOME/opencode`（空・未設定時は `home/.config/opencode`）。
pub(crate) fn personal_root_from_env(home: &Path) -> PathBuf {
    if let Some(xdg) = EnvVar::get("XDG_CONFIG_HOME").filter(|s| !s.trim().is_empty()) {
        return PathBuf::from(xdg.trim()).join(OPENCODE_PERSONAL_CHILD);
    }
    home.join(OPENCODE_PERSONAL_PARENT)
        .join(OPENCODE_PERSONAL_CHILD)
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

        let base = Self::base_dir(scope, context.project_root());
        match kind {
            // OpenCode は frontmatter `name` と親フォルダ名の一致を要求するため、
            // Skill は original_name で配置する（Cursor #377 と同型）。
            ComponentKind::Skill => {
                let dir_name = context.original_name().filter(|n| !n.is_empty())?;
                Some(skill_dir(&base, dir_name))
            }
            _ => None,
        }
    }

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

    fn list_placed(
        &self,
        kind: ComponentKind,
        scope: Scope,
        project_root: &Path,
    ) -> Result<Vec<String>> {
        if !self.can_place_scope(kind, scope) {
            return Ok(vec![]);
        }

        let base = Self::base_dir(scope, project_root);
        match kind {
            ComponentKind::Skill => {
                scan_and_filter(&base, ComponentKind::Skill.plural(), filter_skill_dir)
            }
            _ => Ok(vec![]),
        }
    }
}

#[cfg(test)]
#[path = "opencode_test.rs"]
mod tests;
