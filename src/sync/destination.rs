//! 同期先の定義

use super::options::{SyncOptions, SyncableKind};
use super::placed::{ComponentIdentity, PlacedComponent};
use super::source::parse_component_name;
use crate::component::{ComponentKind, Scope};
use crate::domain::{ComponentRef, PlacementContext, PlacementScope, ProjectContext};
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
    pub fn new(kind: TargetKind, project_root: &Path) -> Result<Self> {
        let target = parse_target(kind.as_str())?;
        Ok(Self {
            target,
            project_root: project_root.to_path_buf(),
        })
    }

    /// テスト用コンストラクタ（Target を注入）
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

    /// 配置済みコンポーネントを取得
    ///
    /// 重複 identity がある場合はエラー
    pub fn placed_components(&self, options: &SyncOptions) -> Result<Vec<PlacedComponent>> {
        let mut components = Vec::new();
        let mut seen_identities = HashSet::new();

        let kinds = self.target_kinds(options);
        let scopes = self.target_scopes(options);

        for syncable_kind in kinds {
            let kind = syncable_kind.to_component_kind();

            for scope in &scopes {
                let placed = self.target.list_placed(kind, *scope, &self.project_root)?;

                for name in placed {
                    let identity = ComponentIdentity::new(kind, name.clone(), *scope);

                    // 重複チェック
                    if !seen_identities.insert(identity.clone()) {
                        return Err(PlmError::InvalidArgument(format!(
                            "Duplicate component identity: {:?}",
                            identity
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
    pub fn path_for(&self, component: &PlacedComponent) -> Result<PathBuf> {
        self.resolve_path(component.kind(), component.name(), component.scope())
    }

    /// このコンポーネントをサポートしているか
    pub fn supports(&self, identity: &ComponentIdentity) -> bool {
        self.target.supports(identity.kind) && self.target.supports_scope(identity.kind, identity.scope)
    }

    /// 対象の SyncableKind リストを取得
    fn target_kinds(&self, options: &SyncOptions) -> Vec<SyncableKind> {
        match options.component_type {
            Some(kind) => vec![kind],
            None => SyncableKind::all().to_vec(),
        }
    }

    /// 対象の Scope リストを取得
    fn target_scopes(&self, options: &SyncOptions) -> Vec<Scope> {
        match options.scope {
            Some(scope) => vec![scope],
            None => vec![Scope::Personal, Scope::Project],
        }
    }

    /// パスを解決
    fn resolve_path(&self, kind: ComponentKind, name: &str, scope: Scope) -> Result<PathBuf> {
        let (origin, component_name) = parse_component_name(name)?;

        let ctx = PlacementContext {
            component: ComponentRef::new(kind, component_name),
            origin: &origin,
            scope: PlacementScope(scope),
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
mod tests {
    use super::*;

    #[test]
    fn test_destination_new() {
        let dest = SyncDestination::new(TargetKind::Copilot, Path::new("."));
        assert!(dest.is_ok());
        assert_eq!(dest.unwrap().name(), "copilot");
    }
}
