//! OpenAI Codex ターゲット実装

use crate::component::{ComponentKind, PlacementContext, PlacementLocation, Scope};
use crate::error::Result;
use crate::target::paths::base_dir;
use crate::target::placed_common;
use crate::target::scanner::{scan_components, ScannedComponent};
use crate::target::{Target, TargetKind};
use std::path::{Path, PathBuf};

const CODEX_SUBDIR: &str = ".codex";

/// OpenAI Codex ターゲット
pub struct CodexTarget;

impl CodexTarget {
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
        base_dir(scope, project_root, CODEX_SUBDIR, CODEX_SUBDIR)
    }

    /// この組み合わせで配置できるか
    ///
    /// # Arguments
    ///
    /// * `kind` - Component kind to check.
    fn can_place(kind: ComponentKind) -> bool {
        kind != ComponentKind::Command && kind != ComponentKind::Hook
    }

    /// コンポーネント種別に応じたフィルタリング
    ///
    /// # Arguments
    ///
    /// * `c` - Scanned component entry.
    /// * `kind` - Component kind expected for the entry.
    fn filter_component(c: &ScannedComponent, kind: ComponentKind) -> Option<String> {
        let make_qualified = |name: &str| c.origin.qualify(name);
        match kind {
            ComponentKind::Skill if c.is_dir => Some(make_qualified(&c.name)),
            ComponentKind::Agent if !c.is_dir && c.name.ends_with(".agent.md") => {
                let name = c.name.trim_end_matches(".agent.md");
                Some(make_qualified(name))
            }
            _ => None,
        }
    }
}

impl Default for CodexTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl Target for CodexTarget {
    fn name(&self) -> &'static str {
        "codex"
    }

    fn display_name(&self) -> &'static str {
        "OpenAI Codex"
    }

    fn kind(&self) -> TargetKind {
        TargetKind::Codex
    }

    fn supported_components(&self) -> &[ComponentKind] {
        &[
            ComponentKind::Skill,
            ComponentKind::Agent,
            ComponentKind::Instruction,
        ]
    }

    fn placement_location(&self, context: &PlacementContext) -> Option<PlacementLocation> {
        let kind = context.kind();
        if !Self::can_place(kind) {
            return None;
        }

        let scope = context.scope();
        let project_root = context.project_root();
        let base = Self::base_dir(scope, project_root);
        let origin = context.origin;
        let name = context.name();

        let _ = origin;
        Some(match kind {
            // フラット構造: skills/<flattened_name> (ディレクトリ)
            ComponentKind::Skill => PlacementLocation::dir(base.join("skills").join(name)),
            // フラット構造: agents/<flattened_name>.agent.md (ファイル)
            ComponentKind::Agent => {
                PlacementLocation::file(base.join("agents").join(format!("{}.agent.md", name)))
            }
            ComponentKind::Instruction => match scope {
                // Project scope: AGENTS.md is at project root, not in .codex
                Scope::Project => PlacementLocation::file(project_root.join("AGENTS.md")),
                Scope::Personal => PlacementLocation::file(base.join("AGENTS.md")),
            },
            ComponentKind::Command | ComponentKind::Hook => return None,
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
                "AGENTS.md",
            ));
        }

        let base = Self::base_dir(scope, project_root);
        let dir_path = match kind {
            ComponentKind::Skill => base.join("skills"),
            ComponentKind::Agent => base.join("agents"),
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
#[path = "codex_test.rs"]
mod tests;
