//! `ComponentDeployment` 構造体本体と `execute()` のディスパッチ

use super::output::DeploymentOutput;
use crate::component::convert::{self, AgentFormat, CommandFormat};
use crate::component::{ComponentKind, Scope};
use crate::error::Result;
use crate::path_ext::PathExt;
use crate::target::TargetKind;
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
    pub(super) source_path: PathBuf,
    pub(super) target_path: PathBuf,
    /// ソースの Command フォーマット（Command の場合のみ有効）
    pub(super) source_format: Option<CommandFormat>,
    /// ターゲットの Command フォーマット（Command の場合のみ有効）
    pub(super) dest_format: Option<CommandFormat>,
    /// ソースの Agent フォーマット（Agent の場合のみ有効）
    pub(super) source_agent_format: Option<AgentFormat>,
    /// ターゲットの Agent フォーマット（Agent の場合のみ有効）
    pub(super) dest_agent_format: Option<AgentFormat>,
    /// Hook 変換を実行するかどうか
    pub(super) hook_convert: bool,
    /// Hook 変換のターゲット種別
    pub(super) target_kind: Option<TargetKind>,
    /// @@PLUGIN_ROOT@@ 置換用のプラグインキャッシュルートパス
    pub(super) plugin_root: Option<PathBuf>,
}

impl ComponentDeployment {
    /// Builder を生成
    pub(crate) fn builder() -> super::ComponentDeploymentBuilder {
        super::ComponentDeploymentBuilder::new()
    }

    /// 配置先パスを取得
    pub fn path(&self) -> &Path {
        &self.target_path
    }

    /// 配置を実行
    ///
    /// `ComponentKind` ごとに専用の `deploy_*` メソッドへディスパッチする。
    pub fn execute(&self) -> Result<DeploymentOutput> {
        match self.kind {
            ComponentKind::Skill => self.deploy_skill(),
            ComponentKind::Command => self.deploy_command(),
            ComponentKind::Agent => self.deploy_agent(),
            ComponentKind::Instruction => self.deploy_instruction(),
            ComponentKind::Hook => self.deploy_hook(),
        }
    }

    fn deploy_skill(&self) -> Result<DeploymentOutput> {
        // Skills are directories
        self.source_path.copy_dir_to(&self.target_path)?;
        Ok(DeploymentOutput::Copied)
    }

    fn deploy_command(&self) -> Result<DeploymentOutput> {
        if let (Some(src_fmt), Some(dest_fmt)) = (self.source_format, self.dest_format) {
            let result = convert::convert_and_write(
                &self.source_path,
                &self.target_path,
                src_fmt,
                dest_fmt,
            )?;
            Ok(DeploymentOutput::CommandConverted(result))
        } else {
            self.source_path.copy_file_to(&self.target_path)?;
            Ok(DeploymentOutput::Copied)
        }
    }

    fn deploy_agent(&self) -> Result<DeploymentOutput> {
        if let (Some(src_fmt), Some(dest_fmt)) = (self.source_agent_format, self.dest_agent_format)
        {
            let result = convert::convert_agent_and_write(
                &self.source_path,
                &self.target_path,
                src_fmt,
                dest_fmt,
            )?;
            Ok(DeploymentOutput::AgentConverted(result))
        } else {
            self.source_path.copy_file_to(&self.target_path)?;
            Ok(DeploymentOutput::Copied)
        }
    }

    fn deploy_instruction(&self) -> Result<DeploymentOutput> {
        self.source_path.copy_file_to(&self.target_path)?;
        Ok(DeploymentOutput::Copied)
    }

    fn deploy_hook(&self) -> Result<DeploymentOutput> {
        if self.hook_convert {
            self.deploy_hook_converted()
        } else {
            self.source_path.copy_file_to(&self.target_path)?;
            Ok(DeploymentOutput::Copied)
        }
    }
}

#[cfg(test)]
#[path = "executor_test.rs"]
mod tests;
