//! Google Antigravity ターゲット実装

use crate::component::{ComponentKind, Scope};
use crate::component::{
    ComponentRef, PlacementContext, PlacementLocation, PlacementScope, ProjectContext,
};
use crate::error::Result;
use crate::target::{PluginOrigin, Target};
use std::fs;
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

        if !dir_path.exists() {
            return Ok(vec![]);
        }

        // 階層構造を走査: skills/<marketplace>/<plugin>/<component>
        let mut names = Vec::new();
        for mp_entry in fs::read_dir(&dir_path)? {
            let mp_entry = mp_entry?;
            if !mp_entry.path().is_dir() {
                continue;
            }
            let marketplace = mp_entry.file_name().to_string_lossy().to_string();

            for plugin_entry in fs::read_dir(mp_entry.path())? {
                let plugin_entry = plugin_entry?;
                if !plugin_entry.path().is_dir() {
                    continue;
                }
                let plugin = plugin_entry.file_name().to_string_lossy().to_string();

                for component_entry in fs::read_dir(plugin_entry.path())? {
                    let component_entry = component_entry?;
                    if !component_entry.path().is_dir() {
                        continue;
                    }

                    // SKILL.mdが存在する場合のみ有効なSkillとして認識
                    let skill_md_path = component_entry.path().join("SKILL.md");
                    if skill_md_path.exists() {
                        let name = component_entry.file_name().to_string_lossy().to_string();
                        names.push(format!("{}/{}/{}", marketplace, plugin, name));
                    }
                }
            }
        }

        Ok(names)
    }
}

#[cfg(test)]
#[path = "antigravity_test.rs"]
mod tests;
