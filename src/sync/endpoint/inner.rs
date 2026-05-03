//! Endpoint の共通本体（共有 inner struct）
//!
//! `SyncSource` / `SyncDestination` の共通フィールド・共通メソッドを集約した
//! `pub(super)` 構造体。両 newtype が `EndpointInner` を内包し、共通メソッドを
//! 委譲する形で重複を解消する。

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::super::model::{PlacedComponent, PlacedRef, SyncOptions, SyncableKind};
use super::parse_component_name;
use crate::component::{
    CommandFormat, ComponentKind, ComponentRef, PlacementContext, PlacementScope, ProjectContext,
    Scope,
};
use crate::error::{PlmError, Result};
use crate::target::{parse_target, Target, TargetKind};

/// `SyncSource` / `SyncDestination` の共通フィールド・共通メソッドを保持する内部型
///
/// `pub(super)` により `endpoint` サブ親配下のみ可視。`endpoint.rs` 側では
/// 非公開の `use self::inner::EndpointInner;` でサブ親に再導入し、`source` /
/// `destination` 子モジュールから `super::EndpointInner` で参照できるようにする。
pub(super) struct EndpointInner {
    pub(super) target: Box<dyn Target>,
    pub(super) project_root: PathBuf,
}

impl std::fmt::Debug for EndpointInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EndpointInner")
            .field("target", &self.target.name())
            .field("project_root", &self.project_root)
            .finish()
    }
}

impl EndpointInner {
    /// 本番用コンストラクタ
    pub(super) fn new(kind: TargetKind, project_root: &Path) -> Result<Self> {
        let target = parse_target(kind.as_str())?;
        Ok(Self {
            target,
            project_root: project_root.to_path_buf(),
        })
    }

    /// テスト用コンストラクタ（Target を注入）
    pub(super) fn with_target(target: Box<dyn Target>, project_root: &Path) -> Self {
        Self {
            target,
            project_root: project_root.to_path_buf(),
        }
    }

    /// ターゲット名
    pub(super) fn name(&self) -> &'static str {
        self.target.name()
    }

    /// Command フォーマット
    pub(super) fn command_format(&self) -> CommandFormat {
        self.target.command_format()
    }

    /// Target trait オブジェクトへの参照
    ///
    /// `SyncDestination::supports` のような固有メソッドが
    /// `target.supports(kind)` / `target.supports_scope(kind, scope)` を
    /// 呼ぶために `pub(super)` で公開する。
    pub(super) fn target(&self) -> &dyn Target {
        self.target.as_ref()
    }

    /// 配置済みコンポーネント一覧
    ///
    /// 重複した `PlacedRef` がある場合はエラーを返す。
    pub(super) fn placed_components(&self, options: &SyncOptions) -> Result<Vec<PlacedComponent>> {
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

    /// 配置済みコンポーネントのパスを解決
    pub(super) fn path_for(&self, component: &PlacedComponent) -> Result<PathBuf> {
        self.resolve_path(component.kind(), component.name(), component.scope())
    }

    /// `SyncOptions` から処理対象の `SyncableKind` リストを取得
    pub(super) fn target_kinds(&self, options: &SyncOptions) -> Vec<SyncableKind> {
        match options.component_type {
            Some(kind) => vec![kind],
            None => SyncableKind::all().to_vec(),
        }
    }

    /// `SyncOptions` から処理対象の `Scope` リストを取得
    pub(super) fn target_scopes(&self, options: &SyncOptions) -> Vec<Scope> {
        match options.scope {
            Some(scope) => vec![scope],
            None => vec![Scope::Personal, Scope::Project],
        }
    }

    /// コンポーネントのパスを解決
    pub(super) fn resolve_path(
        &self,
        kind: ComponentKind,
        name: &str,
        scope: Scope,
    ) -> Result<PathBuf> {
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
