//! OpenAI Codex ターゲット実装

use crate::component::{ComponentKind, Scope};
use crate::component::{
    ComponentRef, PlacementContext, PlacementLocation, PlacementScope, ProjectContext,
};
use crate::error::Result;
use crate::target::scanner::{scan_components, ScannedComponent};
use crate::target::{PluginOrigin, Target, TargetKind};
use std::path::{Path, PathBuf};

/// OpenAI Codex ターゲット
pub struct CodexTarget;

impl CodexTarget {
    pub fn new() -> Self {
        Self
    }

    fn home_dir() -> PathBuf {
        std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("~"))
    }

    /// スコープに応じたベースディレクトリを取得
    fn base_dir(scope: Scope, project_root: &Path) -> PathBuf {
        match scope {
            Scope::Personal => Self::home_dir().join(".codex"),
            Scope::Project => project_root.join(".codex"),
        }
    }

    /// この組み合わせで配置できるか
    fn can_place(kind: ComponentKind) -> bool {
        kind != ComponentKind::Command && kind != ComponentKind::Hook
    }

    /// コンポーネント種別に応じたフィルタリング
    fn filter_component(c: &ScannedComponent, kind: ComponentKind) -> Option<String> {
        let qualified = format!("{}/{}/{}", c.origin.marketplace, c.origin.plugin, c.name);
        match kind {
            ComponentKind::Skill if c.is_dir => Some(qualified),
            ComponentKind::Agent if !c.is_dir && c.name.ends_with(".agent.md") => {
                let name = c.name.trim_end_matches(".agent.md");
                Some(format!(
                    "{}/{}/{}",
                    c.origin.marketplace, c.origin.plugin, name
                ))
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

        Some(match kind {
            // 階層構造: skills/<marketplace>/<plugin>/<skill> (ディレクトリ)
            ComponentKind::Skill => PlacementLocation::dir(
                base.join("skills")
                    .join(&origin.marketplace)
                    .join(&origin.plugin)
                    .join(name),
            ),
            // 階層構造: agents/<marketplace>/<plugin>/<name>.agent.md (ファイル)
            ComponentKind::Agent => PlacementLocation::file(
                base.join("agents")
                    .join(&origin.marketplace)
                    .join(&origin.plugin)
                    .join(format!("{}.agent.md", name)),
            ),
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

        // Instruction は単一ファイル
        if kind == ComponentKind::Instruction {
            let dummy_origin = PluginOrigin::from_marketplace("", "");
            let ctx = PlacementContext {
                component: ComponentRef::new(kind, ""),
                origin: &dummy_origin,
                scope: PlacementScope(scope),
                project: ProjectContext::new(project_root),
            };
            let location = self.placement_location(&ctx).unwrap();
            return if location.as_path().exists() {
                Ok(vec!["AGENTS.md".to_string()])
            } else {
                Ok(vec![])
            };
        }

        let base = Self::base_dir(scope, project_root);
        let dir_path = match kind {
            ComponentKind::Skill => base.join("skills"),
            ComponentKind::Agent => base.join("agents"),
            _ => return Ok(vec![]),
        };

        // scan_components を使用して3層構造を走査
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
