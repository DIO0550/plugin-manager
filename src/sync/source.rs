//! 同期元の定義

use super::options::{SyncOptions, SyncableKind};
use super::placed::{ComponentIdentity, PlacedComponent};
use crate::component::{CommandFormat, ComponentKind, Scope};
use crate::component::{ComponentRef, PlacementContext, PlacementScope, ProjectContext};
use crate::error::{PlmError, Result};
use crate::target::{parse_target, PluginOrigin, Target, TargetKind};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// 同期元
pub struct SyncSource {
    target: Box<dyn Target>,
    project_root: PathBuf,
}

impl std::fmt::Debug for SyncSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncSource")
            .field("target", &self.target.name())
            .field("project_root", &self.project_root)
            .finish()
    }
}

impl SyncSource {
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

    /// Command フォーマットを取得
    pub fn command_format(&self) -> CommandFormat {
        self.target.command_format()
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

    /// コンポーネントのパスを取得
    pub fn path_for(&self, component: &PlacedComponent) -> Result<PathBuf> {
        self.resolve_path(component.kind(), component.name(), component.scope())
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

/// コンポーネント名をパース (marketplace/plugin/component)
pub fn parse_component_name(name: &str) -> Result<(PluginOrigin, &str)> {
    // Instruction の特別扱い
    if name == "AGENTS.md" || name == "copilot-instructions.md" || name == "GEMINI.md" {
        return Ok((PluginOrigin::from_marketplace("", ""), name));
    }

    let parts: Vec<&str> = name.split('/').collect();
    if parts.len() != 3 {
        return Err(PlmError::InvalidArgument(format!(
            "Invalid component name format: '{}'. Expected 'marketplace/plugin/component'",
            name
        )));
    }

    let origin = PluginOrigin::from_marketplace(parts[0], parts[1]);
    Ok((origin, parts[2]))
}

#[cfg(test)]
#[path = "source_test.rs"]
mod tests;
