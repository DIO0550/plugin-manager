//! `ComponentDeployment::execute()` のディスパッチと種別ごとの `deploy_*` 実装

use super::conversion::ConversionConfig;
use super::output::DeploymentOutput;
use super::ComponentDeployment;
use crate::component::convert;
use crate::component::ComponentKind;
use crate::error::Result;
use crate::path_ext::PathExt;

impl ComponentDeployment {
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
#[path = "executor_test.rs"]
mod tests;
