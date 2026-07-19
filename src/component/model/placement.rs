//! 配置(Placement)ドメイン
//!
//! コンポーネントの配置先決定に関するドメインモデルを定義する。

use crate::component::{ComponentKind, Scope};
use crate::target::PluginOrigin;
use std::path::{Path, PathBuf};

/// コンポーネント参照
///
/// 配置先決定に必要な最小の識別子。
/// `name` はフラット化済み識別子、`original_name` はスキャン時の元名。
/// scope は `PlacementContext.scope` 側に保持する。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComponentRef {
    pub kind: ComponentKind,
    pub name: String,
    pub original_name: String,
    pub plugin_name: String,
}

impl ComponentRef {
    /// `original_name = name`、`plugin_name` 空で構築する（後方互換ヘルパー）。
    ///
    /// # Arguments
    ///
    /// * `kind` - Component kind.
    /// * `name` - Component identifier used as both `name` and `original_name`.
    pub fn new(kind: ComponentKind, name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            kind,
            original_name: name.clone(),
            plugin_name: String::new(),
            name,
        }
    }

    /// フラット化情報付きで構築する。
    ///
    /// # Arguments
    ///
    /// * `kind` - Component kind.
    /// * `name` - Flattened identifier (`{plugin}_{original}`).
    /// * `original_name` - Pre-flatten name.
    /// * `plugin_name` - Plugin manifest name.
    pub fn with_names(
        kind: ComponentKind,
        name: impl Into<String>,
        original_name: impl Into<String>,
        plugin_name: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            name: name.into(),
            original_name: original_name.into(),
            plugin_name: plugin_name.into(),
        }
    }
}

impl From<&crate::component::Component> for ComponentRef {
    fn from(c: &crate::component::Component) -> Self {
        Self::with_names(
            c.kind,
            c.name.clone(),
            c.original_name.clone(),
            c.plugin_name.clone(),
        )
    }
}

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
    pub component: ComponentRef,
    pub origin: &'a PluginOrigin,
    pub scope: PlacementScope,
    pub project: ProjectContext<'a>,
}

impl<'a> PlacementContext<'a> {
    /// コンポーネント種別を取得
    pub fn kind(&self) -> ComponentKind {
        self.component.kind
    }

    /// フラット化済みコンポーネント名を取得
    pub fn name(&self) -> &str {
        &self.component.name
    }

    /// スキャン時の元名を取得（空なら `name` にフォールバック）
    pub fn original_name(&self) -> &str {
        if self.component.original_name.is_empty() {
            &self.component.name
        } else {
            &self.component.original_name
        }
    }

    /// プラグイン名を取得
    pub fn plugin_name(&self) -> &str {
        &self.component.plugin_name
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
