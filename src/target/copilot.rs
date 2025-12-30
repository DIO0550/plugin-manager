//! GitHub Copilot ターゲット実装

use crate::component::ComponentKind;
use crate::error::Result;
use crate::target::{Scope, Target};
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
        project_root: &Path,
    ) -> Option<PathBuf> {
        if !Self::can_place(kind, scope) {
            return None;
        }

        let base = Self::base_dir(scope, project_root);

        Some(match kind {
            ComponentKind::Skill => base.join("skills").join(component_name),
            ComponentKind::Agent => base.join("agents"),
            ComponentKind::Prompt => base.join("prompts"),
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
            let path = self.placement_path(kind, scope, "", project_root).unwrap();
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

        let mut names = Vec::new();
        for entry in fs::read_dir(&dir_path)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();

            match kind {
                ComponentKind::Skill if entry.path().is_dir() => {
                    names.push(name);
                }
                ComponentKind::Agent if name.ends_with(".agent.md") => {
                    names.push(name.trim_end_matches(".agent.md").to_string());
                }
                ComponentKind::Prompt if name.ends_with(".prompt.md") => {
                    names.push(name.trim_end_matches(".prompt.md").to_string());
                }
                _ => {}
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

        // Personal scope for skills is not supported
        let path =
            target.placement_path(ComponentKind::Skill, Scope::Personal, "my-skill", project_root);
        assert!(path.is_none());
    }

    #[test]
    fn test_copilot_skill_project_supported() {
        let target = CopilotTarget::new();
        let project_root = Path::new("/project");

        // Project scope for skills is supported
        let path = target
            .placement_path(ComponentKind::Skill, Scope::Project, "my-skill", project_root)
            .unwrap();
        assert_eq!(path, PathBuf::from("/project/.github/skills/my-skill"));
    }

    #[test]
    fn test_copilot_agent_paths() {
        let target = CopilotTarget::new();
        let project_root = Path::new("/project");

        // Personal scope
        let personal_path = target
            .placement_path(ComponentKind::Agent, Scope::Personal, "test", project_root)
            .unwrap();
        assert!(personal_path.to_string_lossy().contains(".copilot/agents"));

        // Project scope
        let project_path = target
            .placement_path(ComponentKind::Agent, Scope::Project, "test", project_root)
            .unwrap();
        assert_eq!(project_path, PathBuf::from("/project/.github/agents"));
    }
}
