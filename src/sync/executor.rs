//! 同期実行器

use super::types::{SyncAction, SyncFailure, SyncItem, SyncOptions, SyncPlan, SyncResult, SyncableKind};
use crate::component::{ComponentKind, Scope};
use crate::domain::{ComponentRef, PlacementContext, PlacementScope, ProjectContext};
use crate::error::{PlmError, Result};
use crate::target::{parse_target, PluginOrigin, Target, TargetKind};
use std::fs;
use std::path::{Path, PathBuf};

/// 同期実行器
pub struct SyncExecutor {
    from: Box<dyn Target>,
    to: Box<dyn Target>,
    project_root: PathBuf,
}

impl std::fmt::Debug for SyncExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncExecutor")
            .field("from", &self.from.name())
            .field("to", &self.to.name())
            .field("project_root", &self.project_root)
            .finish()
    }
}

impl SyncExecutor {
    /// コンストラクタ（同じターゲットへの同期はエラー）
    pub fn new(from: TargetKind, to: TargetKind, project_root: &Path) -> Result<Self> {
        if from.as_str() == to.as_str() {
            return Err(PlmError::InvalidArgument(
                "Cannot sync to the same target".to_string(),
            ));
        }

        let from_target = parse_target(from.as_str())?;
        let to_target = parse_target(to.as_str())?;

        Ok(Self {
            from: from_target,
            to: to_target,
            project_root: project_root.to_path_buf(),
        })
    }

    /// 同期計画を作成
    pub fn plan(&self, options: &SyncOptions) -> Result<SyncPlan> {
        let mut items = Vec::new();

        // 対象種別を取得
        let kinds: Vec<SyncableKind> = match options.component_type {
            Some(kind) => vec![kind],
            None => SyncableKind::all().to_vec(),
        };

        // 対象スコープを取得
        let scopes: Vec<Scope> = match options.scope {
            Some(scope) => vec![scope],
            None => vec![Scope::Personal, Scope::Project],
        };

        for syncable_kind in &kinds {
            let kind = syncable_kind.to_component_kind();

            for &scope in &scopes {
                // ソースの配置済みコンポーネントを取得
                let placed = self.from.list_placed(kind, scope, &self.project_root)?;

                for name in placed {
                    let item = self.create_sync_item(kind, &name, scope)?;
                    items.push(item);
                }
            }
        }

        Ok(SyncPlan {
            from_target: self.from.name().to_string(),
            to_target: self.to.name().to_string(),
            items,
        })
    }

    /// 同期を実行（アトミックコピー with バックアップ）
    pub fn execute(&self, plan: &SyncPlan) -> SyncResult {
        let mut result = SyncResult::default();

        for item in &plan.items {
            if item.action.is_skip() {
                continue;
            }

            match copy_component_atomic(item) {
                Ok(()) => {
                    result.succeeded.push(item.clone());
                }
                Err(e) => {
                    result.failed.push(SyncFailure {
                        item: item.clone(),
                        error: e.to_string(),
                    });
                }
            }
        }

        result
    }

    /// SyncItem を作成
    fn create_sync_item(&self, kind: ComponentKind, name: &str, scope: Scope) -> Result<SyncItem> {
        // 宛先ターゲットがこの種別・スコープをサポートするかチェック
        if !self.to.supports(kind) {
            return Ok(SyncItem {
                kind,
                name: name.to_string(),
                scope,
                source_path: PathBuf::new(),
                target_path: PathBuf::new(),
                action: SyncAction::skip(format!(
                    "{} is not supported on {}",
                    kind.display_name(),
                    self.to.display_name()
                )),
            });
        }

        if !self.to.supports_scope(kind, scope) {
            return Ok(SyncItem {
                kind,
                name: name.to_string(),
                scope,
                source_path: PathBuf::new(),
                target_path: PathBuf::new(),
                action: SyncAction::skip(format!(
                    "{} with {} scope is not supported on {}",
                    kind.display_name(),
                    scope.as_str(),
                    self.to.display_name()
                )),
            });
        }

        // コンポーネント名をパース (marketplace/plugin/component)
        let (origin, component_name) = parse_component_name(name)?;

        // ソースパスを取得
        let source_ctx = PlacementContext {
            component: ComponentRef::new(kind, component_name),
            origin: &origin,
            scope: PlacementScope(scope),
            project: ProjectContext::new(&self.project_root),
        };
        let source_location = self.from.placement_location(&source_ctx);
        let source_path = source_location
            .map(|l| l.into_path())
            .unwrap_or_default();

        // 宛先パスを取得
        let target_ctx = PlacementContext {
            component: ComponentRef::new(kind, component_name),
            origin: &origin,
            scope: PlacementScope(scope),
            project: ProjectContext::new(&self.project_root),
        };
        let target_location = self.to.placement_location(&target_ctx);
        let target_path = target_location
            .map(|l| l.into_path())
            .unwrap_or_default();

        // アクションを決定
        let action = if target_path.exists() {
            SyncAction::Overwrite
        } else {
            SyncAction::Create
        };

        Ok(SyncItem {
            kind,
            name: name.to_string(),
            scope,
            source_path,
            target_path,
            action,
        })
    }
}

/// コンポーネント名をパース (marketplace/plugin/component)
fn parse_component_name(name: &str) -> Result<(PluginOrigin, &str)> {
    // Instruction の特別扱い
    if name == "AGENTS.md" || name == "copilot-instructions.md" {
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

/// アトミックコピー with バックアップ
fn copy_component_atomic(item: &SyncItem) -> Result<()> {
    let src = &item.source_path;
    let dst = &item.target_path;

    // 親ディレクトリを作成
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }

    // 一時パスを作成（宛先の親ディレクトリ内、同一ボリューム保証）
    let temp_path = create_temp_in_parent(dst)?;

    // ソースを一時パスにコピー
    if src.is_dir() {
        copy_dir(src, &temp_path)?;
    } else {
        copy_file(src, &temp_path)?;
    }

    // バックアップを作成（既存ファイルがあれば）
    let backup = backup_existing(dst)?;

    // アトミックリプレース
    match atomic_replace(&temp_path, dst, backup.as_deref()) {
        Ok(()) => {
            // 成功: バックアップを削除
            if let Some(backup_path) = backup {
                let _ = remove_any(&backup_path);
            }
            Ok(())
        }
        Err(e) => {
            // 失敗: 一時ファイルを削除
            let _ = remove_any(&temp_path);
            Err(e)
        }
    }
}

/// 宛先親ディレクトリに一時パスを作成（同一ボリューム保証）
fn create_temp_in_parent(dst: &Path) -> Result<PathBuf> {
    let parent = dst.parent().ok_or_else(|| {
        PlmError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Destination has no parent directory",
        ))
    })?;

    let file_name = dst
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "temp".to_string());

    let temp_name = format!(".{}.tmp.{}", file_name, std::process::id());
    Ok(parent.join(temp_name))
}

/// 既存ファイル/ディレクトリを .bak にリネーム（バックアップ）
fn backup_existing(dst: &Path) -> Result<Option<PathBuf>> {
    if !dst.exists() {
        return Ok(None);
    }

    let backup_path = dst.with_extension("bak");

    // 既存のバックアップがあれば削除
    if backup_path.exists() {
        remove_any(&backup_path)?;
    }

    fs::rename(dst, &backup_path)?;
    Ok(Some(backup_path))
}

/// アトミックリプレース（リネーム + ロールバック）
fn atomic_replace(temp_path: &Path, dst: &Path, backup: Option<&Path>) -> Result<()> {
    match fs::rename(temp_path, dst) {
        Ok(()) => Ok(()),
        Err(e) => {
            // 失敗時: バックアップを戻す
            if let Some(backup_path) = backup {
                let _ = fs::rename(backup_path, dst);
            }
            Err(PlmError::Io(e))
        }
    }
}

/// ファイル/ディレクトリを削除
fn remove_any(path: &Path) -> Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

/// ディレクトリを再帰的にコピー
fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(&file_name);

        if entry_path.is_dir() {
            copy_dir(&entry_path, &dst_path)?;
        } else {
            copy_file(&entry_path, &dst_path)?;
        }
    }

    Ok(())
}

/// ファイルをコピー
fn copy_file(src: &Path, dst: &Path) -> Result<()> {
    fs::copy(src, dst)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cannot_sync_to_same_target() {
        let result = SyncExecutor::new(TargetKind::Codex, TargetKind::Codex, Path::new("."));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("same target"));
    }

    #[test]
    fn test_parse_component_name_valid() {
        let (origin, name) = parse_component_name("github/my-plugin/my-skill").unwrap();
        assert_eq!(origin.marketplace, "github");
        assert_eq!(origin.plugin, "my-plugin");
        assert_eq!(name, "my-skill");
    }

    #[test]
    fn test_parse_component_name_instruction() {
        let (origin, name) = parse_component_name("AGENTS.md").unwrap();
        assert_eq!(origin.marketplace, "");
        assert_eq!(origin.plugin, "");
        assert_eq!(name, "AGENTS.md");
    }

    #[test]
    fn test_parse_component_name_error() {
        let result = parse_component_name("invalid");
        assert!(result.is_err());

        let result = parse_component_name("only/two");
        assert!(result.is_err());
    }
}
