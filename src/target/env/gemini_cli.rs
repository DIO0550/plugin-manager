//! Gemini CLI ターゲット実装

use crate::component::{ComponentKind, PlacementContext, PlacementLocation, Scope};
use crate::error::Result;
use crate::target::filter::filter_skill_dir;
use crate::target::list_helpers::{list_instruction_at, scan_and_filter};
use crate::target::paths::base_dir;
use crate::target::placement_helpers::{instruction_file, skill_dir};
use crate::target::scope_support::{allows_scope, ScopeSupport};
use crate::target::{Target, TargetKind};
use std::path::{Path, PathBuf};

struct GeminiLayout {
    subdir: &'static str,
    instruction_file: &'static str,
}

const LAYOUT: GeminiLayout = GeminiLayout {
    subdir: ".gemini",
    instruction_file: "GEMINI.md",
};

const SUPPORTED: &[ComponentKind] = &[ComponentKind::Skill, ComponentKind::Instruction];

const CAPABILITIES: &[(ComponentKind, ScopeSupport)] = &[
    (ComponentKind::Skill, ScopeSupport::Both),
    (ComponentKind::Instruction, ScopeSupport::Both),
];

/// Gemini CLI ターゲット
pub struct GeminiCliTarget;

impl GeminiCliTarget {
    pub fn new() -> Self {
        Self
    }

    fn base_dir(scope: Scope, project_root: &Path) -> PathBuf {
        base_dir(scope, project_root, LAYOUT.subdir, LAYOUT.subdir)
    }

    fn instruction_path(scope: Scope, project_root: &Path) -> PathBuf {
        instruction_file(
            scope,
            project_root,
            &Self::base_dir(scope, project_root),
            LAYOUT.instruction_file,
        )
        .as_path()
        .to_path_buf()
    }
}

impl Default for GeminiCliTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl Target for GeminiCliTarget {
    fn display_name(&self) -> &'static str {
        "Gemini CLI"
    }

    fn kind(&self) -> TargetKind {
        TargetKind::GeminiCli
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
        Some(match kind {
            ComponentKind::Skill => skill_dir(&base, context.name()),
            ComponentKind::Instruction => {
                instruction_file(scope, project_root, &base, LAYOUT.instruction_file)
            }
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

        if kind == ComponentKind::Instruction {
            return Ok(list_instruction_at(
                &Self::instruction_path(scope, project_root),
                LAYOUT.instruction_file,
            ));
        }

        let base = Self::base_dir(scope, project_root);
        match kind {
            ComponentKind::Skill => scan_and_filter(&base, "skills", filter_skill_dir),
            _ => Ok(vec![]),
        }
    }
}

#[cfg(test)]
#[path = "gemini_cli_test.rs"]
mod tests;
