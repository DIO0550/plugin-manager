//! キャッシュされたプラグイン情報
//!
//! プラグインキャッシュから読み込んだプラグインの情報と、
//! コンポーネントスキャン機能を提供する。

use crate::component::{AgentFormat, CommandFormat, Component, ComponentKind};
use crate::path_ext::PathExt;
use crate::plugin::PluginManifest;
use crate::scan::{scan_components, AGENT_SUFFIX, MARKDOWN_SUFFIX, PROMPT_SUFFIX};
use std::path::{Path, PathBuf};

/// キャッシュされたプラグイン情報
#[derive(Debug, Clone)]
pub struct CachedPlugin {
    pub name: String,
    /// マーケットプレイス名（marketplace経由の場合）
    /// None の場合は直接GitHubからインストール
    pub marketplace: Option<String>,
    pub path: PathBuf,
    pub manifest: PluginManifest,
    pub git_ref: String,
    pub commit_sha: String,
}

impl CachedPlugin {
    /// プラグインのバージョンを取得
    pub fn version(&self) -> &str {
        &self.manifest.version
    }

    /// プラグインの説明を取得
    pub fn description(&self) -> Option<&str> {
        self.manifest.description.as_deref()
    }

    /// スキルが含まれているか
    pub fn has_skills(&self) -> bool {
        self.manifest.has_skills()
    }

    /// スキルのパスを取得
    pub fn skills(&self) -> Option<&str> {
        self.manifest.skills.as_deref()
    }

    /// エージェントが含まれているか
    pub fn has_agents(&self) -> bool {
        self.manifest.has_agents()
    }

    /// エージェントのパスを取得
    pub fn agents(&self) -> Option<&str> {
        self.manifest.agents.as_deref()
    }

    /// コマンドが含まれているか
    pub fn has_commands(&self) -> bool {
        self.manifest.has_commands()
    }

    /// コマンドのパスを取得
    pub fn commands(&self) -> Option<&str> {
        self.manifest.commands.as_deref()
    }

    /// インストラクションが含まれているか
    pub fn has_instructions(&self) -> bool {
        self.manifest.has_instructions()
    }

    /// インストラクションのパスを取得
    pub fn instructions(&self) -> Option<&str> {
        self.manifest.instructions.as_deref()
    }

    /// フックが含まれているか
    pub fn has_hooks(&self) -> bool {
        self.manifest.hooks.is_some()
    }

    /// フックのパスを取得
    pub fn hooks(&self) -> Option<&str> {
        self.manifest.hooks.as_deref()
    }

    /// Command コンポーネントのソース形式を取得
    ///
    /// marketplace フィールドから判定する。
    /// - `Some("claude")` → ClaudeCode
    /// - `Some("copilot")` → Copilot（将来対応）
    /// - `Some("codex")` → Codex（将来対応）
    /// - `None` → ClaudeCode（デフォルト）
    pub fn command_format(&self) -> CommandFormat {
        match self.marketplace.as_deref() {
            Some("claude") => CommandFormat::ClaudeCode,
            Some("copilot") => CommandFormat::Copilot,
            Some("codex") => CommandFormat::Codex,
            // デフォルトは ClaudeCode（現時点で対応しているマーケットプレイスは Claude Code のみ）
            _ => CommandFormat::ClaudeCode,
        }
    }

    /// Agent コンポーネントのソース形式を取得
    ///
    /// marketplace フィールドから判定する。
    /// - `Some("claude")` → ClaudeCode
    /// - `Some("copilot")` → Copilot（将来対応）
    /// - `Some("codex")` → Codex（将来対応）
    /// - `None` → ClaudeCode（デフォルト）
    pub fn agent_format(&self) -> AgentFormat {
        match self.marketplace.as_deref() {
            Some("claude") => AgentFormat::ClaudeCode,
            Some("copilot") => AgentFormat::Copilot,
            Some("codex") => AgentFormat::Codex,
            // デフォルトは ClaudeCode（現時点で対応しているマーケットプレイスは Claude Code のみ）
            _ => AgentFormat::ClaudeCode,
        }
    }

    // =========================================================================
    // パス解決メソッド（デメテルの法則準拠）
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
        // 単一ファイルの場合
        if agents_dir.is_file() {
            return agents_dir.to_path_buf();
        }

        // .agent.md を優先、なければ .md
        let agent_path = agents_dir.join(format!("{}{}", name, AGENT_SUFFIX));
        if agent_path.exists() {
            agent_path
        } else {
            agents_dir.join(format!("{}{}", name, MARKDOWN_SUFFIX))
        }
    }

    /// Command のパスを解決
    fn resolve_command_path(&self, commands_dir: &Path, name: &str) -> PathBuf {
        // .prompt.md を優先、なければ .md
        let prompt_path = commands_dir.join(format!("{}{}", name, PROMPT_SUFFIX));
        if prompt_path.exists() {
            prompt_path
        } else {
            commands_dir.join(format!("{}{}", name, MARKDOWN_SUFFIX))
        }
    }

    /// Instruction のパスを解決
    fn resolve_instruction_path(&self, name: &str) -> PathBuf {
        // AGENTS.md の場合はルートディレクトリ
        if name == "AGENTS" {
            return self.path.join("AGENTS.md");
        }

        // マニフェストで指定されている場合
        if let Some(path_str) = &self.manifest.instructions {
            let path = self.path.join(path_str);
            if path.is_file() {
                return path;
            }
            if path.is_dir() {
                return path.join(format!("{}.md", name));
            }
        }

        // デフォルト: instructions/name.md
        self.manifest
            .instructions_dir(&self.path)
            .join(format!("{}.md", name))
    }

    /// Hook のパスを解決
    fn resolve_hook_path(&self, hooks_dir: &Path, name: &str) -> Option<PathBuf> {
        // hooks_dir 内のファイルを走査して名前が一致するものを探す
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
