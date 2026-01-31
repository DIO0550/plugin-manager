//! Gemini CLI ターゲット実装

use crate::component::{ComponentKind, Scope};
use crate::component::{
    ComponentRef, PlacementContext, PlacementLocation, PlacementScope, ProjectContext,
};
use crate::error::Result;
use crate::target::scanner::{scan_components, ScannedComponent};
use crate::target::{PluginOrigin, Target, TargetKind};
use std::path::{Path, PathBuf};

/// Gemini CLI ターゲット
pub struct GeminiCliTarget;

impl GeminiCliTarget {
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
            Scope::Personal => Self::home_dir().join(".gemini"),
            Scope::Project => project_root.join(".gemini"),
        }
    }

    /// この組み合わせで配置できるか（Skill + Instruction のみサポート）
    fn can_place(kind: ComponentKind) -> bool {
        kind == ComponentKind::Skill || kind == ComponentKind::Instruction
    }

    /// コンポーネント種別に応じたフィルタリング（SKILL.md 存在チェック）
    fn filter_component(c: &ScannedComponent, kind: ComponentKind) -> Option<String> {
        match kind {
            ComponentKind::Skill if c.is_dir => {
                let skill_md = c.path.join("SKILL.md");
                if skill_md.exists() {
                    Some(format!(
                        "{}/{}/{}",
                        c.origin.marketplace, c.origin.plugin, c.name
                    ))
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
        let origin = context.origin;
        let name = context.name();

        Some(match kind {
            ComponentKind::Skill => PlacementLocation::dir(
                base.join("skills")
                    .join(&origin.marketplace)
                    .join(&origin.plugin)
                    .join(name),
            ),
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
            let dummy_origin = PluginOrigin::from_marketplace("", "");
            let ctx = PlacementContext {
                component: ComponentRef::new(kind, ""),
                origin: &dummy_origin,
                scope: PlacementScope(scope),
                project: ProjectContext::new(project_root),
            };
            let location = self.placement_location(&ctx).unwrap();
            return if location.as_path().exists() {
                Ok(vec!["GEMINI.md".to_string()])
            } else {
                Ok(vec![])
            };
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
