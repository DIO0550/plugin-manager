//! GitHub Copilot ターゲット実装

use crate::component::{ComponentKind, PlacementContext, PlacementLocation, Scope};
use crate::error::Result;
use crate::target::filter::{filter_json_suffix, filter_skill_dir, filter_suffix_file};
use crate::target::list_helpers::{list_instruction_at, scan_and_filter};
use crate::target::paths::base_dir;
use crate::target::placement_helpers::{agent_file, instruction_under_base, named_file, skill_dir};
use crate::target::scope_support::{allows_scope, ScopeSupport};
use crate::target::{Target, TargetKind};
use std::path::{Path, PathBuf};

struct CopilotLayout {
    personal_subdir: &'static str,
    project_subdir: &'static str,
    instruction_file: &'static str,
}

const LAYOUT: CopilotLayout = CopilotLayout {
    personal_subdir: ".copilot",
    project_subdir: ".github",
    instruction_file: "copilot-instructions.md",
};

const SUPPORTED: &[ComponentKind] = &[
    ComponentKind::Skill,
    ComponentKind::Agent,
    ComponentKind::Command,
    ComponentKind::Instruction,
    ComponentKind::Hook,
];

/// Agent / Hook は両スコープ、それ以外は Project のみ（現行 can_place と同等）。
const CAPABILITIES: &[(ComponentKind, ScopeSupport)] = &[
    (ComponentKind::Skill, ScopeSupport::ProjectOnly),
    (ComponentKind::Agent, ScopeSupport::Both),
    (ComponentKind::Command, ScopeSupport::ProjectOnly),
    (ComponentKind::Instruction, ScopeSupport::ProjectOnly),
    (ComponentKind::Hook, ScopeSupport::Both),
];

/// GitHub Copilot ターゲット
pub struct CopilotTarget;

impl CopilotTarget {
    pub fn new() -> Self {
        Self
    }

    fn base_dir(scope: Scope, project_root: &Path) -> PathBuf {
        base_dir(
            scope,
            project_root,
            LAYOUT.personal_subdir,
            LAYOUT.project_subdir,
        )
    }
}

impl Default for CopilotTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl Target for CopilotTarget {
    fn display_name(&self) -> &'static str {
        "GitHub Copilot"
    }

    fn kind(&self) -> TargetKind {
        TargetKind::Copilot
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
            ComponentKind::Skill => skill_dir(&base, name),
            ComponentKind::Agent => agent_file(&base, name),
            ComponentKind::Command => named_file(&base, "prompts", name, ".prompt.md"),
            ComponentKind::Instruction => instruction_under_base(&base, LAYOUT.instruction_file),
            ComponentKind::Hook => named_file(&base, "hooks", name, ".json"),
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

        if kind == ComponentKind::Instruction {
            return Ok(list_instruction_at(
                &base.join(LAYOUT.instruction_file),
                LAYOUT.instruction_file,
            ));
        }

        match kind {
            ComponentKind::Skill => scan_and_filter(&base, "skills", filter_skill_dir),
            ComponentKind::Agent => {
                scan_and_filter(&base, "agents", |c| filter_suffix_file(c, ".agent.md"))
            }
            ComponentKind::Command => {
                scan_and_filter(&base, "prompts", |c| filter_suffix_file(c, ".prompt.md"))
            }
            ComponentKind::Hook => scan_and_filter(&base, "hooks", filter_json_suffix),
            _ => Ok(vec![]),
        }
    }
}

#[cfg(test)]
#[path = "copilot_test.rs"]
mod tests;
