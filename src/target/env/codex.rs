//! OpenAI Codex ターゲット実装

mod feature_flag;

pub use feature_flag::{apply_codex_hooks_flag, FeatureFlagOutcome};

use crate::component::{Component, ComponentKind, PlacementContext, PlacementLocation, Scope};
use crate::error::Result;
use crate::target::filter::{filter_exact_file, filter_skill_dir, filter_suffix_file};
use crate::target::list_helpers::{list_instruction_at, scan_and_filter, scan_and_filter_in};
use crate::target::paths::base_dir;
use crate::target::placement_helpers::{agent_file, instruction_file, skill_dir};
use crate::target::scope_support::{allows_scope, ScopeSupport};
use crate::target::{PostPlaceOutcome, Target, TargetKind};
use std::path::{Path, PathBuf};

struct CodexLayout {
    subdir: &'static str,
    config_file: &'static str,
    instruction_file: &'static str,
    hooks_file: &'static str,
}

const LAYOUT: CodexLayout = CodexLayout {
    subdir: ".codex",
    config_file: "config.toml",
    instruction_file: "AGENTS.md",
    hooks_file: "hooks.json",
};

const SUPPORTED: &[ComponentKind] = &[
    ComponentKind::Skill,
    ComponentKind::Agent,
    ComponentKind::Instruction,
    ComponentKind::Hook,
];

const CAPABILITIES: &[(ComponentKind, ScopeSupport)] = &[
    (ComponentKind::Skill, ScopeSupport::Both),
    (ComponentKind::Agent, ScopeSupport::Both),
    (ComponentKind::Instruction, ScopeSupport::Both),
    (ComponentKind::Hook, ScopeSupport::Both),
];

/// OpenAI Codex ターゲット
pub struct CodexTarget;

impl CodexTarget {
    pub fn new() -> Self {
        Self
    }

    fn base_dir(scope: Scope, project_root: &Path) -> PathBuf {
        base_dir(scope, project_root, LAYOUT.subdir, LAYOUT.subdir)
    }

    /// スコープに応じた `config.toml` のフルパスを返す。
    pub(crate) fn config_toml_path(scope: Scope, project_root: &Path) -> PathBuf {
        Self::base_dir(scope, project_root).join(LAYOUT.config_file)
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
            ComponentKind::Instruction => {
                instruction_file(scope, project_root, &base, LAYOUT.instruction_file)
            }
            ComponentKind::Hook => PlacementLocation::file(base.join(LAYOUT.hooks_file)),
            ComponentKind::Command => return None,
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
            ComponentKind::Agent => {
                scan_and_filter(&base, "agents", |c| filter_suffix_file(c, ".agent.md"))
            }
            ComponentKind::Hook => {
                scan_and_filter_in(&base, |c| filter_exact_file(c, LAYOUT.hooks_file, "hooks"))
            }
            _ => Ok(vec![]),
        }
    }
}

#[cfg(test)]
#[path = "codex_test.rs"]
mod tests;
