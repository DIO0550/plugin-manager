//! ターゲットの宣言的レイアウト / ケイパビリティ記述子（#338）
//!
//! 各 env は [`TargetLayout`] に差分データだけを書き、配置・列挙・サポート判定は
//! 本モジュールの共通実装が担う。`supported_components` と `can_place_scope` は
//! 同一の [`crate::target::scope_support::Capabilities`] から導出する。

use crate::component::{ComponentKind, PlacementContext, PlacementLocation, Scope};
use crate::error::Result;
use crate::placement_names::OPENCODE_PERSONAL_CHILD;
use crate::target::filter::{
    filter_exact_file, filter_json_suffix, filter_plain_markdown, filter_skill_dir,
    filter_suffix_file,
};
use crate::target::list_helpers::{list_instruction_at, scan_and_filter, scan_and_filter_in};
use crate::target::paths::{home_dir, xdg_config_child};
use crate::target::placement_helpers::{
    instruction_file, instruction_under_base, named_file, skill_dir,
};
use crate::target::scope_support::Capabilities;
use crate::target::TargetKind;
use std::path::{Path, PathBuf};

/// Personal スコープの環境ルートの決め方。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PersonalRoot {
    /// `home_dir()/<subdir>`（`.codex` 等）
    HomeSubdir(&'static str),
    /// `home_dir()/<parent>/<child>`（Antigravity `~/.gemini/config`）
    HomeNested {
        parent: &'static str,
        child: &'static str,
    },
    /// `$XDG_CONFIG_HOME/opencode`（未設定時 `~/.config/opencode`）
    XdgConfig,
}

/// Skill ディレクトリ名の決め方。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SkillDirName {
    /// フラット化名（`context.name()`）
    Flattened,
    /// frontmatter 元名。未設定・空なら配置不可
    Original,
    /// Personal は元名、Project はフラット化名
    OriginalOnPersonal,
}

/// Agent / Command のファイルサフィックス。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileSuffixStyle {
    /// `ComponentKind::file_suffix()`（`.agent.md` / `.prompt.md`）
    KindSuffix,
    /// プレーン `.md`（Cursor / OpenCode）
    PlainMarkdown,
}

/// Instruction ファイルの配置位置。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InstructionPlacement {
    /// Project: `project_root/<file>`、Personal: `env_root/<file>`
    ProjectRootOrEnvRoot,
    /// 常に `env_root/<file>`（Copilot）
    EnvRoot,
}

/// Hook の配置形状。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HookLayout {
    Unsupported,
    /// 環境ルート直下の単一ファイル（`hooks.json`）
    SingleFile {
        filename: &'static str,
    },
    /// `hooks/<name>.json`（Copilot）
    NamedJson,
}

/// ターゲットごとの配置差分（データ）とケイパビリティ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TargetLayout {
    pub kind: TargetKind,
    pub capabilities: Capabilities,
    pub personal_root: PersonalRoot,
    pub project_subdir: &'static str,
    pub skill_dir_name: SkillDirName,
    pub agent_files: Option<FileSuffixStyle>,
    pub command_files: Option<FileSuffixStyle>,
    pub instruction_at: Option<InstructionPlacement>,
    pub hooks: HookLayout,
}

impl PersonalRoot {
    fn resolve(self) -> PathBuf {
        match self {
            Self::HomeSubdir(subdir) => home_dir().join(subdir),
            Self::HomeNested { parent, child } => home_dir().join(parent).join(child),
            Self::XdgConfig => xdg_config_child(&home_dir(), OPENCODE_PERSONAL_CHILD),
        }
    }
}

impl SkillDirName {
    fn resolve<'a>(self, context: &'a PlacementContext<'_>) -> Option<&'a str> {
        match self {
            Self::Flattened => Some(context.name()),
            Self::Original => nonempty_original(context),
            Self::OriginalOnPersonal => match context.scope() {
                Scope::Personal => nonempty_original(context),
                Scope::Project => Some(context.name()),
            },
        }
    }
}

fn nonempty_original<'a>(context: &'a PlacementContext<'_>) -> Option<&'a str> {
    context.original_name().filter(|name| !name.is_empty())
}

impl FileSuffixStyle {
    fn suffix(self, kind: ComponentKind) -> &'static str {
        match self {
            Self::PlainMarkdown => ".md",
            Self::KindSuffix => kind
                .file_suffix()
                .expect("KindSuffix is only valid for Agent/Command"),
        }
    }
}

impl TargetLayout {
    pub(crate) fn supported_components(&self) -> &'static [ComponentKind] {
        self.capabilities.supported()
    }

    pub(crate) fn allows(&self, kind: ComponentKind, scope: Scope) -> bool {
        self.capabilities.allows(kind, scope)
    }

    pub(crate) fn env_root(&self, scope: Scope, project_root: &Path) -> PathBuf {
        match scope {
            Scope::Personal => self.personal_root.resolve(),
            Scope::Project => project_root.join(self.project_subdir),
        }
    }

    pub(crate) fn placement_location(
        &self,
        context: &PlacementContext,
    ) -> Option<PlacementLocation> {
        let kind = context.kind();
        let scope = context.scope();
        if !self.allows(kind, scope) {
            return None;
        }
        let base = self.env_root(scope, context.project_root());
        match kind {
            ComponentKind::Skill => self.skill_location(&base, context),
            ComponentKind::Agent => {
                self.named_file_location(&base, kind, self.agent_files, context.name())
            }
            ComponentKind::Command => {
                self.named_file_location(&base, kind, self.command_files, context.name())
            }
            ComponentKind::Instruction => {
                self.instruction_location(scope, context.project_root(), &base)
            }
            ComponentKind::Hook => self.hook_location(&base, context.name()),
        }
    }

    pub(crate) fn list_placed(
        &self,
        kind: ComponentKind,
        scope: Scope,
        project_root: &Path,
    ) -> Result<Vec<String>> {
        if !self.allows(kind, scope) {
            return Ok(vec![]);
        }
        let base = self.env_root(scope, project_root);
        match kind {
            ComponentKind::Instruction => Ok(self.list_instruction(scope, project_root, &base)),
            ComponentKind::Skill => self.list_skills(&base),
            ComponentKind::Agent => self.list_named_files(&base, kind, self.agent_files),
            ComponentKind::Command => self.list_named_files(&base, kind, self.command_files),
            ComponentKind::Hook => self.list_hooks(&base),
        }
    }

    /// ケイパビリティ表とパス形状フィールドが食い違っていないこと。
    pub(crate) fn assert_consistent(&self) {
        let derived = self.capabilities.derived_supported();
        assert_eq!(
            derived.as_slice(),
            self.capabilities.supported(),
            "{}: capabilities table must match supported slice",
            self.kind.as_str()
        );
        let supports = |kind| derived.contains(&kind);
        assert_eq!(supports(ComponentKind::Agent), self.agent_files.is_some());
        assert_eq!(
            supports(ComponentKind::Command),
            self.command_files.is_some()
        );
        assert_eq!(
            supports(ComponentKind::Instruction),
            self.instruction_at.is_some()
        );
        if supports(ComponentKind::Instruction) {
            assert!(self.kind.instruction_filename().is_some());
        }
        assert_eq!(
            supports(ComponentKind::Hook),
            !matches!(self.hooks, HookLayout::Unsupported)
        );
    }

    fn skill_location(&self, base: &Path, context: &PlacementContext) -> Option<PlacementLocation> {
        let name = self.skill_dir_name.resolve(context)?;
        Some(skill_dir(base, name))
    }

    fn named_file_location(
        &self,
        base: &Path,
        kind: ComponentKind,
        style: Option<FileSuffixStyle>,
        name: &str,
    ) -> Option<PlacementLocation> {
        let style = style?;
        let subdir = self.kind.placement_subdir(kind)?;
        Some(named_file(base, subdir, name, style.suffix(kind)))
    }

    fn instruction_location(
        &self,
        scope: Scope,
        project_root: &Path,
        base: &Path,
    ) -> Option<PlacementLocation> {
        let filename = self.kind.instruction_filename()?;
        match self.instruction_at? {
            InstructionPlacement::ProjectRootOrEnvRoot => {
                Some(instruction_file(scope, project_root, base, filename))
            }
            InstructionPlacement::EnvRoot => Some(instruction_under_base(base, filename)),
        }
    }

    fn hook_location(&self, base: &Path, name: &str) -> Option<PlacementLocation> {
        match self.hooks {
            HookLayout::Unsupported => None,
            HookLayout::SingleFile { filename } => {
                Some(PlacementLocation::file(base.join(filename)))
            }
            HookLayout::NamedJson => {
                let subdir = self.kind.placement_subdir(ComponentKind::Hook)?;
                Some(named_file(base, subdir, name, ".json"))
            }
        }
    }

    fn list_instruction(&self, scope: Scope, project_root: &Path, base: &Path) -> Vec<String> {
        let Some(filename) = self.kind.instruction_filename() else {
            return vec![];
        };
        let Some(location) = self.instruction_location(scope, project_root, base) else {
            return vec![];
        };
        list_instruction_at(location.as_path(), filename)
    }

    fn list_skills(&self, base: &Path) -> Result<Vec<String>> {
        let Some(subdir) = self.kind.placement_subdir(ComponentKind::Skill) else {
            return Ok(vec![]);
        };
        scan_and_filter(base, subdir, filter_skill_dir)
    }

    fn list_named_files(
        &self,
        base: &Path,
        kind: ComponentKind,
        style: Option<FileSuffixStyle>,
    ) -> Result<Vec<String>> {
        let (Some(style), Some(subdir)) = (style, self.kind.placement_subdir(kind)) else {
            return Ok(vec![]);
        };
        match style {
            FileSuffixStyle::KindSuffix => {
                let suffix = style.suffix(kind);
                scan_and_filter(base, subdir, |c| filter_suffix_file(c, suffix))
            }
            FileSuffixStyle::PlainMarkdown => scan_and_filter(base, subdir, filter_plain_markdown),
        }
    }

    fn list_hooks(&self, base: &Path) -> Result<Vec<String>> {
        match self.hooks {
            HookLayout::Unsupported => Ok(vec![]),
            HookLayout::SingleFile { filename } => scan_and_filter_in(base, |c| {
                filter_exact_file(c, filename, ComponentKind::Hook.plural())
            }),
            HookLayout::NamedJson => {
                let Some(subdir) = self.kind.placement_subdir(ComponentKind::Hook) else {
                    return Ok(vec![]);
                };
                scan_and_filter(base, subdir, filter_json_suffix)
            }
        }
    }
}

/// `Target` impl のサポート判定・配置・列挙を `LAYOUT` に委譲する。
macro_rules! impl_target_layout {
    ($layout:expr) => {
        fn supported_components(&self) -> &[crate::component::ComponentKind] {
            $layout.supported_components()
        }

        fn can_place_scope(
            &self,
            kind: crate::component::ComponentKind,
            scope: crate::component::Scope,
        ) -> bool {
            $layout.allows(kind, scope)
        }

        fn placement_location(
            &self,
            context: &crate::component::PlacementContext,
        ) -> Option<crate::component::PlacementLocation> {
            $layout.placement_location(context)
        }

        fn list_placed(
            &self,
            kind: crate::component::ComponentKind,
            scope: crate::component::Scope,
            project_root: &std::path::Path,
        ) -> crate::error::Result<Vec<String>> {
            $layout.list_placed(kind, scope, project_root)
        }
    };
}
pub(crate) use impl_target_layout;

#[cfg(test)]
#[path = "descriptor_test.rs"]
mod tests;
