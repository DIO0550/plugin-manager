//! コンポーネントのデプロイ処理
//!
//! `ComponentDeployment` 構造体本体と基本アクセサを定義する。
//! デプロイ実行ロジック (`execute()` / `deploy_*`) は `executor` サブモジュールへ分離。

mod bash;
mod builder;
mod conversion;
mod executor;
mod hook_deploy;
mod output;

use crate::component::{Component, ComponentKind, Scope};
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
}
