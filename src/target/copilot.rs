//! GitHub Copilot ターゲット実装

use crate::component::{ComponentKind, Scope};
use crate::domain::{ComponentRef, PlacementContext, PlacementLocation, PlacementScope, ProjectContext};
use crate::error::Result;
use crate::target::{PluginOrigin, Target};
use std::fs;
use std::path::{Path, PathBuf};

/// GitHub Copilot ターゲット
pub struct CopilotTarget;

impl CopilotTarget {
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
            Scope::Personal => Self::home_dir().join(".copilot"),
            Scope::Project => project_root.join(".github"),
        }
    }

    /// この組み合わせで配置できるか
    fn can_place(kind: ComponentKind, scope: Scope) -> bool {
        match (kind, scope) {
            (ComponentKind::Agent, _) => true,
            (_, Scope::Project) => true,
            _ => false,
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

    fn supported_components(&self) -> &[ComponentKind] {
        &[
            ComponentKind::Skill,
            ComponentKind::Agent,
            ComponentKind::Command,
            ComponentKind::Instruction,
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
            // 階層構造: prompts/<marketplace>/<plugin>/<name>.prompt.md (ファイル)
            ComponentKind::Command => PlacementLocation::file(
                base.join("prompts")
                    .join(&origin.marketplace)
                    .join(&origin.plugin)
                    .join(format!("{}.prompt.md", name)),
            ),
            ComponentKind::Instruction => PlacementLocation::file(base.join("copilot-instructions.md")),
            ComponentKind::Hook => return None,
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
                Ok(vec!["copilot-instructions.md".to_string()])
            } else {
                Ok(vec![])
            };
        }

        let base = Self::base_dir(scope, project_root);
        let dir_path = match kind {
            ComponentKind::Skill => base.join("skills"),
            ComponentKind::Agent => base.join("agents"),
            ComponentKind::Command => base.join("prompts"),
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
                            names.push(format!("{}/{}/{}", marketplace, plugin, name));
                        }
                        ComponentKind::Agent if name.ends_with(".agent.md") => {
                            let agent_name = name.trim_end_matches(".agent.md").to_string();
                            names.push(format!("{}/{}/{}", marketplace, plugin, agent_name));
                        }
                        ComponentKind::Command if name.ends_with(".prompt.md") => {
                            let command_name = name.trim_end_matches(".prompt.md").to_string();
                            names.push(format!("{}/{}/{}", marketplace, plugin, command_name));
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
mod tests {
    use super::*;

    #[test]
    fn test_copilot_target_name() {
        let target = CopilotTarget::new();
        assert_eq!(target.name(), "copilot");
        assert_eq!(target.display_name(), "GitHub Copilot");
    }

    #[test]
    fn test_copilot_supported_components() {
        let target = CopilotTarget::new();
        assert!(target.supports(ComponentKind::Skill));
        assert!(target.supports(ComponentKind::Agent));
        assert!(target.supports(ComponentKind::Command));
        assert!(target.supports(ComponentKind::Instruction));
        assert!(!target.supports(ComponentKind::Hook));
    }

    #[test]
    fn test_copilot_skill_personal_not_supported() {
        let target = CopilotTarget::new();
        let project_root = Path::new("/project");
        let origin = PluginOrigin::from_marketplace("official", "my-plugin");

        // Personal scope for skills is not supported
        let ctx = PlacementContext {
            component: ComponentRef::new(ComponentKind::Skill, "my-skill"),
            origin: &origin,
            scope: PlacementScope(Scope::Personal),
            project: ProjectContext::new(project_root),
        };
        assert!(target.placement_location(&ctx).is_none());
    }

    #[test]
    fn test_copilot_placement_location_skill_project() {
        let target = CopilotTarget::new();
        let project_root = Path::new("/project");
        let origin = PluginOrigin::from_marketplace("official", "my-plugin");

        let ctx = PlacementContext {
            component: ComponentRef::new(ComponentKind::Skill, "my-skill"),
            origin: &origin,
            scope: PlacementScope(Scope::Project),
            project: ProjectContext::new(project_root),
        };
        let location = target.placement_location(&ctx).unwrap();

        assert!(location.is_dir());
        assert_eq!(
            location.as_path(),
            Path::new("/project/.github/skills/official/my-plugin/my-skill")
        );
    }

    #[test]
    fn test_copilot_placement_location_agent() {
        let target = CopilotTarget::new();
        let project_root = Path::new("/project");
        let origin = PluginOrigin::from_marketplace("official", "my-plugin");

        // Personal scope
        let ctx_personal = PlacementContext {
            component: ComponentRef::new(ComponentKind::Agent, "test"),
            origin: &origin,
            scope: PlacementScope(Scope::Personal),
            project: ProjectContext::new(project_root),
        };
        let location_personal = target.placement_location(&ctx_personal).unwrap();
        assert!(location_personal.is_file());
        assert!(location_personal
            .as_path()
            .to_string_lossy()
            .contains(".copilot/agents/official/my-plugin/test.agent.md"));

        // Project scope
        let ctx_project = PlacementContext {
            component: ComponentRef::new(ComponentKind::Agent, "test"),
            origin: &origin,
            scope: PlacementScope(Scope::Project),
            project: ProjectContext::new(project_root),
        };
        let location_project = target.placement_location(&ctx_project).unwrap();
        assert!(location_project.is_file());
        assert_eq!(
            location_project.as_path(),
            Path::new("/project/.github/agents/official/my-plugin/test.agent.md")
        );
    }

    #[test]
    fn test_copilot_placement_location_command() {
        let target = CopilotTarget::new();
        let project_root = Path::new("/project");
        let origin = PluginOrigin::from_marketplace("official", "my-plugin");

        // Personal scope for commands is not supported
        let ctx_personal = PlacementContext {
            component: ComponentRef::new(ComponentKind::Command, "my-command"),
            origin: &origin,
            scope: PlacementScope(Scope::Personal),
            project: ProjectContext::new(project_root),
        };
        assert!(target.placement_location(&ctx_personal).is_none());

        // Project scope
        let ctx_project = PlacementContext {
            component: ComponentRef::new(ComponentKind::Command, "my-command"),
            origin: &origin,
            scope: PlacementScope(Scope::Project),
            project: ProjectContext::new(project_root),
        };
        let location = target.placement_location(&ctx_project).unwrap();
        assert!(location.is_file());
        assert_eq!(
            location.as_path(),
            Path::new("/project/.github/prompts/official/my-plugin/my-command.prompt.md")
        );
    }

    #[test]
    fn test_copilot_placement_location_instruction() {
        let target = CopilotTarget::new();
        let project_root = Path::new("/project");
        let origin = PluginOrigin::from_marketplace("official", "my-plugin");

        let ctx = PlacementContext {
            component: ComponentRef::new(ComponentKind::Instruction, "test"),
            origin: &origin,
            scope: PlacementScope(Scope::Project),
            project: ProjectContext::new(project_root),
        };
        let location = target.placement_location(&ctx).unwrap();

        assert!(location.is_file());
        assert_eq!(
            location.as_path(),
            Path::new("/project/.github/copilot-instructions.md")
        );
    }
}
