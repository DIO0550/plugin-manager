//! GitHub Copilot ターゲット実装

use crate::component::ComponentKind;
use crate::error::Result;
use crate::target::{PluginOrigin, Scope, Target};
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
            ComponentKind::Prompt,
            ComponentKind::Instruction,
        ]
    }

    fn placement_path(
        &self,
        kind: ComponentKind,
        scope: Scope,
        component_name: &str,
        origin: &PluginOrigin,
        project_root: &Path,
    ) -> Option<PathBuf> {
        if !Self::can_place(kind, scope) {
            return None;
        }

        let base = Self::base_dir(scope, project_root);

        Some(match kind {
            // 階層構造: skills/<marketplace>/<plugin>/<skill>
            ComponentKind::Skill => base
                .join("skills")
                .join(&origin.marketplace)
                .join(&origin.plugin)
                .join(component_name),
            // 階層構造: agents/<marketplace>/<plugin>/
            ComponentKind::Agent => base
                .join("agents")
                .join(&origin.marketplace)
                .join(&origin.plugin),
            // 階層構造: prompts/<marketplace>/<plugin>/
            ComponentKind::Prompt => base
                .join("prompts")
                .join(&origin.marketplace)
                .join(&origin.plugin),
            ComponentKind::Instruction => base.join("copilot-instructions.md"),
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
            let path = self
                .placement_path(kind, scope, "", &dummy_origin, project_root)
                .unwrap();
            return if path.exists() {
                Ok(vec!["copilot-instructions.md".to_string()])
            } else {
                Ok(vec![])
            };
        }

        let base = Self::base_dir(scope, project_root);
        let dir_path = match kind {
            ComponentKind::Skill => base.join("skills"),
            ComponentKind::Agent => base.join("agents"),
            ComponentKind::Prompt => base.join("prompts"),
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
                        ComponentKind::Prompt if name.ends_with(".prompt.md") => {
                            let prompt_name = name.trim_end_matches(".prompt.md").to_string();
                            names.push(format!("{}/{}/{}", marketplace, plugin, prompt_name));
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
        assert!(target.supports(ComponentKind::Prompt));
        assert!(target.supports(ComponentKind::Instruction));
    }

    #[test]
    fn test_copilot_skill_personal_not_supported() {
        let target = CopilotTarget::new();
        let project_root = Path::new("/project");
        let origin = PluginOrigin::from_marketplace("official", "my-plugin");

        // Personal scope for skills is not supported
        let path = target.placement_path(
            ComponentKind::Skill,
            Scope::Personal,
            "my-skill",
            &origin,
            project_root,
        );
        assert!(path.is_none());
    }

    #[test]
    fn test_copilot_skill_project_with_hierarchy() {
        let target = CopilotTarget::new();
        let project_root = Path::new("/project");
        let origin = PluginOrigin::from_marketplace("official", "my-plugin");

        // Project scope for skills with hierarchy
        let path = target
            .placement_path(
                ComponentKind::Skill,
                Scope::Project,
                "my-skill",
                &origin,
                project_root,
            )
            .unwrap();
        assert_eq!(
            path,
            PathBuf::from("/project/.github/skills/official/my-plugin/my-skill")
        );
    }

    #[test]
    fn test_copilot_agent_paths_with_hierarchy() {
        let target = CopilotTarget::new();
        let project_root = Path::new("/project");
        let origin = PluginOrigin::from_marketplace("official", "my-plugin");

        // Personal scope
        let personal_path = target
            .placement_path(
                ComponentKind::Agent,
                Scope::Personal,
                "test",
                &origin,
                project_root,
            )
            .unwrap();
        assert!(personal_path
            .to_string_lossy()
            .contains(".copilot/agents/official/my-plugin"));

        // Project scope
        let project_path = target
            .placement_path(
                ComponentKind::Agent,
                Scope::Project,
                "test",
                &origin,
                project_root,
            )
            .unwrap();
        assert_eq!(
            project_path,
            PathBuf::from("/project/.github/agents/official/my-plugin")
        );
    }
}
