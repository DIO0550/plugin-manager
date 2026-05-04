//! 同期元の定義

use std::path::{Path, PathBuf};

use super::super::model::{PlacedComponent, SyncOptions};
use super::TargetBinding;
use crate::component::CommandFormat;
use crate::error::Result;
use crate::target::{Target, TargetKind};

/// 同期元
pub struct SyncSource(TargetBinding);

impl std::fmt::Debug for SyncSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncSource")
            .field("target", &self.0.target.name())
            .field("project_root", &self.0.project_root)
            .finish()
    }
}

impl SyncSource {
    /// 本番用コンストラクタ
    ///
    /// # Arguments
    ///
    /// * `kind` - Target environment kind to read components from.
    /// * `project_root` - Project root directory used when resolving placement paths.
    pub fn new(kind: TargetKind, project_root: &Path) -> Result<Self> {
        Ok(Self(TargetBinding::new(kind, project_root)?))
    }

    /// テスト用コンストラクタ（Target を注入）
    ///
    /// # Arguments
    ///
    /// * `target` - Injected target implementation to use in tests.
    /// * `project_root` - Project root directory used when resolving placement paths.
    pub fn with_target(target: Box<dyn Target>, project_root: &Path) -> Self {
        Self(TargetBinding::with_target(target, project_root))
    }

    /// ターゲット名を取得
    pub fn name(&self) -> &'static str {
        self.0.name()
    }

    /// Command フォーマットを取得
    pub fn command_format(&self) -> CommandFormat {
        self.0.command_format()
    }

    /// 配置済みコンポーネントを取得
    ///
    /// 重複した PlacedRef がある場合はエラー
    ///
    /// # Arguments
    ///
    /// * `options` - Options selecting which kinds and scopes to include.
    pub fn placed_components(&self, options: &SyncOptions) -> Result<Vec<PlacedComponent>> {
        self.0.placed_components(options)
    }

    /// コンポーネントのパスを取得
    ///
    /// # Arguments
    ///
    /// * `component` - Placed component whose source path should be resolved.
    pub fn path_for(&self, component: &PlacedComponent) -> Result<PathBuf> {
        self.0.path_for(component)
    }
}

#[cfg(test)]
#[path = "source_test.rs"]
mod tests;
