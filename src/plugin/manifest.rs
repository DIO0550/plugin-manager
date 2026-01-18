use crate::error::{PlmError, Result};
use crate::path_ext::PathExt;
use crate::scan::{
    InstructionPath, ScanPaths, DEFAULT_AGENTS_DIR, DEFAULT_COMMANDS_DIR, DEFAULT_HOOKS_DIR,
    DEFAULT_INSTRUCTIONS_DIR, DEFAULT_INSTRUCTIONS_FILE, DEFAULT_SKILLS_DIR,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// プラグイン作者情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

/// plugin.json のスキーマ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<Author>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub keywords: Option<Vec<String>>,

    // コンポーネントパス（プラグインルートからの相対パス）
    #[serde(default)]
    pub commands: Option<String>,
    #[serde(default)]
    pub agents: Option<String>,
    #[serde(default)]
    pub skills: Option<String>,
    #[serde(default)]
    pub instructions: Option<String>,
    #[serde(default)]
    pub hooks: Option<String>,
    #[serde(default, rename = "mcpServers")]
    pub mcp_servers: Option<String>,
    #[serde(default, rename = "lspServers")]
    pub lsp_servers: Option<String>,

    /// インストール日時（RFC3339形式、例: "2025-01-15T10:30:00Z"）
    /// 後方互換のため読み込みのみ対応。新規インストールでは .plm-meta.json に記録される。
    #[serde(default, rename = "installedAt", skip_serializing)]
    pub installed_at: Option<String>,
}

impl PluginManifest {
    /// JSONからパース
    pub fn parse(content: &str) -> Result<Self> {
        serde_json::from_str(content)
            .map_err(|e| PlmError::InvalidManifest(format!("Failed to parse plugin.json: {}", e)))
    }

    /// ファイルから読み込み
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// スキルが含まれているか
    pub fn has_skills(&self) -> bool {
        self.skills.is_some()
    }

    /// エージェントが含まれているか
    pub fn has_agents(&self) -> bool {
        self.agents.is_some()
    }

    /// コマンドが含まれているか
    pub fn has_commands(&self) -> bool {
        self.commands.is_some()
    }

    /// インストラクションが含まれているか
    pub fn has_instructions(&self) -> bool {
        self.instructions.is_some()
    }

    // =========================================================================
    // パス解決メソッド
    // =========================================================================

    /// スキルディレクトリのパスを解決
    pub fn skills_dir(&self, base: &Path) -> PathBuf {
        base.join_or(self.skills.as_deref(), DEFAULT_SKILLS_DIR)
    }

    /// エージェントディレクトリのパスを解決
    pub fn agents_dir(&self, base: &Path) -> PathBuf {
        base.join_or(self.agents.as_deref(), DEFAULT_AGENTS_DIR)
    }

    /// コマンドディレクトリのパスを解決
    pub fn commands_dir(&self, base: &Path) -> PathBuf {
        base.join_or(self.commands.as_deref(), DEFAULT_COMMANDS_DIR)
    }

    /// インストラクションパスを解決（ファイルまたはディレクトリ）
    pub fn instructions_path(&self, base: &Path) -> PathBuf {
        base.join_or(self.instructions.as_deref(), DEFAULT_INSTRUCTIONS_FILE)
    }

    /// インストラクションディレクトリのパスを解決（デフォルトディレクトリ用）
    pub fn instructions_dir(&self, base: &Path) -> PathBuf {
        base.join_or(self.instructions.as_deref(), DEFAULT_INSTRUCTIONS_DIR)
    }

    /// フックディレクトリのパスを解決
    pub fn hooks_dir(&self, base: &Path) -> PathBuf {
        base.join_or(self.hooks.as_deref(), DEFAULT_HOOKS_DIR)
    }

    // =========================================================================
    // ScanPaths 変換
    // =========================================================================

    /// ScanPaths への変換（scan モジュール用）
    ///
    /// ドメイン非依存の `scan_from_paths()` API で使用するための変換メソッド。
    /// マニフェストのパス設定を解決し、`ScanPaths` 構造体を生成する。
    pub fn to_scan_paths(&self, plugin_path: &Path) -> ScanPaths {
        let instructions = if let Some(path_str) = &self.instructions {
            let path = plugin_path.join(path_str);
            if path.is_file() {
                InstructionPath::File(path)
            } else if path.is_dir() {
                InstructionPath::Dir(path)
            } else {
                // 存在しない場合はデフォルトにフォールバック
                InstructionPath::Default {
                    instructions_dir: self.instructions_dir(plugin_path),
                    root_agents_md: plugin_path.join("AGENTS.md"),
                }
            }
        } else {
            InstructionPath::Default {
                instructions_dir: self.instructions_dir(plugin_path),
                root_agents_md: plugin_path.join("AGENTS.md"),
            }
        };

        ScanPaths {
            skills_dir: self.skills_dir(plugin_path),
            agents_path: self.agents_dir(plugin_path),
            commands_dir: self.commands_dir(plugin_path),
            instructions,
            hooks_dir: self.hooks_dir(plugin_path),
        }
    }
}

#[cfg(test)]
#[path = "manifest_test.rs"]
mod tests;
