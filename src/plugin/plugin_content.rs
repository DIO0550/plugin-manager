//! プラグイン内部モデル
//!
//! パッケージ内の個別プラグインを表現する。コンポーネントスキャン・パス解決の責務を持つ。

use crate::component::{Component, ComponentKind};
use crate::path_ext::PathExt;
use crate::plugin::PluginManifest;
use crate::scan::{scan_components, AGENT_SUFFIX, MARKDOWN_SUFFIX, PROMPT_SUFFIX};
use std::path::{Path, PathBuf};

/// パッケージ内の個別プラグイン
///
/// `name`, `manifest`, `path` を保持し、コンポーネントスキャンとパス解決を担う。
#[derive(Debug, Clone)]
pub struct Plugin {
    pub name: String,
    pub manifest: PluginManifest,
    pub path: PathBuf,
}

impl Plugin {
    // =========================================================================
    // ディレクトリ解決メソッド
    // =========================================================================

    /// スキルディレクトリのパスを解決
    pub fn skills_dir(&self) -> PathBuf {
        self.manifest.skills_dir(&self.path)
    }

    /// エージェントディレクトリのパスを解決
    pub fn agents_dir(&self) -> PathBuf {
        self.manifest.agents_dir(&self.path)
    }

    /// コマンドディレクトリのパスを解決
    pub fn commands_dir(&self) -> PathBuf {
        self.manifest.commands_dir(&self.path)
    }

    /// インストラクションパスを解決
    pub fn instructions_path(&self) -> PathBuf {
        self.manifest.instructions_path(&self.path)
    }

    /// フックディレクトリのパスを解決
    pub fn hooks_dir(&self) -> PathBuf {
        self.manifest.hooks_dir(&self.path)
    }

    // =========================================================================
    // スキャンメソッド
    // =========================================================================

    /// プラグイン内のコンポーネントをスキャン
    pub fn components(&self) -> Vec<Component> {
        let scan = scan_components(&self.path, &self.manifest);
        let mut components = Vec::new();

        // Skills
        let skills_dir = self.skills_dir();
        for name in scan.skills {
            components.push(Component {
                kind: ComponentKind::Skill,
                path: skills_dir.join(&name),
                name,
            });
        }

        // Agents
        let agents_dir = self.agents_dir();
        for name in scan.agents {
            let path = self.resolve_agent_path(&agents_dir, &name);
            components.push(Component {
                kind: ComponentKind::Agent,
                path,
                name,
            });
        }

        // Commands
        let commands_dir = self.commands_dir();
        for name in scan.commands {
            let path = self.resolve_command_path(&commands_dir, &name);
            components.push(Component {
                kind: ComponentKind::Command,
                path,
                name,
            });
        }

        // Instructions
        for name in scan.instructions {
            let path = self.resolve_instruction_path(&name);
            components.push(Component {
                kind: ComponentKind::Instruction,
                path,
                name,
            });
        }

        // Hooks
        let hooks_dir = self.hooks_dir();
        for name in scan.hooks {
            if let Some(path) = self.resolve_hook_path(&hooks_dir, &name) {
                components.push(Component {
                    kind: ComponentKind::Hook,
                    path,
                    name,
                });
            }
        }

        components
    }

    // =========================================================================
    // パス解決ヘルパー（名前 → パス）
    // =========================================================================

    fn resolve_agent_path(&self, agents_dir: &Path, name: &str) -> PathBuf {
        if agents_dir.is_file() {
            return agents_dir.to_path_buf();
        }

        let agent_path = agents_dir.join(format!("{}{}", name, AGENT_SUFFIX));
        if agent_path.exists() {
            agent_path
        } else {
            agents_dir.join(format!("{}{}", name, MARKDOWN_SUFFIX))
        }
    }

    fn resolve_command_path(&self, commands_dir: &Path, name: &str) -> PathBuf {
        let prompt_path = commands_dir.join(format!("{}{}", name, PROMPT_SUFFIX));
        if prompt_path.exists() {
            prompt_path
        } else {
            commands_dir.join(format!("{}{}", name, MARKDOWN_SUFFIX))
        }
    }

    fn resolve_instruction_path(&self, name: &str) -> PathBuf {
        if name == "AGENTS" {
            return self.path.join("AGENTS.md");
        }

        if let Some(path_str) = &self.manifest.instructions {
            let path = self.path.join(path_str);
            if path.is_file() {
                return path;
            }
            if path.is_dir() {
                return path.join(format!("{}.md", name));
            }
        }

        self.manifest
            .instructions_dir(&self.path)
            .join(format!("{}.md", name))
    }

    fn resolve_hook_path(&self, hooks_dir: &Path, name: &str) -> Option<PathBuf> {
        hooks_dir
            .read_dir_entries()
            .into_iter()
            .filter(|p| p.is_file())
            .find(|path| {
                path.file_name()
                    .and_then(|f| f.to_str())
                    .map(|f| f.rsplit_once('.').map(|(n, _)| n).unwrap_or(f) == name)
                    .unwrap_or(false)
            })
    }
}
