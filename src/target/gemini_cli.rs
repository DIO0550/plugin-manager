//! Gemini CLI ターゲット実装

use crate::component::{ComponentKind, PlacementContext, PlacementLocation, Scope};
use crate::error::Result;
use crate::target::paths::base_dir;
use crate::target::placed_common;
use crate::target::scanner::{scan_components, ScannedComponent};
use crate::target::{Target, TargetKind};
use std::path::{Path, PathBuf};

const GEMINI_SUBDIR: &str = ".gemini";

/// Gemini CLI ターゲット
pub struct GeminiCliTarget;

impl GeminiCliTarget {
    pub fn new() -> Self {
        Self
    }

    /// スコープに応じたベースディレクトリを取得
    ///
    /// # Arguments
    ///
    /// * `scope` - Scope (`Personal` or `Project`) that selects the base directory.
    /// * `project_root` - Project root directory used for project scope.
    fn base_dir(scope: Scope, project_root: &Path) -> PathBuf {
        base_dir(scope, project_root, GEMINI_SUBDIR, GEMINI_SUBDIR)
    }

    /// この組み合わせで配置できるか（Skill + Instruction のみサポート）
    ///
    /// # Arguments
    ///
    /// * `kind` - Component kind to check.
    fn can_place(kind: ComponentKind) -> bool {
        kind == ComponentKind::Skill || kind == ComponentKind::Instruction
    }

    /// コンポーネント種別に応じたフィルタリング（SKILL.md 存在チェック）
    ///
    /// # Arguments
    ///
    /// * `c` - Scanned component entry.
    /// * `kind` - Component kind expected for the entry.
    fn filter_component(c: &ScannedComponent, kind: ComponentKind) -> Option<String> {
        match kind {
            ComponentKind::Skill if c.is_dir => {
                let skill_md = c.path.join("SKILL.md");
                if skill_md.exists() {
                    Some(c.name.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Default for GeminiCliTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl Target for GeminiCliTarget {
    fn name(&self) -> &'static str {
        "gemini"
    }

    fn display_name(&self) -> &'static str {
        "Gemini CLI"
    }

    fn kind(&self) -> TargetKind {
        TargetKind::GeminiCli
    }

    fn supported_components(&self) -> &[ComponentKind] {
        &[ComponentKind::Skill, ComponentKind::Instruction]
    }

    fn placement_location(&self, context: &PlacementContext) -> Option<PlacementLocation> {
        let kind = context.kind();
        if !Self::can_place(kind) {
            return None;
        }

        let scope = context.scope();
        let project_root = context.project_root();
        let base = Self::base_dir(scope, project_root);
        let name = context.name();

        Some(match kind {
            ComponentKind::Skill => PlacementLocation::dir(base.join("skills").join(name)),
            ComponentKind::Instruction => match scope {
                Scope::Project => PlacementLocation::file(project_root.join("GEMINI.md")),
                Scope::Personal => PlacementLocation::file(base.join("GEMINI.md")),
            },
            _ => return None,
        })
    }

    fn list_placed(
        &self,
        kind: ComponentKind,
        scope: Scope,
        project_root: &Path,
    ) -> Result<Vec<String>> {
        if !Self::can_place(kind) {
            return Ok(vec![]);
        }

        if kind == ComponentKind::Instruction {
            return Ok(placed_common::list_instruction(
                self,
                scope,
                project_root,
                "GEMINI.md",
            ));
        }

        let base = Self::base_dir(scope, project_root);
        let dir_path = base.join("skills");

        let names = scan_components(&dir_path)?
            .into_iter()
            .filter_map(|c| Self::filter_component(&c, kind))
            .collect();

        Ok(names)
    }
}

#[cfg(test)]
#[path = "gemini_cli_test.rs"]
mod tests;
