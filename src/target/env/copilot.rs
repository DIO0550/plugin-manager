//! GitHub Copilot ターゲット実装

use crate::component::{ComponentKind, PlacementContext, Scope};
use crate::placement_names::{COPILOT_PERSONAL_SUBDIR, COPILOT_PROJECT_SUBDIR};
use crate::target::scope_support::capabilities;
use crate::target::{
    impl_target_layout, FileSuffixStyle, HookLayout, InstructionPlacement, PersonalRoot,
    PostPlaceOutcome, SkillDirName, Target, TargetKind, TargetLayout,
};
use std::path::Path;

pub(crate) const LAYOUT: TargetLayout = TargetLayout {
    kind: TargetKind::Copilot,
    capabilities: capabilities!(
        Skill => Both,
        Agent => Both,
        Command => ProjectOnly,
        Instruction => ProjectOnly,
        Hook => Both,
    ),
    personal_root: PersonalRoot::HomeSubdir(COPILOT_PERSONAL_SUBDIR),
    project_subdir: COPILOT_PROJECT_SUBDIR,
    skill_dir_name: SkillDirName::OriginalOnPersonal,
    agent_files: Some(FileSuffixStyle::KindSuffix),
    command_files: Some(FileSuffixStyle::KindSuffix),
    instruction_at: Some(InstructionPlacement::EnvRoot),
    hooks: HookLayout::NamedJson,
};

/// GitHub Copilot ターゲット
pub struct CopilotTarget;

impl CopilotTarget {
    pub fn new() -> Self {
        Self
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

    impl_target_layout!(LAYOUT);

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
}

#[cfg(test)]
#[path = "copilot_test.rs"]
mod tests;
