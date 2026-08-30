//! GitHub Copilot ターゲット実装

use crate::component::{ComponentKind, PlacementContext, PlacementLocation, Scope};
use crate::error::Result;
use crate::placement_names::{
    COPILOT_COMMAND_SUBDIR, COPILOT_PERSONAL_SUBDIR, COPILOT_PROJECT_SUBDIR, INSTRUCTION_COPILOT,
};
use crate::target::filter::{filter_json_suffix, filter_skill_dir, filter_suffix_file};
use crate::target::list_helpers::{list_instruction_at, scan_and_filter};
use crate::target::paths::base_dir;
use crate::target::placement_helpers::{agent_file, instruction_under_base, named_file, skill_dir};
use crate::target::scope_support::{allows_scope, ScopeSupport};
use crate::target::{PostPlaceOutcome, Target, TargetKind};
use std::path::{Path, PathBuf};

struct CopilotLayout {
    personal_subdir: &'static str,
    project_subdir: &'static str,
    instruction_file: &'static str,
}

const LAYOUT: CopilotLayout = CopilotLayout {
    personal_subdir: COPILOT_PERSONAL_SUBDIR,
    project_subdir: COPILOT_PROJECT_SUBDIR,
    instruction_file: INSTRUCTION_COPILOT,
};

const SUPPORTED: &[ComponentKind] = &[
    ComponentKind::Skill,
    ComponentKind::Agent,
    ComponentKind::Command,
    ComponentKind::Instruction,
    ComponentKind::Hook,
];

/// Skill / Agent / Hook は両スコープ、それ以外は Project のみ。
const CAPABILITIES: &[(ComponentKind, ScopeSupport)] = &[
    (ComponentKind::Skill, ScopeSupport::Both),
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

    /// Personal Skill は元名でフラット配置するため、別プラグインの同名 Skill を保護する。
    fn personal_skill_overwrite_error(target_path: &Path, plugin_root: &Path) -> Option<String> {
        if !target_path.exists() {
            return None;
        }
        let already_owned = crate::plugin::meta::load_meta(plugin_root)
            .map(|meta| meta.manages_file("copilot", target_path))
            .unwrap_or(false);
        if already_owned {
            return None;
        }
        Some(format!(
            "{} already exists and is not managed by this plugin. Refusing to overwrite; remove it or uninstall the owning plugin first.",
            target_path.display()
        ))
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
            ComponentKind::Skill => {
                // VS Code は ~/.copilot/skills/ の直下だけを走査する。
                // Personal は namespace を挟めないため frontmatter と一致する元名を使う。
                let dir_name = match scope {
                    Scope::Personal => context.original_name().filter(|name| !name.is_empty())?,
                    Scope::Project => name,
                };
                skill_dir(&base, dir_name)
            }
            ComponentKind::Agent => agent_file(&base, name),
            ComponentKind::Command => {
                let suffix = ComponentKind::Command
                    .file_suffix()
                    .expect("Command always has a file suffix");
                named_file(&base, COPILOT_COMMAND_SUBDIR, name, suffix)
            }
            ComponentKind::Instruction => instruction_under_base(&base, LAYOUT.instruction_file),
            ComponentKind::Hook => named_file(&base, ComponentKind::Hook.plural(), name, ".json"),
        })
    }

    fn pre_place_check(
        &self,
        context: &PlacementContext,
        target_path: &Path,
        plugin_root: &Path,
    ) -> std::result::Result<(), String> {
        if context.kind() == ComponentKind::Skill && context.scope() == Scope::Personal {
            if let Some(error) = Self::personal_skill_overwrite_error(target_path, plugin_root) {
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
        if context.kind() == ComponentKind::Skill && context.scope() == Scope::Personal {
            crate::install::record_managed_file_ownership(plugin_root, deployed_path, "copilot");
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

        if kind == ComponentKind::Instruction {
            return Ok(list_instruction_at(
                &base.join(LAYOUT.instruction_file),
                LAYOUT.instruction_file,
            ));
        }

        match kind {
            ComponentKind::Skill => {
                scan_and_filter(&base, ComponentKind::Skill.plural(), filter_skill_dir)
            }
            ComponentKind::Agent => {
                let suffix = ComponentKind::Agent
                    .file_suffix()
                    .expect("Agent always has a file suffix");
                scan_and_filter(&base, ComponentKind::Agent.plural(), |c| {
                    filter_suffix_file(c, suffix)
                })
            }
            ComponentKind::Command => {
                let suffix = ComponentKind::Command
                    .file_suffix()
                    .expect("Command always has a file suffix");
                scan_and_filter(&base, COPILOT_COMMAND_SUBDIR, |c| {
                    filter_suffix_file(c, suffix)
                })
            }
            ComponentKind::Hook => {
                scan_and_filter(&base, ComponentKind::Hook.plural(), filter_json_suffix)
            }
            _ => Ok(vec![]),
        }
    }
}

#[cfg(test)]
#[path = "copilot_test.rs"]
mod tests;
