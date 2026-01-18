//! コンポーネントのデプロイ処理

use crate::component::{Component, ComponentKind, Scope};
use crate::error::{PlmError, Result};
use crate::path_ext::PathExt;
use std::path::{Path, PathBuf};

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
                self.source_path.copy_dir_to(&self.target_path)?;
            }
            ComponentKind::Agent
            | ComponentKind::Command
            | ComponentKind::Instruction
            | ComponentKind::Hook => {
                // These are files
                self.source_path.copy_file_to(&self.target_path)?;
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

#[cfg(test)]
#[path = "deployment_test.rs"]
mod tests;
