//! コンポーネント種別の定義
//!
//! プラグインに含まれるコンポーネントの種類を定義する。

use crate::error::{PlmError, Result};
use clap::ValueEnum;
use std::fs;
use std::path::{Path, PathBuf};

/// コンポーネント種別
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
pub enum ComponentKind {
    /// スキル（SKILL.md形式）
    Skill,
    /// エージェント（.agent.md形式）
    Agent,
    /// プロンプト（.prompt.md形式）
    Prompt,
    /// インストラクション（AGENTS.md, copilot-instructions.md形式）
    Instruction,
}

impl ComponentKind {
    /// 識別子文字列を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            ComponentKind::Skill => "skill",
            ComponentKind::Agent => "agent",
            ComponentKind::Prompt => "prompt",
            ComponentKind::Instruction => "instruction",
        }
    }

    /// 複数形の文字列を取得
    pub fn plural(&self) -> &'static str {
        match self {
            ComponentKind::Skill => "skills",
            ComponentKind::Agent => "agents",
            ComponentKind::Prompt => "prompts",
            ComponentKind::Instruction => "instructions",
        }
    }

    /// 表示名を取得
    pub fn display_name(&self) -> &'static str {
        match self {
            ComponentKind::Skill => "Skill",
            ComponentKind::Agent => "Agent",
            ComponentKind::Prompt => "Prompt",
            ComponentKind::Instruction => "Instruction",
        }
    }

    /// 全コンポーネント種別を取得
    pub fn all() -> &'static [ComponentKind] {
        &[
            ComponentKind::Skill,
            ComponentKind::Agent,
            ComponentKind::Prompt,
            ComponentKind::Instruction,
        ]
    }
}

impl std::fmt::Display for ComponentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// プラグイン内のコンポーネント
#[derive(Debug, Clone)]
pub struct Component {
    pub kind: ComponentKind,
    pub name: String,
    pub path: PathBuf,
}

/// デプロイスコープ
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Scope {
    /// ユーザーレベル（~/.codex/, ~/.copilot/）
    Personal,
    /// プロジェクトレベル（.codex/, .github/）
    Project,
}

impl Scope {
    /// 識別子文字列を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            Scope::Personal => "personal",
            Scope::Project => "project",
        }
    }

    /// 表示名を取得
    pub fn display_name(&self) -> &'static str {
        match self {
            Scope::Personal => "Personal",
            Scope::Project => "Project",
        }
    }

    /// 説明文を取得
    pub fn description(&self) -> &'static str {
        match self {
            Scope::Personal => "~/.codex/, ~/.copilot/",
            Scope::Project => ".codex/, .github/",
        }
    }
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// コンポーネントの配置情報
#[derive(Debug, Clone)]
pub struct ComponentPlacement {
    pub kind: ComponentKind,
    pub name: String,
    pub scope: Scope,
    source_path: PathBuf,
    target_path: PathBuf,
}

impl ComponentPlacement {
    /// Builderを生成
    pub fn builder() -> ComponentPlacementBuilder {
        ComponentPlacementBuilder::new()
    }

    /// 配置先パスを取得
    pub fn path(&self) -> &Path {
        &self.target_path
    }

    /// ソースパスを取得
    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    /// 配置を実行（ファイルコピー）
    pub fn execute(&self) -> Result<()> {
        match self.kind {
            ComponentKind::Skill => {
                // Skills are directories
                copy_directory(&self.source_path, &self.target_path)?;
            }
            ComponentKind::Agent | ComponentKind::Prompt | ComponentKind::Instruction => {
                // These are files
                copy_file(&self.source_path, &self.target_path)?;
            }
        }
        Ok(())
    }
}

/// ComponentPlacement のビルダー
#[derive(Debug, Default)]
pub struct ComponentPlacementBuilder {
    kind: Option<ComponentKind>,
    name: Option<String>,
    scope: Option<Scope>,
    source_path: Option<PathBuf>,
    target_path: Option<PathBuf>,
}

impl ComponentPlacementBuilder {
    /// 新しいビルダーを生成
    pub fn new() -> Self {
        Self::default()
    }

    /// Component から kind, name, source_path を設定
    pub fn component(mut self, component: &Component) -> Self {
        self.kind = Some(component.kind);
        self.name = Some(component.name.clone());
        self.source_path = Some(component.path.clone());
        self
    }

    /// コンポーネント種別を設定
    pub fn kind(mut self, kind: ComponentKind) -> Self {
        self.kind = Some(kind);
        self
    }

    /// コンポーネント名を設定
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// スコープを設定
    pub fn scope(mut self, scope: Scope) -> Self {
        self.scope = Some(scope);
        self
    }

    /// ソースパスを設定
    pub fn source_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.source_path = Some(path.into());
        self
    }

    /// ターゲットパスを設定
    pub fn target_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.target_path = Some(path.into());
        self
    }

    /// ComponentPlacement を構築
    pub fn build(self) -> Result<ComponentPlacement> {
        let kind = self
            .kind
            .ok_or_else(|| PlmError::Validation("kind is required".to_string()))?;
        let name = self
            .name
            .ok_or_else(|| PlmError::Validation("name is required".to_string()))?;
        let scope = self
            .scope
            .ok_or_else(|| PlmError::Validation("scope is required".to_string()))?;
        let source_path = self
            .source_path
            .ok_or_else(|| PlmError::Validation("source_path is required".to_string()))?;
        let target_path = self
            .target_path
            .ok_or_else(|| PlmError::Validation("target_path is required".to_string()))?;

        Ok(ComponentPlacement {
            kind,
            name,
            scope,
            source_path,
            target_path,
        })
    }
}

/// ディレクトリを再帰的にコピー
fn copy_directory(source: &Path, target: &Path) -> Result<()> {
    if target.exists() {
        fs::remove_dir_all(target)?;
    }
    fs::create_dir_all(target)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());

        if source_path.is_dir() {
            copy_directory(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path)?;
        }
    }

    Ok(())
}

/// ファイルをコピー（親ディレクトリも作成）
fn copy_file(source: &Path, target: &Path) -> Result<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, target)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_kind_as_str() {
        assert_eq!(ComponentKind::Skill.as_str(), "skill");
        assert_eq!(ComponentKind::Agent.as_str(), "agent");
        assert_eq!(ComponentKind::Prompt.as_str(), "prompt");
        assert_eq!(ComponentKind::Instruction.as_str(), "instruction");
    }

    #[test]
    fn test_component_kind_plural() {
        assert_eq!(ComponentKind::Skill.plural(), "skills");
        assert_eq!(ComponentKind::Agent.plural(), "agents");
    }

    #[test]
    fn test_component_kind_all() {
        assert_eq!(ComponentKind::all().len(), 4);
    }

    #[test]
    fn test_scope_as_str() {
        assert_eq!(Scope::Personal.as_str(), "personal");
        assert_eq!(Scope::Project.as_str(), "project");
    }
}
