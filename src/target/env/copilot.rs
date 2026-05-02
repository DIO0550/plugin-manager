//! GitHub Copilot ターゲット実装

use crate::component::{ComponentKind, PlacementContext, PlacementLocation, Scope};
use crate::error::Result;
use crate::target::paths::base_dir;
use crate::target::placed_common;
use crate::target::scanner::{scan_components, ScannedComponent};
use crate::target::{Target, TargetKind};
use std::path::{Path, PathBuf};

const COPILOT_PERSONAL_SUBDIR: &str = ".copilot";
const COPILOT_PROJECT_SUBDIR: &str = ".github";

/// GitHub Copilot ターゲット
pub struct CopilotTarget;

impl CopilotTarget {
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
        base_dir(
            scope,
            project_root,
            COPILOT_PERSONAL_SUBDIR,
            COPILOT_PROJECT_SUBDIR,
        )
    }

    /// この組み合わせで配置できるか
    ///
    /// # Arguments
    ///
    /// * `kind` - Component kind to check.
    /// * `scope` - Scope (`Personal` or `Project`) to check.
    fn can_place(kind: ComponentKind, scope: Scope) -> bool {
        matches!(
            (kind, scope),
            (ComponentKind::Agent, _) | (ComponentKind::Hook, _) | (_, Scope::Project)
        )
    }

    /// コンポーネント種別に応じたフィルタリング（Command対応含む）
    ///
    /// # Arguments
    ///
    /// * `c` - Scanned component entry.
    /// * `kind` - Component kind expected for the entry.
    fn filter_component(c: &ScannedComponent, kind: ComponentKind) -> Option<String> {
        match kind {
            // Skill: SKILL.md が直下にあるディレクトリのみ採用（二重防御）。
            ComponentKind::Skill if c.is_dir && c.path.join("SKILL.md").is_file() => {
                Some(c.name.clone())
            }
            ComponentKind::Agent if !c.is_dir && c.name.ends_with(".agent.md") => {
                Some(c.name.trim_end_matches(".agent.md").to_string())
            }
            ComponentKind::Command if !c.is_dir && c.name.ends_with(".prompt.md") => {
                Some(c.name.trim_end_matches(".prompt.md").to_string())
            }
            ComponentKind::Hook if !c.is_dir && c.name.ends_with(".json") => {
                Some(c.name.trim_end_matches(".json").to_string())
            }
            _ => None,
        }
    }
}

impl Default for CopilotTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl Target for CopilotTarget {
    fn name(&self) -> &'static str {
        "copilot"
    }

    fn display_name(&self) -> &'static str {
        "GitHub Copilot"
    }

    fn kind(&self) -> TargetKind {
        TargetKind::Copilot
    }

    fn supported_components(&self) -> &[ComponentKind] {
        &[
            ComponentKind::Skill,
            ComponentKind::Agent,
            ComponentKind::Command,
            ComponentKind::Instruction,
            ComponentKind::Hook,
        ]
    }

    fn placement_location(&self, context: &PlacementContext) -> Option<PlacementLocation> {
        let kind = context.kind();
        let scope = context.scope();
        if !Self::can_place(kind, scope) {
            return None;
        }

        let project_root = context.project_root();
        let base = Self::base_dir(scope, project_root);
        let name = context.name();

        Some(match kind {
            // フラット構造: skills/<flattened_name> (ディレクトリ)
            ComponentKind::Skill => PlacementLocation::dir(base.join("skills").join(name)),
            // フラット構造: agents/<flattened_name>.agent.md (ファイル)
            ComponentKind::Agent => {
                PlacementLocation::file(base.join("agents").join(format!("{}.agent.md", name)))
            }
            // フラット構造: prompts/<flattened_name>.prompt.md (ファイル)
            ComponentKind::Command => {
                PlacementLocation::file(base.join("prompts").join(format!("{}.prompt.md", name)))
            }
            ComponentKind::Instruction => {
                PlacementLocation::file(base.join("copilot-instructions.md"))
            }
            // フラット構造: hooks/<flattened_name>.json (ファイル)
            ComponentKind::Hook => {
                PlacementLocation::file(base.join("hooks").join(format!("{}.json", name)))
            }
        })
    }

    fn list_placed(
        &self,
        kind: ComponentKind,
        scope: Scope,
        project_root: &Path,
    ) -> Result<Vec<String>> {
        if !Self::can_place(kind, scope) {
            return Ok(vec![]);
        }

        if kind == ComponentKind::Instruction {
            return Ok(placed_common::list_instruction(
                self,
                scope,
                project_root,
                "copilot-instructions.md",
            ));
        }

        let base = Self::base_dir(scope, project_root);
        let dir_path = match kind {
            ComponentKind::Skill => base.join("skills"),
            ComponentKind::Agent => base.join("agents"),
            ComponentKind::Command => base.join("prompts"),
            ComponentKind::Hook => base.join("hooks"),
            _ => return Ok(vec![]),
        };

        let names = scan_components(&dir_path)?
            .into_iter()
            .filter_map(|c| Self::filter_component(&c, kind))
            .collect();

        Ok(names)
    }
}

#[cfg(test)]
#[path = "copilot_test.rs"]
mod tests;
