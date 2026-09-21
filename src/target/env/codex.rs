//! OpenAI Codex ターゲット実装

mod feature_flag;

pub use feature_flag::{apply_codex_hooks_flag, FeatureFlagOutcome};

use crate::component::{Component, ComponentKind, PlacementContext, Scope};
use crate::placement_names::CODEX_SUBDIR;
use crate::target::scope_support::capabilities;
use crate::target::{
    impl_target_layout, FileSuffixStyle, HookLayout, InstructionPlacement, PersonalRoot,
    PostPlaceOutcome, SkillDirName, Target, TargetKind, TargetLayout,
};
use std::path::{Path, PathBuf};

pub(crate) const LAYOUT: TargetLayout = TargetLayout {
    kind: TargetKind::Codex,
    capabilities: capabilities!(
        Skill => Both,
        Agent => Both,
        Instruction => Both,
        Hook => Both,
    ),
    personal_root: PersonalRoot::HomeSubdir(CODEX_SUBDIR),
    project_subdir: CODEX_SUBDIR,
    skill_dir_name: SkillDirName::Flattened,
    agent_files: Some(FileSuffixStyle::KindSuffix),
    command_files: None,
    instruction_at: Some(InstructionPlacement::ProjectRootOrEnvRoot),
    hooks: HookLayout::SingleFile {
        filename: "hooks.json",
    },
};

const CONFIG_FILE: &str = "config.toml";

/// OpenAI Codex ターゲット
pub struct CodexTarget;

impl CodexTarget {
    pub fn new() -> Self {
        Self
    }

    /// スコープに応じた `config.toml` のフルパスを返す。
    pub(crate) fn config_toml_path(scope: Scope, project_root: &Path) -> PathBuf {
        LAYOUT.env_root(scope, project_root).join(CONFIG_FILE)
    }

    /// Codex は 1 スコープにつき単一の `hooks.json` を読むため、複数 Hook を
    /// 個別配置すると同じファイルを上書きする。マージ未実装の間は拒否する。
    pub fn hook_component_conflict_error(components: &[Component]) -> Option<String> {
        let hook_count = components
            .iter()
            .filter(|component| component.kind == ComponentKind::Hook)
            .count();

        (hook_count > 1).then(|| {
            format!(
                "Codex target supports a single hooks.json per scope; {} Hook components would overwrite each other. Select one Hook component or wait for merge support.",
                hook_count
            )
        })
    }

    /// 配置先 `hooks.json` がすでに存在し、管理下に無い場合にエラーを返す。
    pub fn hook_overwrite_error(target_path: &Path, plugin_root: &Path) -> Option<String> {
        if !target_path.exists() {
            return None;
        }

        let already_owned = crate::plugin::meta::load_meta(plugin_root)
            .map(|meta| meta.manages_file("codex", target_path))
            .unwrap_or(false);

        if already_owned {
            return None;
        }

        Some(format!(
            "{} already exists and is not managed by this plugin. \
             Refusing to overwrite; remove the file or merge it manually before re-installing.",
            target_path.display()
        ))
    }
}

impl Default for CodexTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl Target for CodexTarget {
    fn display_name(&self) -> &'static str {
        "OpenAI Codex"
    }

    fn kind(&self) -> TargetKind {
        TargetKind::Codex
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
        if context.kind() == ComponentKind::Hook {
            if let Some(error) = Self::hook_overwrite_error(target_path, plugin_root) {
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
        enable_feature_flag: bool,
    ) -> PostPlaceOutcome {
        let mut outcome = PostPlaceOutcome::default();

        if context.kind() != ComponentKind::Hook {
            return outcome;
        }

        crate::install::record_codex_hook_ownership(plugin_root, deployed_path);

        if enable_feature_flag {
            let config_path = Self::config_toml_path(context.scope(), context.project_root());
            match apply_codex_hooks_flag(&config_path) {
                Ok(ffo) => outcome.feature_flags.push(ffo),
                Err(e) => {
                    eprintln!(
                        "Warning: failed to enable [features] codex_hooks in {}: {}",
                        config_path.display(),
                        e
                    );
                }
            }
            outcome.feature_flag_attempted = true;
        }

        outcome
    }
}

#[cfg(test)]
#[path = "codex_test.rs"]
mod tests;
