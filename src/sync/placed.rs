//! 配置済みコンポーネントの定義

use crate::component::{ComponentKind, Scope};
use crate::error::{PlmError, Result};
use std::path::{Path, PathBuf};

/// コンポーネントを一意に識別するキー
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComponentIdentity {
    pub kind: ComponentKind,
    pub name: String,
    pub scope: Scope,
}

impl ComponentIdentity {
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
    pub identity: ComponentIdentity,
    pub path: PathBuf,
}

impl PlacedComponent {
    pub fn new(
        kind: ComponentKind,
        name: impl Into<String>,
        scope: Scope,
        path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            identity: ComponentIdentity::new(kind, name, scope),
            path: path.into(),
        }
    }

    /// 識別子を取得
    pub fn identity(&self) -> &ComponentIdentity {
        &self.identity
    }

    /// kind を取得
    pub fn kind(&self) -> ComponentKind {
        self.identity.kind
    }

    /// name を取得
    pub fn name(&self) -> &str {
        &self.identity.name
    }

    /// scope を取得
    pub fn scope(&self) -> Scope {
        self.identity.scope
    }

    /// パスが project_root 配下かを検証
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
