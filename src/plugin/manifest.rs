use crate::error::{PlmError, Result};
use crate::path_ext::PathExt;
use crate::scan::{
    DEFAULT_AGENTS_DIR, DEFAULT_COMMANDS_DIR, DEFAULT_HOOKS_DIR, DEFAULT_INSTRUCTIONS_DIR,
    DEFAULT_INSTRUCTIONS_FILE, DEFAULT_SKILLS_DIR,
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
}

impl PluginManifest {
    /// JSONからパース
    pub fn parse(content: &str) -> Result<Self> {
        serde_json::from_str(content).map_err(|e| {
            PlmError::InvalidManifest(format!("Failed to parse plugin.json: {}", e))
        })
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal() {
        let json = r#"{"name": "test-plugin", "version": "1.0.0"}"#;
        let manifest = PluginManifest::parse(json).unwrap();
        assert_eq!(manifest.name, "test-plugin");
        assert_eq!(manifest.version, "1.0.0");
        assert!(manifest.description.is_none());
    }

    #[test]
    fn test_parse_full() {
        let json = r#"{
            "name": "full-plugin",
            "version": "2.0.0",
            "description": "A full plugin",
            "author": {
                "name": "Test Author",
                "email": "test@example.com"
            },
            "skills": "./skills/",
            "agents": "./agents/"
        }"#;
        let manifest = PluginManifest::parse(json).unwrap();
        assert_eq!(manifest.name, "full-plugin");
        assert_eq!(manifest.version, "2.0.0");
        assert_eq!(manifest.description, Some("A full plugin".to_string()));
        assert!(manifest.author.is_some());
        assert!(manifest.has_skills());
        assert!(manifest.has_agents());
    }

    #[test]
    fn test_parse_invalid() {
        let json = r#"{"name": "test"}"#; // missing version
        assert!(PluginManifest::parse(json).is_err());
    }
}
