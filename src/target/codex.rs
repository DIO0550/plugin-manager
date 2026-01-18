//! OpenAI Codex ターゲット実装

use crate::component::{ComponentKind, Scope};
use crate::component::{
    ComponentRef, PlacementContext, PlacementLocation, PlacementScope, ProjectContext,
};
use crate::error::Result;
use crate::target::{PluginOrigin, Target};
use std::fs;
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

        if !dir_path.exists() {
            return Ok(vec![]);
        }

        // 階層構造を走査: <kind>/<marketplace>/<plugin>/<component>
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
                    let name = component_entry.file_name().to_string_lossy().to_string();

                    match kind {
                        ComponentKind::Skill if component_entry.path().is_dir() => {
                            // 完全修飾名: marketplace/plugin/component
                            names.push(format!("{}/{}/{}", marketplace, plugin, name));
                        }
                        ComponentKind::Agent if name.ends_with(".agent.md") => {
                            let agent_name = name.trim_end_matches(".agent.md").to_string();
                            names.push(format!("{}/{}/{}", marketplace, plugin, agent_name));
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(names)
    }
}

#[cfg(test)]
#[path = "codex_test.rs"]
mod tests;
