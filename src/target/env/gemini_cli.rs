//! Gemini CLI ターゲット実装

use crate::placement_names::GEMINI_SUBDIR;
use crate::target::scope_support::capabilities;
use crate::target::{
    impl_target_layout, HookLayout, InstructionPlacement, PersonalRoot, SkillDirName, Target,
    TargetKind, TargetLayout,
};

pub(crate) const LAYOUT: TargetLayout = TargetLayout {
    kind: TargetKind::GeminiCli,
    capabilities: capabilities!(
        Skill => Both,
        Instruction => Both,
    ),
    personal_root: PersonalRoot::HomeSubdir(GEMINI_SUBDIR),
    project_subdir: GEMINI_SUBDIR,
    skill_dir_name: SkillDirName::Flattened,
    agent_files: None,
    command_files: None,
    instruction_at: Some(InstructionPlacement::ProjectRootOrEnvRoot),
    hooks: HookLayout::Unsupported,
};

/// Gemini CLI ターゲット
pub struct GeminiCliTarget;

impl GeminiCliTarget {
    pub fn new() -> Self {
        Self
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

    impl_target_layout!(LAYOUT);
}

#[cfg(test)]
#[path = "gemini_cli_test.rs"]
mod tests;
