//! コンポーネントのデプロイ処理

mod bash_escape;
mod executor;
mod hook_deploy;
mod output;

pub use executor::ComponentDeployment;
pub use output::DeploymentOutput;

use super::convert::{AgentFormat, CommandFormat};
use crate::component::{Component, ComponentKind, Scope};
use crate::error::{PlmError, Result};
use crate::target::TargetKind;
use std::path::PathBuf;

/// ComponentDeployment のビルダー
#[derive(Debug, Default)]
pub(crate) struct ComponentDeploymentBuilder {
    kind: Option<ComponentKind>,
    name: Option<String>,
    scope: Option<Scope>,
    source_path: Option<PathBuf>,
    target_path: Option<PathBuf>,
    source_format: Option<CommandFormat>,
    dest_format: Option<CommandFormat>,
    source_agent_format: Option<AgentFormat>,
    dest_agent_format: Option<AgentFormat>,
    hook_convert: Option<bool>,
    target_kind: Option<TargetKind>,
    plugin_root: Option<PathBuf>,
}

impl ComponentDeploymentBuilder {
    /// 新しいビルダーを生成
    pub fn new() -> Self {
        Self::default()
    }

    /// Component から kind, name, source_path を設定
    ///
    /// # Arguments
    ///
    /// * `component` - Component whose `kind`, `name`, and `path` are copied into the builder.
    pub fn component(mut self, component: &Component) -> Self {
        self.kind = Some(component.kind);
        self.name = Some(component.name.clone());
        self.source_path = Some(component.path.clone());
        self
    }

    /// コンポーネント種別を設定
    ///
    /// # Arguments
    ///
    /// * `kind` - Component kind (`Skill`, `Agent`, `Command`, `Instruction`, `Hook`).
    pub fn kind(mut self, kind: ComponentKind) -> Self {
        self.kind = Some(kind);
        self
    }

    /// コンポーネント名を設定
    ///
    /// # Arguments
    ///
    /// * `name` - Component name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// スコープを設定
    ///
    /// # Arguments
    ///
    /// * `scope` - Deployment scope (`Personal` or `Project`).
    pub fn scope(mut self, scope: Scope) -> Self {
        self.scope = Some(scope);
        self
    }

    /// ターゲットパスを設定
    ///
    /// # Arguments
    ///
    /// * `path` - Target path to deploy the component to.
    pub fn target_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.target_path = Some(path.into());
        self
    }

    /// ソースの Command フォーマットを設定
    ///
    /// # Arguments
    ///
    /// * `format` - Source command format.
    pub fn source_format(mut self, format: CommandFormat) -> Self {
        self.source_format = Some(format);
        self
    }

    /// ターゲットの Command フォーマットを設定
    ///
    /// # Arguments
    ///
    /// * `format` - Destination command format.
    pub fn dest_format(mut self, format: CommandFormat) -> Self {
        self.dest_format = Some(format);
        self
    }

    /// ソースの Agent フォーマットを設定
    ///
    /// # Arguments
    ///
    /// * `format` - Source agent format.
    pub fn source_agent_format(mut self, format: AgentFormat) -> Self {
        self.source_agent_format = Some(format);
        self
    }

    /// ターゲットの Agent フォーマットを設定
    ///
    /// # Arguments
    ///
    /// * `format` - Destination agent format.
    pub fn dest_agent_format(mut self, format: AgentFormat) -> Self {
        self.dest_agent_format = Some(format);
        self
    }

    /// Hook 変換を有効化
    ///
    /// # Arguments
    ///
    /// * `convert` - Whether to run the hook converter during deployment.
    pub fn hook_convert(mut self, convert: bool) -> Self {
        self.hook_convert = Some(convert);
        self
    }

    /// Hook 変換のターゲット種別を設定
    ///
    /// # Arguments
    ///
    /// * `kind` - Target kind that drives hook conversion rules.
    pub fn target_kind(mut self, kind: TargetKind) -> Self {
        self.target_kind = Some(kind);
        self
    }

    /// プラグインルートパスを設定（@@PLUGIN_ROOT@@ 置換用）
    ///
    /// # Arguments
    ///
    /// * `path` - Plugin cache root substituted for the `@@PLUGIN_ROOT@@` placeholder in scripts.
    pub fn plugin_root(mut self, path: impl Into<PathBuf>) -> Self {
        self.plugin_root = Some(path.into());
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

        let hook_convert = self.hook_convert.unwrap_or(false);

        if hook_convert && self.target_kind.is_none() {
            return Err(PlmError::Validation(
                "target_kind is required when hook_convert is enabled".to_string(),
            ));
        }

        Ok(ComponentDeployment {
            kind,
            name,
            scope,
            source_path,
            target_path,
            source_format: self.source_format,
            dest_format: self.dest_format,
            source_agent_format: self.source_agent_format,
            dest_agent_format: self.dest_agent_format,
            hook_convert,
            target_kind: self.target_kind,
            plugin_root: self.plugin_root,
        })
    }
}

#[cfg(test)]
#[path = "deployment_test.rs"]
mod tests;
