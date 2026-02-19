//! プラグイン操作意図
//!
//! PluginIntent は事前スキャン済みデータを保持し、
//! 低レベルファイル操作への展開と実行を担う。

// Re-exported for tests
#[cfg(test)]
pub use super::plugin_action::PluginAction;
#[cfg(not(test))]
use super::plugin_action::PluginAction;
use super::plugin_action_types::{FileOperation, ScopedPath, TargetId};
use crate::component::{
    Component, ComponentKind, ComponentRef, PlacementContext, PlacementScope, ProjectContext, Scope,
};
use crate::fs::{FileSystem, RealFs};
use crate::target::{all_targets, AffectedTargets, OperationResult, PluginOrigin, Target};
use std::path::{Path, PathBuf};

/// プラグイン操作意図（事前スキャン済みデータを保持）
#[derive(Debug)]
pub struct PluginIntent {
    action: PluginAction,
    components: Vec<Component>,
    project_root: PathBuf,
    target_filter: Option<String>,
}

impl PluginIntent {
    /// 計画を構築
    pub fn new(action: PluginAction, components: Vec<Component>, project_root: PathBuf) -> Self {
        Self {
            action,
            components,
            project_root,
            target_filter: None,
        }
    }

    /// ターゲットフィルタ付きで計画を構築
    pub fn with_target_filter(
        action: PluginAction,
        components: Vec<Component>,
        project_root: PathBuf,
        target_filter: Option<&str>,
    ) -> Self {
        Self {
            action,
            components,
            project_root,
            target_filter: target_filter.map(String::from),
        }
    }

    /// アクションを取得
    pub fn action(&self) -> &PluginAction {
        &self.action
    }

    /// コンポーネント数を取得
    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    /// Functional Core: 低レベルファイル操作に展開（完全に純粋）
    ///
    /// ファイルシステムにアクセスしない。保持済みデータのみ使用。
    /// target_filter が設定されている場合は、そのターゲットのみを対象とする。
    pub fn expand(&self) -> Vec<(TargetId, FileOperation)> {
        let targets = all_targets();
        let origin =
            PluginOrigin::from_cached_plugin(self.action.marketplace(), self.action.plugin_name());

        targets
            .iter()
            .filter(|target| {
                // ターゲットフィルタが指定されている場合は一致するもののみ
                match &self.target_filter {
                    Some(filter) => target.name() == filter,
                    None => true,
                }
            })
            .flat_map(|target| {
                self.components
                    .iter()
                    .filter(|component| target.supports(component.kind))
                    .filter_map(|component| {
                        self.create_operation(target.as_ref(), component, &origin)
                    })
            })
            .collect()
    }

    /// ドライラン: 実行予定の操作を確認
    pub fn dry_run(&self) -> Vec<(TargetId, FileOperation)> {
        self.expand()
    }

    /// FileOperation を構築
    fn build_file_operation(&self, component: &Component, scoped: ScopedPath) -> FileOperation {
        match (self.action.is_deploy(), component.kind) {
            (true, ComponentKind::Skill) => FileOperation::CopyDir {
                source: component.path.clone(),
                target: scoped,
            },
            (true, _) => FileOperation::CopyFile {
                source: component.path.clone(),
                target: scoped,
            },
            (false, ComponentKind::Skill) => FileOperation::RemoveDir { path: scoped },
            (false, _) => FileOperation::RemoveFile { path: scoped },
        }
    }

    /// 単一コンポーネントの操作を生成
    fn create_operation(
        &self,
        target: &dyn Target,
        component: &Component,
        origin: &PluginOrigin,
    ) -> Option<(TargetId, FileOperation)> {
        let context = PlacementContext {
            component: ComponentRef::new(component.kind, &component.name),
            origin,
            scope: PlacementScope(Scope::Project),
            project: ProjectContext::new(&self.project_root),
        };

        let location = target.placement_location(&context)?;
        let target_path = location.into_path();
        let scoped = ScopedPath::new(target_path, &self.project_root).ok()?;

        let op = self.build_file_operation(component, scoped);
        Some((TargetId::new(target.name()), op))
    }

    /// Imperative Shell: 実行（副作用）
    pub fn apply(self) -> OperationResult {
        let operations = self.expand();
        execute_file_operations(operations, &self.project_root)
    }
}

/// ファイル操作を実行
fn execute_file_operations(
    operations: Vec<(TargetId, FileOperation)>,
    _project_root: &Path,
) -> OperationResult {
    use crate::path_ext::PathExt;

    let fs = RealFs;
    let mut affected = AffectedTargets::new();

    // ターゲットごとにグループ化
    let mut by_target: std::collections::HashMap<TargetId, Vec<FileOperation>> =
        std::collections::HashMap::new();

    for (target_id, op) in operations {
        by_target.entry(target_id).or_default().push(op);
    }

    for (target_id, ops) in by_target {
        let mut success_count = 0;
        let mut error_msg = None;

        for op in ops {
            let result = match &op {
                FileOperation::CopyFile { source, target } => source.copy_file_to(target.as_path()),
                FileOperation::CopyDir { source, target } => source.copy_dir_to(target.as_path()),
                FileOperation::RemoveFile { path } => {
                    let p = path.as_path();
                    if fs.exists(p) {
                        fs.remove_file(p)
                    } else {
                        Ok(())
                    }
                }
                FileOperation::RemoveDir { path } => {
                    let p = path.as_path();
                    if fs.exists(p) {
                        fs.remove_dir_all(p)
                    } else {
                        Ok(())
                    }
                }
            };

            match result {
                Ok(()) => success_count += 1,
                Err(e) => {
                    error_msg = Some(e.to_string());
                    break;
                }
            }
        }

        if let Some(msg) = error_msg {
            affected.record_error(target_id.as_str(), msg);
        } else {
            affected.record_success(target_id.as_str(), success_count);
        }
    }

    affected.into_result()
}

#[cfg(test)]
#[path = "plugin_intent_test.rs"]
mod tests;
