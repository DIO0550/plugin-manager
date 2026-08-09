//! TargetKind 向け配置パス API（#339）
//!
//! 文字列定数の正本は [`crate::placement_names`]。本モジュールは TargetKind への薄い API を提供する。

use crate::component::{ComponentKind, Scope};
use crate::placement_names::{
    self, ALL_INSTRUCTION_FILENAMES, ANTIGRAVITY_PROJECT_SUBDIR, COPILOT_COMMAND_SUBDIR,
    INSTRUCTION_AGENTS, INSTRUCTION_COPILOT, INSTRUCTION_GEMINI,
};
use crate::target::TargetKind;
use std::path::{Path, PathBuf};

impl TargetKind {
    /// Instruction ファイル名。非サポートターゲットは `None`。
    pub fn instruction_filename(self) -> Option<&'static str> {
        match self {
            TargetKind::Codex | TargetKind::Cursor | TargetKind::OpenCode => {
                Some(INSTRUCTION_AGENTS)
            }
            TargetKind::Copilot => Some(INSTRUCTION_COPILOT),
            TargetKind::GeminiCli => Some(INSTRUCTION_GEMINI),
            TargetKind::Antigravity => None,
        }
    }

    /// ターゲット上の配置サブディレクトリ。
    ///
    /// デフォルトは [`ComponentKind::default_subdir`]。Copilot Command のみ `"prompts"`。
    pub fn placement_subdir(self, kind: ComponentKind) -> Option<&'static str> {
        match (self, kind) {
            (TargetKind::Copilot, ComponentKind::Command) => Some(COPILOT_COMMAND_SUBDIR),
            (_, kind) => kind.default_subdir(),
        }
    }

    /// Personal スコープの環境ルートを `home` 配下に解決する。
    pub fn personal_base(self, home: &Path) -> PathBuf {
        match self {
            TargetKind::Codex => home.join(placement_names::CODEX_SUBDIR),
            TargetKind::Copilot => home.join(placement_names::COPILOT_PERSONAL_SUBDIR),
            TargetKind::Antigravity => home
                .join(placement_names::ANTIGRAVITY_PERSONAL_PARENT)
                .join(placement_names::ANTIGRAVITY_PERSONAL_CHILD),
            TargetKind::GeminiCli => home.join(placement_names::GEMINI_SUBDIR),
            TargetKind::Cursor => home.join(placement_names::CURSOR_SUBDIR),
            TargetKind::OpenCode => home
                .join(placement_names::OPENCODE_PERSONAL_PARENT)
                .join(placement_names::OPENCODE_PERSONAL_CHILD),
        }
    }

    /// Project スコープの環境ルートを `project_root` 配下に解決する。
    pub fn project_base(self, project_root: &Path) -> PathBuf {
        match self {
            TargetKind::Codex => project_root.join(placement_names::CODEX_SUBDIR),
            TargetKind::Copilot => project_root.join(placement_names::COPILOT_PROJECT_SUBDIR),
            TargetKind::Antigravity => project_root.join(ANTIGRAVITY_PROJECT_SUBDIR),
            TargetKind::GeminiCli => project_root.join(placement_names::GEMINI_SUBDIR),
            TargetKind::Cursor => project_root.join(placement_names::CURSOR_SUBDIR),
            TargetKind::OpenCode => project_root.join(placement_names::OPENCODE_PROJECT_SUBDIR),
        }
    }

    /// アンインストール後クリーンアップ用の (base, kind_subdir) 一覧。
    pub fn cleanup_specs(
        self,
        home: Option<&Path>,
        project_root: &Path,
    ) -> Vec<(PathBuf, &'static str)> {
        let mut specs = Vec::new();

        if let Some(h) = home {
            let base = self.personal_base(h);
            for sub in self.cleanup_kind_subdirs(Scope::Personal) {
                specs.push((base.clone(), sub));
            }
        }

        let base = self.project_base(project_root);
        for sub in self.cleanup_kind_subdirs(Scope::Project) {
            specs.push((base.clone(), sub));
        }

        specs
    }

    fn cleanup_kind_subdirs(self, scope: Scope) -> Vec<&'static str> {
        let kinds: &[ComponentKind] = match (self, scope) {
            (TargetKind::Codex, _) => &[ComponentKind::Agent, ComponentKind::Skill],
            (TargetKind::Copilot, Scope::Personal) => &[ComponentKind::Agent, ComponentKind::Hook],
            (TargetKind::Copilot, Scope::Project) => &[
                ComponentKind::Agent,
                ComponentKind::Command,
                ComponentKind::Skill,
                ComponentKind::Hook,
            ],
            (TargetKind::Antigravity, _) | (TargetKind::GeminiCli, _) => &[ComponentKind::Skill],
            // OpenCode は Cursor と同型（Hooks 非対応）
            (TargetKind::Cursor, _) | (TargetKind::OpenCode, _) => &[
                ComponentKind::Skill,
                ComponentKind::Agent,
                ComponentKind::Command,
            ],
        };
        kinds
            .iter()
            .filter_map(|&kind| self.placement_subdir(kind))
            .collect()
    }
}

/// `instruction_filename` 集合が `ALL_INSTRUCTION_FILENAMES` と一致することを検証する。
pub fn assert_instruction_filenames_consistent() {
    let mut from_targets: Vec<&'static str> = [
        TargetKind::Antigravity,
        TargetKind::Codex,
        TargetKind::Copilot,
        TargetKind::Cursor,
        TargetKind::GeminiCli,
        TargetKind::OpenCode,
    ]
    .into_iter()
    .filter_map(TargetKind::instruction_filename)
    .collect();
    from_targets.sort_unstable();
    from_targets.dedup();

    let mut expected: Vec<&'static str> = ALL_INSTRUCTION_FILENAMES.to_vec();
    expected.sort_unstable();
    assert_eq!(
        from_targets, expected,
        "TargetKind::instruction_filename set must match ALL_INSTRUCTION_FILENAMES"
    );
}

#[cfg(test)]
#[path = "layout_test.rs"]
mod tests;
