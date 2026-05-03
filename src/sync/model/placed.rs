//! 配置済みコンポーネントの定義

use crate::component::{ComponentKind, Scope};
use crate::error::{PlmError, Result};
use std::path::{Path, PathBuf};

/// 配置済みコンポーネントの識別キー（`kind` + `name` + `scope`）。
///
/// sync で source / dest のマッチングに使う HashMap キー用途に特化した型。
/// 同一 target 内では (kind, name, scope) の組で一意。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlacedRef {
    pub kind: ComponentKind,
    pub name: String,
    pub scope: Scope,
}

impl PlacedRef {
    pub fn new(kind: ComponentKind, name: impl Into<String>, scope: Scope) -> Self {
        Self {
            kind,
            name: name.into(),
            scope,
        }
    }
}

/// ターゲット上に配置されているコンポーネント
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacedComponent {
    pub placed_ref: PlacedRef,
    pub path: PathBuf,
}

impl PlacedComponent {
    /// Create a new `PlacedComponent`.
    ///
    /// # Arguments
    ///
    /// * `kind` - Component kind.
    /// * `name` - Fully-qualified component name.
    /// * `scope` - Placement scope of the component.
    /// * `path` - File-system path where the component is placed.
    pub fn new(
        kind: ComponentKind,
        name: impl Into<String>,
        scope: Scope,
        path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            placed_ref: PlacedRef::new(kind, name, scope),
            path: path.into(),
        }
    }

    /// 識別キーを取得
    pub fn placed_ref(&self) -> &PlacedRef {
        &self.placed_ref
    }

    /// kind を取得
    pub fn kind(&self) -> ComponentKind {
        self.placed_ref.kind
    }

    /// name を取得
    pub fn name(&self) -> &str {
        &self.placed_ref.name
    }

    /// scope を取得
    pub fn scope(&self) -> Scope {
        self.placed_ref.scope
    }

    /// パスが project_root 配下かを検証
    ///
    /// # Arguments
    ///
    /// * `project_root` - Project root directory that the path must be contained within.
    pub fn validate_path(&self, project_root: &Path) -> Result<()> {
        // パスが存在しない場合は検証をスキップ（これから作成される場合）
        if !self.path.exists() {
            return Ok(());
        }

        let canonical = self.path.canonicalize().map_err(|e| {
            PlmError::InvalidArgument(format!(
                "Failed to canonicalize path {:?}: {}",
                self.path, e
            ))
        })?;

        let root_canonical = project_root.canonicalize().map_err(|e| {
            PlmError::InvalidArgument(format!(
                "Failed to canonicalize project root {:?}: {}",
                project_root, e
            ))
        })?;

        if !canonical.starts_with(&root_canonical) {
            return Err(PlmError::InvalidArgument(format!(
                "Path escapes project root: {:?}",
                self.path
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "placed_test.rs"]
mod tests;
