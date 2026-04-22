//! 配置(Placement)ドメイン
//!
//! コンポーネントの配置先決定に関するドメインモデルを定義する。

use crate::component::{ComponentIdentity, ComponentKind, Scope};
use crate::target::PluginOrigin;
use std::path::{Path, PathBuf};

/// 配置スコープ
///
/// `Scope` をラップして配置ドメイン固有の型として扱う。
#[derive(Debug, Clone, Copy)]
pub struct PlacementScope(Scope);

impl PlacementScope {
    pub fn new(scope: Scope) -> Self {
        Self(scope)
    }

    pub fn scope(&self) -> Scope {
        self.0
    }
}

/// プロジェクトコンテキスト
///
/// プロジェクトルートなどの環境依存情報を保持する。
#[derive(Debug, Clone)]
pub struct ProjectContext<'a> {
    pub project_root: &'a Path,
}

impl<'a> ProjectContext<'a> {
    /// Create a new `ProjectContext`.
    ///
    /// # Arguments
    ///
    /// * `project_root` - Absolute or canonicalized path to the project root directory.
    pub fn new(project_root: &'a Path) -> Self {
        Self { project_root }
    }
}

/// 配置コンテキスト
///
/// 配置先決定に必要な全情報を集約する。
#[derive(Debug, Clone)]
pub struct PlacementContext<'a> {
    pub component: ComponentIdentity,
    pub origin: &'a PluginOrigin,
    pub scope: PlacementScope,
    pub project: ProjectContext<'a>,
}

impl<'a> PlacementContext<'a> {
    /// コンポーネント種別を取得
    pub fn kind(&self) -> ComponentKind {
        self.component.kind
    }

    /// コンポーネント名を取得
    pub fn name(&self) -> &str {
        &self.component.name
    }

    /// スコープを取得
    pub fn scope(&self) -> Scope {
        self.scope.scope()
    }

    /// プロジェクトルートを取得
    pub fn project_root(&self) -> &Path {
        self.project.project_root
    }
}

/// 配置先ロケーション
///
/// ファイルまたはディレクトリの配置先を表す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlacementLocation {
    /// ファイルへの配置
    File(PathBuf),
    /// ディレクトリへの配置
    Dir(PathBuf),
}

impl PlacementLocation {
    /// ファイル配置を作成
    ///
    /// # Arguments
    ///
    /// * `path` - Target file path.
    pub fn file(path: impl Into<PathBuf>) -> Self {
        Self::File(path.into())
    }

    /// ディレクトリ配置を作成
    ///
    /// # Arguments
    ///
    /// * `path` - Target directory path.
    pub fn dir(path: impl Into<PathBuf>) -> Self {
        Self::Dir(path.into())
    }

    /// パスを取得
    pub fn as_path(&self) -> &Path {
        match self {
            PlacementLocation::File(path) | PlacementLocation::Dir(path) => path,
        }
    }

    /// PathBuf に変換
    pub fn into_path(self) -> PathBuf {
        match self {
            PlacementLocation::File(path) | PlacementLocation::Dir(path) => path,
        }
    }

    /// ファイルかどうか
    pub fn is_file(&self) -> bool {
        matches!(self, PlacementLocation::File(_))
    }

    /// ディレクトリかどうか
    pub fn is_dir(&self) -> bool {
        matches!(self, PlacementLocation::Dir(_))
    }
}

#[cfg(test)]
#[path = "placement_test.rs"]
mod tests;
