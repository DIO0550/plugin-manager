//! Google Antigravity ターゲット実装

use crate::component::{ComponentKind, PlacementContext, PlacementLocation, Scope};
use crate::error::Result;
use crate::target::scanner::{scan_components, ScannedComponent};
use crate::target::Target;
use std::path::{Path, PathBuf};

/// Google Antigravity ターゲット
pub struct AntigravityTarget;

impl AntigravityTarget {
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
            Scope::Personal => Self::home_dir().join(".gemini").join("antigravity"),
            Scope::Project => project_root.join(".agent"),
        }
    }

    /// この組み合わせで配置できるか（Skillのみサポート）
    fn can_place(kind: ComponentKind) -> bool {
        kind == ComponentKind::Skill
    }

    /// コンポーネント種別に応じたフィルタリング（SKILL.md存在チェック維持）
    fn filter_component(c: &ScannedComponent, kind: ComponentKind) -> Option<String> {
        match kind {
            ComponentKind::Skill if c.is_dir => {
                // SKILL.mdが存在する場合のみ有効なSkillとして認識
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

        // scan_components を使用して3層構造を走査
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
