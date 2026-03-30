//! マーケットプレイスパッケージ（内部ドメイン型）
//!
//! コンポーネント群（skills, agents, commands, instructions, hooks）を含む
//! マーケットプレイスのパッケージ。コンポーネントスキャン・パス解決を担う。

use crate::component::{AgentFormat, CommandFormat, Component, ComponentKind};
use crate::path_ext::PathExt;
use crate::plugin::PluginManifest;
use crate::scan::{scan_components, AGENT_SUFFIX, MARKDOWN_SUFFIX, PROMPT_SUFFIX};
use std::path::{Path, PathBuf};

use super::cached_plugin::RemoteMarketplaceData;

/// マーケットプレイスパッケージ（内部ドメイン型）
///
/// コンポーネント群（skills, agents, commands, instructions, hooks）を含む
/// マーケットプレイスのパッケージ。コンポーネントスキャン・パス解決を担う。
#[derive(Debug, Clone)]
pub struct MarketplacePackage {
    pub name: String,
    pub marketplace: Option<String>,
    pub path: PathBuf,
    pub manifest: PluginManifest,
}

impl MarketplacePackage {
    /// Command コンポーネントのソース形式を取得
    pub fn command_format(&self) -> CommandFormat {
        match self.marketplace.as_deref() {
            Some("claude") => CommandFormat::ClaudeCode,
            Some("copilot") => CommandFormat::Copilot,
            Some("codex") => CommandFormat::Codex,
            _ => CommandFormat::ClaudeCode,
        }
    }

    /// Agent コンポーネントのソース形式を取得
    pub fn agent_format(&self) -> AgentFormat {
        match self.marketplace.as_deref() {
            Some("claude") => AgentFormat::ClaudeCode,
            Some("copilot") => AgentFormat::Copilot,
            Some("codex") => AgentFormat::Codex,
            _ => AgentFormat::ClaudeCode,
        }
    }

    // =========================================================================
    // パス解決メソッド
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
    ///
    /// 統一スキャンAPI (`scan_components`) を使用し、名前からパスを解決して
    /// `Component` に変換する。パス解決の責務は本メソッドが担う。
    pub fn components(&self) -> Vec<Component> {
        let scan = scan_components(&self.path, &self.manifest);
        let mut components = Vec::new();

        // Skills: skills_dir/name/
        let skills_dir = self.skills_dir();
        for name in scan.skills {
            components.push(Component {
                kind: ComponentKind::Skill,
                path: skills_dir.join(&name),
                name,
            });
        }

        // Agents: agents_dir/name.agent.md or agents_dir/name.md or single file
        let agents_dir = self.agents_dir();
        for name in scan.agents {
            let path = self.resolve_agent_path(&agents_dir, &name);
            components.push(Component {
                kind: ComponentKind::Agent,
                path,
                name,
            });
        }

        // Commands: commands_dir/name.prompt.md or commands_dir/name.md
        let commands_dir = self.commands_dir();
        for name in scan.commands {
            let path = self.resolve_command_path(&commands_dir, &name);
            components.push(Component {
                kind: ComponentKind::Command,
                path,
                name,
            });
        }

        // Instructions: instructions_path or instructions_dir/name.md
        for name in scan.instructions {
            let path = self.resolve_instruction_path(&name);
            components.push(Component {
                kind: ComponentKind::Instruction,
                path,
                name,
            });
        }

        // Hooks: hooks_dir/name.*
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

    /// Agent のパスを解決
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

    /// Command のパスを解決
    fn resolve_command_path(&self, commands_dir: &Path, name: &str) -> PathBuf {
        let prompt_path = commands_dir.join(format!("{}{}", name, PROMPT_SUFFIX));
        if prompt_path.exists() {
            prompt_path
        } else {
            commands_dir.join(format!("{}{}", name, MARKDOWN_SUFFIX))
        }
    }

    /// Instruction のパスを解決
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

    /// Hook のパスを解決
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

impl From<RemoteMarketplaceData> for MarketplacePackage {
    fn from(remote: RemoteMarketplaceData) -> Self {
        Self {
            name: remote.name,
            marketplace: remote.marketplace,
            path: remote.path,
            manifest: remote.manifest,
        }
    }
}
