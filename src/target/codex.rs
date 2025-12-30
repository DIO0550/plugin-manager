//! OpenAI Codex ターゲット実装

use crate::component::ComponentKind;
use crate::error::Result;
use crate::target::{Scope, Target};
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
        kind != ComponentKind::Prompt
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

    fn placement_path(
        &self,
        kind: ComponentKind,
        scope: Scope,
        component_name: &str,
        project_root: &Path,
    ) -> Option<PathBuf> {
        if !Self::can_place(kind) {
            return None;
        }

        let base = Self::base_dir(scope, project_root);

        Some(match kind {
            ComponentKind::Skill => base.join("skills").join(component_name),
            ComponentKind::Agent => base.join("agents"),
            ComponentKind::Instruction => match scope {
                // Project scope: AGENTS.md is at project root, not in .codex
                Scope::Project => project_root.join("AGENTS.md"),
                Scope::Personal => base.join("AGENTS.md"),
            },
            ComponentKind::Prompt => unreachable!(), // Already filtered by is_scope_supported
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
            let path = self.placement_path(kind, scope, "", project_root).unwrap();
            return if path.exists() {
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
    fn test_codex_target_name() {
        let target = CodexTarget::new();
        assert_eq!(target.name(), "codex");
        assert_eq!(target.display_name(), "OpenAI Codex");
    }

    #[test]
    fn test_codex_supported_components() {
        let target = CodexTarget::new();
        assert!(target.supports(ComponentKind::Skill));
        assert!(target.supports(ComponentKind::Agent));
        assert!(target.supports(ComponentKind::Instruction));
        assert!(!target.supports(ComponentKind::Prompt));
    }

    #[test]
    fn test_codex_placement_path_skill() {
        let target = CodexTarget::new();
        let project_root = Path::new("/project");

        // Project scope
        let path = target
            .placement_path(ComponentKind::Skill, Scope::Project, "my-skill", project_root)
            .unwrap();
        assert_eq!(path, PathBuf::from("/project/.codex/skills/my-skill"));
    }

    #[test]
    fn test_codex_prompt_not_supported() {
        let target = CodexTarget::new();
        let project_root = Path::new("/project");

        let path =
            target.placement_path(ComponentKind::Prompt, Scope::Project, "test", project_root);
        assert!(path.is_none());
    }
}
