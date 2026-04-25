//! コンポーネントのデプロイ処理
//!
//! `ComponentDeployment` 構造体本体と配置実行 (`execute()` / `deploy_*`) を定義する。
//! Hook 変換のような大きめの処理は `hook_deploy` サブモジュールへ分離。

mod bash;
mod builder;
mod conversion;
mod hook_deploy;
mod output;

use crate::component::convert;
use crate::component::{Component, ComponentKind, Scope};
use crate::error::Result;
use crate::path_ext::PathExt;
use std::path::{Path, PathBuf};

pub use builder::ComponentDeploymentBuilder;
pub use conversion::ConversionConfig;
pub use output::DeploymentOutput;

/// コンポーネントのデプロイ情報
///
/// 配置の実行（コピー/削除など）を担当する。
/// 配置先の決定は `PlacementLocation` が担当する。
#[derive(Debug, Clone)]
pub struct ComponentDeployment {
    pub(super) component: Component,
    pub scope: Scope,
    pub(super) target_path: PathBuf,
    pub(super) conversion: ConversionConfig,
}

impl ComponentDeployment {
    /// Builder を生成
    pub fn builder() -> ComponentDeploymentBuilder {
        ComponentDeploymentBuilder::default()
    }

    /// コンポーネント種別を取得
    pub fn kind(&self) -> ComponentKind {
        self.component.kind
    }

    /// コンポーネント名を取得
    pub fn name(&self) -> &str {
        &self.component.name
    }

    /// ソースパスを取得（同モジュール内のみ）
    pub(super) fn source_path(&self) -> &Path {
        &self.component.path
    }

    /// 配置先パスを取得
    pub fn path(&self) -> &Path {
        &self.target_path
    }

    /// 配置を実行
    ///
    /// `ComponentKind` ごとに専用の `deploy_*` メソッドへディスパッチする。
    pub fn execute(&self) -> Result<DeploymentOutput> {
        match self.kind() {
            ComponentKind::Skill => self.deploy_skill(),
            ComponentKind::Command => self.deploy_command(),
            ComponentKind::Agent => self.deploy_agent(),
            ComponentKind::Instruction => self.deploy_instruction(),
            ComponentKind::Hook => self.deploy_hook(),
        }
    }

    fn deploy_skill(&self) -> Result<DeploymentOutput> {
        // Skills are directories
        self.source_path().copy_dir_to(&self.target_path)?;
        Ok(DeploymentOutput::Copied)
    }

    fn deploy_command(&self) -> Result<DeploymentOutput> {
        match &self.conversion {
            ConversionConfig::Command { source, dest } => {
                let result = convert::convert_and_write(
                    self.source_path(),
                    &self.target_path,
                    *source,
                    *dest,
                )?;
                Ok(DeploymentOutput::CommandConverted(result))
            }
            _ => {
                self.source_path().copy_file_to(&self.target_path)?;
                Ok(DeploymentOutput::Copied)
            }
        }
    }

    fn deploy_agent(&self) -> Result<DeploymentOutput> {
        match &self.conversion {
            ConversionConfig::Agent { source, dest } => {
                let result = convert::convert_agent_and_write(
                    self.source_path(),
                    &self.target_path,
                    *source,
                    *dest,
                )?;
                Ok(DeploymentOutput::AgentConverted(result))
            }
            _ => {
                self.source_path().copy_file_to(&self.target_path)?;
                Ok(DeploymentOutput::Copied)
            }
        }
    }

    fn deploy_instruction(&self) -> Result<DeploymentOutput> {
        self.source_path().copy_file_to(&self.target_path)?;
        Ok(DeploymentOutput::Copied)
    }

    fn deploy_hook(&self) -> Result<DeploymentOutput> {
        match &self.conversion {
            ConversionConfig::Hook {
                target_kind,
                plugin_root,
            } => self.deploy_hook_converted(*target_kind, plugin_root.as_deref()),
            _ => {
                self.source_path().copy_file_to(&self.target_path)?;
                Ok(DeploymentOutput::Copied)
            }
        }
    }
}

#[cfg(test)]
#[path = "deployment/deployment_test.rs"]
mod tests;
