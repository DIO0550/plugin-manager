//! 同期 Feature
//!
//! 異なるターゲット環境間でコンポーネントを同期する。
//!
//! ## 使い方
//!
//! ```ignore
//! use plm::sync::{sync, SyncSource, SyncDestination, SyncOptions};
//! use plm::target::TargetKind;
//!
//! let source = SyncSource::new(TargetKind::Codex, &project_root)?;
//! let dest = SyncDestination::new(TargetKind::Copilot, &project_root)?;
//!
//! let result = sync(&source, &dest, &SyncOptions {
//!     dry_run: true,
//!     ..Default::default()
//! })?;
//!
//! println!("Created: {}", result.create_count());
//! println!("Updated: {}", result.update_count());
//! println!("Deleted: {}", result.delete_count());
//! ```

mod endpoint;
mod model;

pub use crate::fs::{FileSystem, RealFs};
pub use endpoint::{SyncDestination, SyncSource};
pub use model::{
    PlacedComponent, PlacedRef, SyncAction, SyncFailure, SyncOptions, SyncResult, SyncableKind,
};

use crate::component::{convert, ComponentKind};
use crate::error::Result;
use endpoint::Endpoint;
use std::collections::HashMap;

/// 同期を実行
///
/// `dry_run: true` の場合は実際のファイル操作を行わず、
/// 何が行われるかを `SyncResult` で返す。
///
/// # Arguments
///
/// * `source` - Source target to read placed components from.
/// * `dest` - Destination target to synchronize into.
/// * `options` - Options controlling scope, component type and dry-run behavior.
pub fn sync(
    source: &SyncSource,
    dest: &SyncDestination,
    options: &SyncOptions,
) -> Result<SyncResult> {
    sync_with_fs(source, dest, options, &RealFs)
}

/// テスト用エントリポイント（FileSystem を注入）
///
/// # Arguments
///
/// * `source` - Source target to read placed components from.
/// * `dest` - Destination target to synchronize into.
/// * `options` - Options controlling scope, component type and dry-run behavior.
/// * `fs` - File system abstraction used for existence, mtime and content-hash checks.
pub(crate) fn sync_with_fs(
    source: &SyncSource,
    dest: &SyncDestination,
    options: &SyncOptions,
    fs: &dyn FileSystem,
) -> Result<SyncResult> {
    let source_components = collect_components(Endpoint::Source(source), options)?;
    let dest_components = collect_components(Endpoint::Destination(dest), options)?;

    let source_map: HashMap<&PlacedRef, &PlacedComponent> = source_components
        .iter()
        .map(|c| (c.placed_ref(), c))
        .collect();
    let dest_map: HashMap<&PlacedRef, &PlacedComponent> = dest_components
        .iter()
        .map(|c| (c.placed_ref(), c))
        .collect();

    let mut to_create = Vec::new();
    let mut to_update = Vec::new();
    let mut skipped = Vec::new();
    let mut unsupported = Vec::new();

    for (placed_ref, src_component) in &source_map {
        if !dest.supports(placed_ref) {
            unsupported.push((*src_component).clone());
            continue;
        }

        match dest_map.get(placed_ref) {
            None => to_create.push((*src_component).clone()),
            Some(dest_component) => {
                if needs_update(src_component, dest_component, fs)? {
                    to_update.push((*src_component).clone());
                } else {
                    skipped.push((*src_component).clone());
                }
            }
        }
    }

    let to_delete: Vec<PlacedComponent> = dest_map
        .iter()
        .filter(|(id, _)| !source_map.contains_key(*id))
        .map(|(_, c)| (*c).clone())
        .collect();

    if options.dry_run {
        return Ok(SyncResult::dry_run(
            to_create,
            to_update,
            to_delete,
            skipped,
            unsupported,
        ));
    }

    let plan = SyncPlan {
        to_create,
        to_update,
        to_delete,
    };
    execute_sync(source, dest, plan, fs)
}

fn collect_components(
    endpoint: Endpoint<'_>,
    options: &SyncOptions,
) -> Result<Vec<PlacedComponent>> {
    endpoint.placed_components(options)
}

/// 更新が必要かを判定（mtime または内容比較）
///
/// # Arguments
///
/// * `src` - Source-side placed component to compare.
/// * `dest` - Destination-side placed component to compare.
/// * `fs` - File system abstraction used to query mtime and content hash.
fn needs_update(
    src: &PlacedComponent,
    dest: &PlacedComponent,
    fs: &dyn FileSystem,
) -> Result<bool> {
    // ファイルが存在しない場合は更新不要（新規作成になる）
    if !fs.exists(&src.path) || !fs.exists(&dest.path) {
        return Ok(true);
    }

    let src_mtime = fs.mtime(&src.path)?;
    let dest_mtime = fs.mtime(&dest.path)?;

    if src_mtime > dest_mtime {
        return Ok(true);
    }

    // mtime が同じか古い場合でも内容が違う可能性があるのでハッシュ比較
    Ok(fs.content_hash(&src.path)? != fs.content_hash(&dest.path)?)
}

/// 同期の実行計画
struct SyncPlan {
    to_create: Vec<PlacedComponent>,
    to_update: Vec<PlacedComponent>,
    to_delete: Vec<PlacedComponent>,
}

/// 同期を実行
///
/// # Arguments
///
/// * `source` - Source target used to resolve source paths.
/// * `dest` - Destination target used to resolve destination paths.
/// * `plan` - Precomputed plan containing create/update/delete component sets.
/// * `fs` - File system abstraction used to perform copy, remove and directory operations.
fn execute_sync(
    source: &SyncSource,
    dest: &SyncDestination,
    plan: SyncPlan,
    fs: &dyn FileSystem,
) -> Result<SyncResult> {
    let mut result = SyncResult::default();

    for component in plan.to_create {
        match execute_create(source, dest, &component, fs) {
            Ok(()) => result.created.push(component),
            Err(e) => result.failed.push(SyncFailure::new(
                component,
                SyncAction::Create,
                e.to_string(),
            )),
        }
    }

    for component in plan.to_update {
        match execute_update(source, dest, &component, fs) {
            Ok(()) => result.updated.push(component),
            Err(e) => result.failed.push(SyncFailure::new(
                component,
                SyncAction::Update,
                e.to_string(),
            )),
        }
    }

    for component in plan.to_delete {
        match execute_delete(&component, fs) {
            Ok(()) => result.deleted.push(component),
            Err(e) => result.failed.push(SyncFailure::new(
                component,
                SyncAction::Delete,
                e.to_string(),
            )),
        }
    }

    Ok(result)
}

/// 新規作成を実行
///
/// # Arguments
///
/// * `source` - Source target used to resolve the source path.
/// * `dest` - Destination target used to resolve the destination path.
/// * `component` - Component to create on the destination.
/// * `fs` - File system abstraction used for copy operations.
fn execute_create(
    source: &SyncSource,
    dest: &SyncDestination,
    component: &PlacedComponent,
    fs: &dyn FileSystem,
) -> Result<()> {
    let src_path = source.path_for(component)?;
    let dst_path = dest.path_for(component)?;

    if fs.is_dir(&src_path) {
        fs.copy_dir(&src_path, &dst_path)
    } else if component.kind() == ComponentKind::Command {
        // Command はフォーマット変換を行う
        convert::convert_and_write(
            &src_path,
            &dst_path,
            source.command_format(),
            dest.command_format(),
        )?;
        Ok(())
    } else {
        fs.copy_file(&src_path, &dst_path)
    }
}

/// 更新を実行
///
/// # Arguments
///
/// * `source` - Source target used to resolve the source path.
/// * `dest` - Destination target used to resolve the destination path.
/// * `component` - Component to update on the destination.
/// * `fs` - File system abstraction used for copy operations.
fn execute_update(
    source: &SyncSource,
    dest: &SyncDestination,
    component: &PlacedComponent,
    fs: &dyn FileSystem,
) -> Result<()> {
    // 更新は作成と同じ操作（上書き）
    execute_create(source, dest, component, fs)
}

/// 削除を実行
///
/// # Arguments
///
/// * `component` - Component whose destination path should be removed.
/// * `fs` - File system abstraction used to check existence and remove the entry.
fn execute_delete(component: &PlacedComponent, fs: &dyn FileSystem) -> Result<()> {
    if fs.exists(&component.path) {
        fs.remove(&component.path)
    } else {
        Ok(())
    }
}

#[cfg(test)]
#[path = "sync_test.rs"]
mod tests;
