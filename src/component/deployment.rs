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
use crate::fs::{FileSystem, RealFs};
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
        self.execute_with_fs(&RealFs)
    }

    /// テスト用エントリポイント（`FileSystem` を注入）
    pub fn execute_with_fs(&self, fs: &dyn FileSystem) -> Result<DeploymentOutput> {
        match self.kind() {
            ComponentKind::Skill => self.deploy_skill(fs),
            ComponentKind::Command => self.deploy_command(fs),
            ComponentKind::Agent => self.deploy_agent(fs),
            ComponentKind::Instruction => self.deploy_instruction(fs),
            ComponentKind::Hook => self.deploy_hook(fs),
        }
    }

    fn deploy_skill(&self, fs: &dyn FileSystem) -> Result<DeploymentOutput> {
        // Skills are directories — replace target to avoid stale files.
        fs.replace_dir(self.source_path(), &self.target_path)?;

        // ターゲットがサポートしない frontmatter フィールドを SKILL.md から除去する。
        if let ConversionConfig::Skill { target_kind } = &self.conversion {
            if let Some(allowed) = convert::skill_allowed_fields(*target_kind) {
                let manifest = self.target_path.join("SKILL.md");
                if fs.exists(&manifest) && !fs.is_dir(&manifest) {
                    let original = fs.read_to_string(&manifest)?;
                    let stripped = convert::strip_skill_frontmatter_fields(&original, allowed);
                    if stripped != original {
                        // 部分書き込みでデプロイ済み Skill を壊さないよう、
                        // 他の変換と同様にアトミック（tmp → rename）に書き戻す。
                        convert::atomic_write(&manifest, &stripped)?;
                    }
                }
            }
        }

        Ok(DeploymentOutput::Copied)
    }

    fn deploy_command(&self, fs: &dyn FileSystem) -> Result<DeploymentOutput> {
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
                fs.copy_file(self.source_path(), &self.target_path)?;
                Ok(DeploymentOutput::Copied)
            }
        }
    }

    fn deploy_agent(&self, fs: &dyn FileSystem) -> Result<DeploymentOutput> {
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
                fs.copy_file(self.source_path(), &self.target_path)?;
                Ok(DeploymentOutput::Copied)
            }
        }
    }

    fn deploy_instruction(&self, fs: &dyn FileSystem) -> Result<DeploymentOutput> {
        fs.copy_file(self.source_path(), &self.target_path)?;
        Ok(DeploymentOutput::Copied)
    }

    fn deploy_hook(&self, fs: &dyn FileSystem) -> Result<DeploymentOutput> {
        match &self.conversion {
            ConversionConfig::Hook {
                target_kind,
                plugin_root,
            } => self.deploy_hook_converted(fs, *target_kind, plugin_root.as_deref()),
            _ => {
                fs.copy_file(self.source_path(), &self.target_path)?;
                Ok(DeploymentOutput::Copied)
            }
        }
    }
}

#[cfg(test)]
#[path = "deployment_test.rs"]
mod tests;
