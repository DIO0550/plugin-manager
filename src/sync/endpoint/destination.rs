//! 同期先の定義

use super::super::model::{PlacedComponent, PlacedRef, SyncOptions, SyncableKind};
use super::source::parse_component_name;
use crate::component::{
    CommandFormat, ComponentKind, ComponentRef, PlacementContext, PlacementScope, ProjectContext,
    Scope,
};
use crate::error::{PlmError, Result};
use crate::target::{parse_target, Target, TargetKind};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// 同期先
pub struct SyncDestination {
    target: Box<dyn Target>,
    project_root: PathBuf,
}

impl std::fmt::Debug for SyncDestination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncDestination")
            .field("target", &self.target.name())
            .field("project_root", &self.project_root)
            .finish()
    }
}

impl SyncDestination {
    /// 本番用コンストラクタ
    ///
    /// # Arguments
    ///
    /// * `kind` - Target environment kind to synchronize into.
    /// * `project_root` - Project root directory used when resolving placement paths.
    pub fn new(kind: TargetKind, project_root: &Path) -> Result<Self> {
        let target = parse_target(kind.as_str())?;
        Ok(Self {
            target,
            project_root: project_root.to_path_buf(),
        })
    }

    /// テスト用コンストラクタ（Target を注入）
    ///
    /// # Arguments
    ///
    /// * `target` - Injected target implementation to use in tests.
    /// * `project_root` - Project root directory used when resolving placement paths.
    pub fn with_target(target: Box<dyn Target>, project_root: &Path) -> Self {
        Self {
            target,
            project_root: project_root.to_path_buf(),
        }
    }

    /// ターゲット名を取得
    pub fn name(&self) -> &'static str {
        self.target.name()
    }

    /// Command フォーマットを取得
    pub fn command_format(&self) -> CommandFormat {
        self.target.command_format()
    }

    /// 配置済みコンポーネントを取得
    ///
    /// 重複した PlacedRef がある場合はエラー
    ///
    /// # Arguments
    ///
    /// * `options` - Options selecting which kinds and scopes to include.
    pub fn placed_components(&self, options: &SyncOptions) -> Result<Vec<PlacedComponent>> {
        let mut components = Vec::new();
        let mut seen_refs = HashSet::new();

        let kinds = self.target_kinds(options);
        let scopes = self.target_scopes(options);

        for syncable_kind in kinds {
            let kind = syncable_kind.to_component_kind();

            for scope in &scopes {
                let placed = self.target.list_placed(kind, *scope, &self.project_root)?;

                for name in placed {
                    let placed_ref = PlacedRef::new(kind, name.clone(), *scope);

                    if !seen_refs.insert(placed_ref.clone()) {
                        return Err(PlmError::InvalidArgument(format!(
                            "Duplicate placed component ref: {:?}",
                            placed_ref
                        )));
                    }

                    let path = self.resolve_path(kind, &name, *scope)?;
                    components.push(PlacedComponent::new(kind, name, *scope, path));
                }
            }
        }

        Ok(components)
    }

    /// コンポーネントの配置先パスを取得
    ///
    /// # Arguments
    ///
    /// * `component` - Placed component whose destination path should be resolved.
    pub fn path_for(&self, component: &PlacedComponent) -> Result<PathBuf> {
        self.resolve_path(component.kind(), component.name(), component.scope())
    }

    /// このコンポーネントをサポートしているか
    ///
    /// # Arguments
    ///
    /// * `placed_ref` - Placed component reference whose kind and scope support is checked.
    pub fn supports(&self, placed_ref: &PlacedRef) -> bool {
        self.target.supports(placed_ref.kind)
            && self
                .target
                .supports_scope(placed_ref.kind, placed_ref.scope)
    }

    /// 対象の SyncableKind リストを取得
    ///
    /// # Arguments
    ///
    /// * `options` - Options whose `component_type` filter is applied.
    fn target_kinds(&self, options: &SyncOptions) -> Vec<SyncableKind> {
        match options.component_type {
            Some(kind) => vec![kind],
            None => SyncableKind::all().to_vec(),
        }
    }

    /// 対象の Scope リストを取得
    ///
    /// # Arguments
    ///
    /// * `options` - Options whose `scope` filter is applied.
    fn target_scopes(&self, options: &SyncOptions) -> Vec<Scope> {
        match options.scope {
            Some(scope) => vec![scope],
            None => vec![Scope::Personal, Scope::Project],
        }
    }

    /// パスを解決
    ///
    /// # Arguments
    ///
    /// * `kind` - Component kind to resolve.
    /// * `name` - Fully-qualified component name (e.g. `marketplace/plugin/component`).
    /// * `scope` - Placement scope (personal or project).
    fn resolve_path(&self, kind: ComponentKind, name: &str, scope: Scope) -> Result<PathBuf> {
        let (origin, component_name) = parse_component_name(name)?;

        let ctx = PlacementContext {
            component: ComponentRef::new(kind, component_name),
            origin: &origin,
            scope: PlacementScope::new(scope),
            project: ProjectContext::new(&self.project_root),
        };

        self.target
            .placement_location(&ctx)
            .map(|l| l.into_path())
            .ok_or_else(|| {
                PlmError::InvalidArgument(format!(
                    "Cannot resolve path for {} on {}",
                    name,
                    self.target.name()
                ))
            })
    }
}

#[cfg(test)]
#[path = "destination_test.rs"]
mod tests;
