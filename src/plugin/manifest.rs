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

    /// インストール日時（RFC3339形式、例: "2025-01-15T10:30:00Z"）
    /// 後方互換のため読み込みのみ対応。新規インストールでは .plm-meta.json に記録される。
    #[serde(default, rename = "installedAt", skip_serializing)]
    pub installed_at: Option<String>,
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

    // === 境界値テスト: 必須フィールド ===

    #[test]
    fn test_parse_missing_name() {
        // name 欠落
        let json = r#"{"version": "1.0.0"}"#;
        assert!(PluginManifest::parse(json).is_err());
    }

    #[test]
    fn test_parse_missing_version() {
        // version 欠落
        let json = r#"{"name": "test"}"#;
        assert!(PluginManifest::parse(json).is_err());
    }

    #[test]
    fn test_parse_name_wrong_type() {
        // name が数値
        let json = r#"{"name": 123, "version": "1.0.0"}"#;
        assert!(PluginManifest::parse(json).is_err());
    }

    #[test]
    fn test_parse_version_wrong_type() {
        // version が数値
        let json = r#"{"name": "test", "version": 100}"#;
        assert!(PluginManifest::parse(json).is_err());
    }

    #[test]
    fn test_parse_empty_name() {
        // 空文字の name（パースは成功するが意味的には無効）
        let json = r#"{"name": "", "version": "1.0.0"}"#;
        let manifest = PluginManifest::parse(json).unwrap();
        assert_eq!(manifest.name, "");
    }

    #[test]
    fn test_parse_empty_version() {
        // 空文字の version
        let json = r#"{"name": "test", "version": ""}"#;
        let manifest = PluginManifest::parse(json).unwrap();
        assert_eq!(manifest.version, "");
    }

    // === 境界値テスト: 空文字パス ===

    #[test]
    fn test_parse_empty_skills_path() {
        // skills: "" の場合
        let json = r#"{"name": "test", "version": "1.0.0", "skills": ""}"#;
        let manifest = PluginManifest::parse(json).unwrap();
        assert!(manifest.has_skills());
        assert_eq!(manifest.skills, Some("".to_string()));
    }

    #[test]
    fn test_parse_empty_agents_path() {
        // agents: "" の場合
        let json = r#"{"name": "test", "version": "1.0.0", "agents": ""}"#;
        let manifest = PluginManifest::parse(json).unwrap();
        assert!(manifest.has_agents());
    }

    // === 境界値テスト: パス解決 ===

    #[test]
    fn test_skills_dir_default() {
        // デフォルトパス
        let json = r#"{"name": "test", "version": "1.0.0"}"#;
        let manifest = PluginManifest::parse(json).unwrap();
        let base = Path::new("/plugin");
        assert_eq!(manifest.skills_dir(base), Path::new("/plugin/skills"));
    }

    #[test]
    fn test_skills_dir_custom() {
        // カスタムパス
        let json = r#"{"name": "test", "version": "1.0.0", "skills": "my-skills"}"#;
        let manifest = PluginManifest::parse(json).unwrap();
        let base = Path::new("/plugin");
        assert_eq!(manifest.skills_dir(base), Path::new("/plugin/my-skills"));
    }

    #[test]
    fn test_skills_dir_empty_uses_empty_path() {
        // 空文字の場合は空パスとして扱われる
        let json = r#"{"name": "test", "version": "1.0.0", "skills": ""}"#;
        let manifest = PluginManifest::parse(json).unwrap();
        let base = Path::new("/plugin");
        // 空文字は join で base そのものになる
        assert_eq!(manifest.skills_dir(base), Path::new("/plugin/"));
    }

    #[test]
    fn test_skills_dir_with_dot_slash() {
        // ./ プレフィックス
        let json = r#"{"name": "test", "version": "1.0.0", "skills": "./skills"}"#;
        let manifest = PluginManifest::parse(json).unwrap();
        let base = Path::new("/plugin");
        assert_eq!(manifest.skills_dir(base), Path::new("/plugin/./skills"));
    }

    #[test]
    fn test_skills_dir_absolute_path() {
        // 絶対パス指定
        let json = r#"{"name": "test", "version": "1.0.0", "skills": "/absolute/path"}"#;
        let manifest = PluginManifest::parse(json).unwrap();
        let base = Path::new("/plugin");
        // 絶対パスは base を置換する（Path::join の仕様）
        assert_eq!(manifest.skills_dir(base), Path::new("/absolute/path"));
    }

    #[test]
    fn test_agents_dir_default() {
        let json = r#"{"name": "test", "version": "1.0.0"}"#;
        let manifest = PluginManifest::parse(json).unwrap();
        let base = Path::new("/plugin");
        assert_eq!(manifest.agents_dir(base), Path::new("/plugin/agents"));
    }

    #[test]
    fn test_instructions_path_default() {
        let json = r#"{"name": "test", "version": "1.0.0"}"#;
        let manifest = PluginManifest::parse(json).unwrap();
        let base = Path::new("/plugin");
        assert_eq!(
            manifest.instructions_path(base),
            Path::new("/plugin/instructions.md")
        );
    }

    #[test]
    fn test_instructions_dir_default() {
        let json = r#"{"name": "test", "version": "1.0.0"}"#;
        let manifest = PluginManifest::parse(json).unwrap();
        let base = Path::new("/plugin");
        assert_eq!(
            manifest.instructions_dir(base),
            Path::new("/plugin/instructions")
        );
    }

    #[test]
    fn test_hooks_dir_default() {
        let json = r#"{"name": "test", "version": "1.0.0"}"#;
        let manifest = PluginManifest::parse(json).unwrap();
        let base = Path::new("/plugin");
        assert_eq!(manifest.hooks_dir(base), Path::new("/plugin/hooks"));
    }

    #[test]
    fn test_commands_dir_default() {
        let json = r#"{"name": "test", "version": "1.0.0"}"#;
        let manifest = PluginManifest::parse(json).unwrap();
        let base = Path::new("/plugin");
        assert_eq!(manifest.commands_dir(base), Path::new("/plugin/commands"));
    }

    // === 境界値テスト: JSON不正 ===

    #[test]
    fn test_parse_invalid_json() {
        // 不正なJSON
        let json = r#"{"name": "test", version: "1.0.0"}"#;
        assert!(PluginManifest::parse(json).is_err());
    }

    #[test]
    fn test_parse_empty_json() {
        // 空のJSON
        let json = "{}";
        assert!(PluginManifest::parse(json).is_err());
    }

    #[test]
    fn test_parse_empty_string() {
        // 空文字
        let json = "";
        assert!(PluginManifest::parse(json).is_err());
    }

    #[test]
    fn test_parse_null_fields() {
        // null 値
        let json = r#"{"name": "test", "version": "1.0.0", "skills": null}"#;
        let manifest = PluginManifest::parse(json).unwrap();
        assert!(!manifest.has_skills());
    }

    // === installed_at フィールドのテスト ===

    #[test]
    fn test_parse_with_installed_at() {
        let json = r#"{
            "name": "test",
            "version": "1.0.0",
            "installedAt": "2025-01-15T10:30:00Z"
        }"#;
        let manifest = PluginManifest::parse(json).unwrap();
        assert_eq!(
            manifest.installed_at,
            Some("2025-01-15T10:30:00Z".to_string())
        );
    }

    #[test]
    fn test_parse_without_installed_at() {
        let json = r#"{"name": "test", "version": "1.0.0"}"#;
        let manifest = PluginManifest::parse(json).unwrap();
        assert!(manifest.installed_at.is_none());
    }

    #[test]
    fn test_parse_installed_at_null() {
        let json = r#"{"name": "test", "version": "1.0.0", "installedAt": null}"#;
        let manifest = PluginManifest::parse(json).unwrap();
        assert!(manifest.installed_at.is_none());
    }
}
