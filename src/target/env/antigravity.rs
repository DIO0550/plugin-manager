//! Google Antigravity ターゲット実装

use crate::component::{ComponentKind, PlacementContext, PlacementLocation, Scope};
use crate::error::Result;
use crate::target::filter::filter_skill_dir;
use crate::target::list_helpers::scan_and_filter;
use crate::target::paths::home_dir;
use crate::target::placement_helpers::skill_dir;
use crate::target::scope_support::{allows_scope, ScopeSupport};
use crate::target::{Target, TargetKind};
use std::path::{Path, PathBuf};

/// Antigravity のパス定数（Phase F: 薄いレイアウト）。
struct AntigravityLayout {
    personal_parent: &'static str,
    personal_child: &'static str,
    project_subdir: &'static str,
}

const LAYOUT: AntigravityLayout = AntigravityLayout {
    personal_parent: ".gemini",
    personal_child: "antigravity",
    project_subdir: ".agent",
};

const SUPPORTED: &[ComponentKind] = &[ComponentKind::Skill];

const CAPABILITIES: &[(ComponentKind, ScopeSupport)] =
    &[(ComponentKind::Skill, ScopeSupport::Both)];

/// Google Antigravity ターゲット
pub struct AntigravityTarget;

impl AntigravityTarget {
    pub fn new() -> Self {
        Self
    }

    fn base_dir(scope: Scope, project_root: &Path) -> PathBuf {
        match scope {
            Scope::Personal => home_dir()
                .join(LAYOUT.personal_parent)
                .join(LAYOUT.personal_child),
            Scope::Project => project_root.join(LAYOUT.project_subdir),
        }
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

        let base = Self::base_dir(scope, context.project_root());
        Some(match kind {
            ComponentKind::Skill => skill_dir(&base, context.name()),
            _ => return None,
        })
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
            ComponentKind::Skill => scan_and_filter(&base, "skills", filter_skill_dir),
            _ => Ok(vec![]),
        }
    }
}

#[cfg(test)]
#[path = "antigravity_test.rs"]
mod tests;
