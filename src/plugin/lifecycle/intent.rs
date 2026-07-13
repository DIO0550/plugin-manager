//! プラグイン操作意図
//!
//! PluginIntent は事前スキャン済みデータを保持し、
//! 低レベルファイル操作への展開と実行を担う。

// Re-exported for tests
#[cfg(test)]
pub use super::action::PluginAction;
#[cfg(not(test))]
use super::action::PluginAction;
use crate::component::{
    Component, ComponentKind, ComponentRef, FileOperation, PlacementContext, PlacementScope,
    ProjectContext, Scope, ScopedPath,
};
use crate::target::{
    all_targets, AffectedTargets, OperationOutcome, PluginOrigin, Target, TargetKind,
};
use std::path::{Path, PathBuf};

/// 単一コンポーネントの操作生成結果
type CreateOperationResult =
    std::result::Result<Option<(TargetKind, FileOperation)>, (TargetKind, String)>;

/// `expand()` の結果
#[derive(Debug)]
pub struct ExpandOutcome {
    /// 正常に生成されたファイル操作
    pub operations: Vec<(TargetKind, FileOperation)>,
    /// パス検証エラー（ターゲット種別, エラーメッセージ）
    pub validation_errors: Vec<(TargetKind, String)>,
}

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
    ///
    /// # Arguments
    ///
    /// * `action` - high-level plugin action to plan for
    /// * `components` - pre-scanned components participating in the action
    /// * `project_root` - project root used for path scoping
    pub fn new(action: PluginAction, components: Vec<Component>, project_root: PathBuf) -> Self {
        Self {
            action,
            components,
            project_root,
            target_filter: None,
        }
    }

    /// ターゲットフィルタ付きで計画を構築
    ///
    /// # Arguments
    ///
    /// * `action` - high-level plugin action to plan for
    /// * `components` - pre-scanned components participating in the action
    /// * `project_root` - project root used for path scoping
    /// * `target_filter` - optional target name restricting expansion to a single target
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

    /// Functional Core: 低レベルファイル操作に展開する。
    ///
    /// 主に保持済みデータを使用して展開を行うが、`create_operation` 内での
    /// パスの検証や正規化などに伴い、ファイルシステムを参照することがある。
    /// target_filter が設定されている場合は、そのターゲットのみを対象とする。
    ///
    /// パス検証エラーが発生した場合は `ExpandOutcome::validation_errors` に記録される。
    pub fn expand(&self) -> ExpandOutcome {
        let targets = all_targets();
        let origin =
            PluginOrigin::from_cached_plugin(self.action.marketplace(), self.action.plugin_name());

        let mut operations = Vec::new();
        let mut validation_errors = Vec::new();

        for target in targets.iter().filter(|target| match &self.target_filter {
            Some(filter) => target.name() == filter,
            None => true,
        }) {
            for component in self.components.iter().filter(|c| target.supports(c.kind)) {
                match self.create_operation(target.as_ref(), component, &origin) {
                    Ok(Some(op)) => operations.push(op),
                    Ok(None) => {} // placement not applicable
                    Err((target_id, msg)) => validation_errors.push((target_id, msg)),
                }
            }
        }

        ExpandOutcome {
            operations,
            validation_errors,
        }
    }

    /// ドライラン: 実行予定の操作を確認
    pub fn dry_run(&self) -> ExpandOutcome {
        self.expand()
    }

    /// FileOperation を構築
    ///
    /// # Arguments
    ///
    /// * `component` - component whose source path feeds the operation
    /// * `scoped` - validated destination path for the operation
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
    ///
    /// - `Ok(None)`: ターゲットがこのコンポーネントの配置場所を持たない（正常）
    /// - `Ok(Some(...))`: 操作を正常に生成
    /// - `Err(...)`: パス検証エラー（ディレクトリトラバーサル等）
    ///
    /// # Arguments
    ///
    /// * `target` - target environment whose placement rules are consulted
    /// * `component` - component to be placed on the target
    /// * `origin` - plugin origin used to build the placement context
    fn create_operation(
        &self,
        target: &dyn Target,
        component: &Component,
        origin: &PluginOrigin,
    ) -> CreateOperationResult {
        let context = PlacementContext {
            component: ComponentRef::new(component.kind, &component.name),
            origin,
            scope: PlacementScope::new(Scope::Project),
            project: ProjectContext::new(&self.project_root),
        };

        let location = match target.placement_location(&context) {
            Some(loc) => loc,
            None => return Ok(None),
        };
        let target_path = location.into_path();
        let scoped = ScopedPath::new(target_path, &self.project_root)
            .map_err(|e| (target.kind(), format!("Path validation failed: {}", e)))?;

        let op = self.build_file_operation(component, scoped);
        Ok(Some((target.kind(), op)))
    }

    /// Imperative Shell: 実行（副作用）
    pub fn apply(self) -> OperationOutcome {
        let result = self.expand();
        execute_file_operations(result, &self.project_root)
    }
}

/// ファイル操作を実行
///
/// # Arguments
///
/// * `expand_outcome` - pre-computed operations and validation errors from `expand`
/// * `_project_root` - project root (currently unused but retained for future scoping needs)
fn execute_file_operations(
    expand_outcome: ExpandOutcome,
    _project_root: &Path,
) -> OperationOutcome {
    use crate::fs::{FileSystem, RealFs};

    let fs = RealFs;
    let mut affected = AffectedTargets::new();

    for (target_kind, msg) in expand_outcome.validation_errors {
        affected.record_error(target_kind.as_str(), msg);
    }

    let mut by_target: std::collections::HashMap<TargetKind, Vec<FileOperation>> =
        std::collections::HashMap::new();

    for (target_kind, op) in expand_outcome.operations {
        by_target.entry(target_kind).or_default().push(op);
    }

    for (target_kind, ops) in by_target {
        let mut success_count = 0;
        let mut error_msg = None;

        for op in ops {
            let result = match &op {
                FileOperation::CopyFile { source, target } => {
                    fs.copy_file(source, target.as_path())
                }
                FileOperation::CopyDir { source, target } => {
                    fs.copy_dir_replace(source, target.as_path())
                }
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
            affected.record_error(target_kind.as_str(), msg);
        } else {
            affected.record_success(target_kind.as_str(), success_count);
        }
    }

    affected.into_result()
}

#[cfg(test)]
#[path = "intent_test.rs"]
mod tests;
