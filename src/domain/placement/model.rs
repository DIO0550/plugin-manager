//! 配置ドメインの値オブジェクト
//!
//! 配置先決定に必要な情報をモデル化する。

use crate::component::{ComponentKind, Scope};
use crate::target::PluginOrigin;
use std::path::{Path, PathBuf};

/// コンポーネント参照
///
/// コンポーネントの種別と名前を保持する値オブジェクト。
#[derive(Debug, Clone)]
pub struct ComponentRef {
    pub kind: ComponentKind,
    pub name: String,
}

impl ComponentRef {
    pub fn new(kind: ComponentKind, name: impl Into<String>) -> Self {
        Self {
            kind,
            name: name.into(),
        }
    }
}

/// 配置スコープ
///
/// `Scope` をラップして配置ドメイン固有の型として扱う。
#[derive(Debug, Clone, Copy)]
pub struct PlacementScope(pub Scope);

impl PlacementScope {
    pub fn personal() -> Self {
        Self(Scope::Personal)
    }

    pub fn project() -> Self {
        Self(Scope::Project)
    }

    pub fn inner(&self) -> Scope {
        self.0
    }
}

impl From<Scope> for PlacementScope {
    fn from(scope: Scope) -> Self {
        Self(scope)
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

    /// コンポーネント名を取得
    pub fn name(&self) -> &str {
        &self.component.name
    }

    /// スコープを取得
    pub fn scope(&self) -> Scope {
        self.scope.0
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
    pub fn file(path: impl Into<PathBuf>) -> Self {
        Self::File(path.into())
    }

    /// ディレクトリ配置を作成
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
mod tests {
    use super::*;

    #[test]
    fn test_component_ref() {
        let component = ComponentRef::new(ComponentKind::Skill, "my-skill");
        assert_eq!(component.kind, ComponentKind::Skill);
        assert_eq!(component.name, "my-skill");
    }

    #[test]
    fn test_placement_scope() {
        let personal = PlacementScope::personal();
        assert_eq!(personal.inner(), Scope::Personal);

        let project = PlacementScope::project();
        assert_eq!(project.inner(), Scope::Project);

        let from_scope = PlacementScope::from(Scope::Personal);
        assert_eq!(from_scope.inner(), Scope::Personal);
    }

    #[test]
    fn test_placement_location() {
        let file = PlacementLocation::file("/path/to/file.md");
        assert!(file.is_file());
        assert!(!file.is_dir());
        assert_eq!(file.as_path(), Path::new("/path/to/file.md"));

        let dir = PlacementLocation::dir("/path/to/dir");
        assert!(dir.is_dir());
        assert!(!dir.is_file());
        assert_eq!(dir.as_path(), Path::new("/path/to/dir"));
    }

    #[test]
    fn test_placement_context() {
        let origin = PluginOrigin::from_marketplace("official", "my-plugin");
        let project_root = Path::new("/project");

        let ctx = PlacementContext {
            component: ComponentRef::new(ComponentKind::Skill, "my-skill"),
            origin: &origin,
            scope: PlacementScope(Scope::Project),
            project: ProjectContext::new(project_root),
        };

        assert_eq!(ctx.kind(), ComponentKind::Skill);
        assert_eq!(ctx.name(), "my-skill");
        assert_eq!(ctx.scope(), Scope::Project);
        assert_eq!(ctx.project_root(), project_root);
    }
}
