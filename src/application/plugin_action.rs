//! プラグインアクションと計画
//!
//! Functional Core / Imperative Shell パターンに基づく設計。
//! - PluginAction: 高レベルユースケース（Install/Uninstall/Enable/Disable）
//! - PluginIntent: 意図を表す値オブジェクト（展開可能）
//! - FileOperation: 低レベルファイル操作

use crate::component::{Component, ComponentKind, Scope};
use crate::domain::{ComponentRef, PlacementContext, PlacementScope, ProjectContext};
use crate::error::{PlmError, Result};
use crate::fs::{FileSystem, RealFs};
use crate::target::{all_targets, AffectedTargets, OperationResult, PluginOrigin, Target};
use std::path::{Path, PathBuf};

/// プラグインアクション（高レベルユースケース）
#[derive(Debug, Clone)]
pub enum PluginAction {
    Install {
        plugin_name: String,
        marketplace: Option<String>,
    },
    Uninstall {
        plugin_name: String,
        marketplace: Option<String>,
    },
    Enable {
        plugin_name: String,
        marketplace: Option<String>,
    },
    Disable {
        plugin_name: String,
        marketplace: Option<String>,
    },
}

impl PluginAction {
    /// アクションの種類を文字列で取得
    pub fn kind(&self) -> &'static str {
        match self {
            PluginAction::Install { .. } => "install",
            PluginAction::Uninstall { .. } => "uninstall",
            PluginAction::Enable { .. } => "enable",
            PluginAction::Disable { .. } => "disable",
        }
    }

    /// プラグイン名を取得
    pub fn plugin_name(&self) -> &str {
        match self {
            PluginAction::Install { plugin_name, .. }
            | PluginAction::Uninstall { plugin_name, .. }
            | PluginAction::Enable { plugin_name, .. }
            | PluginAction::Disable { plugin_name, .. } => plugin_name,
        }
    }

    /// マーケットプレイスを取得
    pub fn marketplace(&self) -> Option<&str> {
        match self {
            PluginAction::Install { marketplace, .. }
            | PluginAction::Uninstall { marketplace, .. }
            | PluginAction::Enable { marketplace, .. }
            | PluginAction::Disable { marketplace, .. } => marketplace.as_deref(),
        }
    }

    /// デプロイ系アクションか（Enable/Install）
    pub fn is_deploy(&self) -> bool {
        matches!(
            self,
            PluginAction::Install { .. } | PluginAction::Enable { .. }
        )
    }

    /// 削除系アクションか（Disable/Uninstall）
    pub fn is_remove(&self) -> bool {
        matches!(
            self,
            PluginAction::Uninstall { .. } | PluginAction::Disable { .. }
        )
    }
}

/// ターゲット識別子（型安全）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TargetId(String);

impl TargetId {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for TargetId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl std::fmt::Display for TargetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// スコープ付きパス（project_root配下であることを保証）
///
/// ディレクトリトラバーサル攻撃を防ぐため、
/// パスが指定されたルート配下にあることを型レベルで保証する。
#[derive(Debug, Clone)]
pub struct ScopedPath {
    path: PathBuf,
}

impl ScopedPath {
    /// 検証して生成
    ///
    /// # Errors
    /// - パスが project_root 配下でない場合
    pub fn new(path: PathBuf, project_root: &Path) -> Result<Self> {
        // 絶対パスに変換して比較
        let canonical_root = project_root
            .canonicalize()
            .unwrap_or_else(|_| project_root.to_path_buf());

        // パスがproject_root配下かチェック（パスが存在しない場合は親ディレクトリで判断）
        let check_path = if path.exists() {
            path.canonicalize().unwrap_or_else(|_| path.clone())
        } else {
            // 存在しないパスの場合、親が存在するか確認
            let mut ancestors = path.ancestors();
            let _ = ancestors.next(); // 自分自身をスキップ
            ancestors
                .find(|p| p.exists())
                .and_then(|p| p.canonicalize().ok())
                .unwrap_or_else(|| path.clone())
        };

        // project_root配下であることを確認
        if !check_path.starts_with(&canonical_root) && !path.starts_with(project_root) {
            return Err(PlmError::Validation(format!(
                "Path '{}' is not under project root '{}'",
                path.display(),
                project_root.display()
            )));
        }

        Ok(Self { path })
    }

    /// パスを取得
    pub fn as_path(&self) -> &Path {
        &self.path
    }

    /// PathBuf に変換
    pub fn into_path(self) -> PathBuf {
        self.path
    }
}

/// 低レベルファイル操作（内部用）
#[derive(Debug, Clone)]
pub enum FileOperation {
    CopyFile { source: PathBuf, target: ScopedPath },
    CopyDir { source: PathBuf, target: ScopedPath },
    RemoveFile { path: ScopedPath },
    RemoveDir { path: ScopedPath },
}

impl FileOperation {
    /// 操作の種類を文字列で取得
    pub fn kind(&self) -> &'static str {
        match self {
            FileOperation::CopyFile { .. } => "copy_file",
            FileOperation::CopyDir { .. } => "copy_dir",
            FileOperation::RemoveFile { .. } => "remove_file",
            FileOperation::RemoveDir { .. } => "remove_dir",
        }
    }

    /// ターゲットパスを取得
    pub fn target_path(&self) -> &Path {
        match self {
            FileOperation::CopyFile { target, .. } | FileOperation::CopyDir { target, .. } => {
                target.as_path()
            }
            FileOperation::RemoveFile { path } | FileOperation::RemoveDir { path } => {
                path.as_path()
            }
        }
    }
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
#[path = "plugin_action_test.rs"]
mod tests;
