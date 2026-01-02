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
    /// コマンド（.prompt.md形式）
    Command,
    /// インストラクション（AGENTS.md, copilot-instructions.md形式）
    Instruction,
    /// フック（任意のスクリプト）
    Hook,
}

impl ComponentKind {
    /// 識別子文字列を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            ComponentKind::Skill => "skill",
            ComponentKind::Agent => "agent",
            ComponentKind::Command => "command",
            ComponentKind::Instruction => "instruction",
            ComponentKind::Hook => "hook",
        }
    }

    /// 複数形の文字列を取得
    pub fn plural(&self) -> &'static str {
        match self {
            ComponentKind::Skill => "skills",
            ComponentKind::Agent => "agents",
            ComponentKind::Command => "commands",
            ComponentKind::Instruction => "instructions",
            ComponentKind::Hook => "hooks",
        }
    }

    /// 表示名を取得
    pub fn display_name(&self) -> &'static str {
        match self {
            ComponentKind::Skill => "Skill",
            ComponentKind::Agent => "Agent",
            ComponentKind::Command => "Command",
            ComponentKind::Instruction => "Instruction",
            ComponentKind::Hook => "Hook",
        }
    }

    /// 全コンポーネント種別を取得
    pub fn all() -> &'static [ComponentKind] {
        &[
            ComponentKind::Skill,
            ComponentKind::Agent,
            ComponentKind::Command,
            ComponentKind::Instruction,
            ComponentKind::Hook,
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

/// コンポーネントのデプロイ情報
///
/// 配置の実行（コピー/削除など）を担当する。
/// 配置先の決定は `PlacementLocation` が担当する。
#[derive(Debug, Clone)]
pub struct ComponentDeployment {
    pub kind: ComponentKind,
    pub name: String,
    pub scope: Scope,
    source_path: PathBuf,
    target_path: PathBuf,
}

impl ComponentDeployment {
    /// Builderを生成
    pub fn builder() -> ComponentDeploymentBuilder {
        ComponentDeploymentBuilder::new()
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
            ComponentKind::Agent
            | ComponentKind::Command
            | ComponentKind::Instruction
            | ComponentKind::Hook => {
                // These are files
                copy_file(&self.source_path, &self.target_path)?;
            }
        }
        Ok(())
    }
}

/// ComponentDeployment のビルダー
#[derive(Debug, Default)]
pub struct ComponentDeploymentBuilder {
    kind: Option<ComponentKind>,
    name: Option<String>,
    scope: Option<Scope>,
    source_path: Option<PathBuf>,
    target_path: Option<PathBuf>,
}

impl ComponentDeploymentBuilder {
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

    /// ComponentDeployment を構築
    pub fn build(self) -> Result<ComponentDeployment> {
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

        Ok(ComponentDeployment {
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
        assert_eq!(ComponentKind::Command.as_str(), "command");
        assert_eq!(ComponentKind::Instruction.as_str(), "instruction");
        assert_eq!(ComponentKind::Hook.as_str(), "hook");
    }

    #[test]
    fn test_component_kind_plural() {
        assert_eq!(ComponentKind::Skill.plural(), "skills");
        assert_eq!(ComponentKind::Agent.plural(), "agents");
    }

    #[test]
    fn test_component_kind_all() {
        assert_eq!(ComponentKind::all().len(), 5);
    }

    #[test]
    fn test_scope_as_str() {
        assert_eq!(Scope::Personal.as_str(), "personal");
        assert_eq!(Scope::Project.as_str(), "project");
    }
}
