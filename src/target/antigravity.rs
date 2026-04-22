//! Google Antigravity ターゲット実装

use crate::component::{
    ComponentIdentity, ComponentKind, PlacementContext, PlacementLocation, Scope,
};
use crate::error::Result;
use crate::target::paths::home_dir;
use crate::target::scanner::{scan_components, ScannedComponent};
use crate::target::{Target, TargetKind};
use std::path::{Path, PathBuf};

const ANTIGRAVITY_PERSONAL_PARENT: &str = ".gemini";
const ANTIGRAVITY_PERSONAL_CHILD: &str = "antigravity";
const ANTIGRAVITY_PROJECT_SUBDIR: &str = ".agent";

/// Google Antigravity ターゲット
pub struct AntigravityTarget;

impl AntigravityTarget {
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
        match scope {
            Scope::Personal => home_dir()
                .join(ANTIGRAVITY_PERSONAL_PARENT)
                .join(ANTIGRAVITY_PERSONAL_CHILD),
            Scope::Project => project_root.join(ANTIGRAVITY_PROJECT_SUBDIR),
        }
    }

    /// この組み合わせで配置できるか（Skillのみサポート）
    ///
    /// # Arguments
    ///
    /// * `kind` - Component kind to check.
    fn can_place(kind: ComponentKind) -> bool {
        kind == ComponentKind::Skill
    }

    /// コンポーネント種別に応じたフィルタリング（SKILL.md存在チェック維持）
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
                    Some(ComponentIdentity::new(kind, c.name.as_str()).qualified_name(&c.origin))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Default for AntigravityTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl Target for AntigravityTarget {
    fn name(&self) -> &'static str {
        "antigravity"
    }

    fn display_name(&self) -> &'static str {
        "Google Antigravity"
    }

    fn kind(&self) -> TargetKind {
        TargetKind::Antigravity
    }

    fn supported_components(&self) -> &[ComponentKind] {
        &[ComponentKind::Skill]
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
#[path = "antigravity_test.rs"]
mod tests;
