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

mod action;
mod destination;
mod options;
mod placed;
mod result;
mod source;

pub use crate::fs::{FileSystem, RealFs};
pub use action::SyncAction;
pub use destination::SyncDestination;
pub use options::{SyncOptions, SyncableKind};
pub use placed::{ComponentIdentity, PlacedComponent};
pub use result::{SyncFailure, SyncResult};
pub use source::SyncSource;

use crate::error::Result;
use std::collections::HashMap;

/// 同期を実行
///
/// `dry_run: true` の場合は実際のファイル操作を行わず、
/// 何が行われるかを `SyncResult` で返す。
pub fn sync(
    source: &SyncSource,
    dest: &SyncDestination,
    options: &SyncOptions,
) -> Result<SyncResult> {
    sync_with_fs(source, dest, options, &RealFs)
}

/// テスト用エントリポイント（FileSystem を注入）
pub(crate) fn sync_with_fs(
    source: &SyncSource,
    dest: &SyncDestination,
    options: &SyncOptions,
    fs: &dyn FileSystem,
) -> Result<SyncResult> {
    // 1. コンポーネント取得（重複チェック込み）
    let source_components = source.placed_components(options)?;
    let dest_components = dest.placed_components(options)?;

    // 2. HashMap で O(n) 差分計算
    let source_map: HashMap<&ComponentIdentity, &PlacedComponent> = source_components
        .iter()
        .map(|c| (c.identity(), c))
        .collect();
    let dest_map: HashMap<&ComponentIdentity, &PlacedComponent> =
        dest_components.iter().map(|c| (c.identity(), c)).collect();

    let mut to_create = Vec::new();
    let mut to_update = Vec::new();
    let mut skipped = Vec::new();
    let mut unsupported = Vec::new();

    // 3. Source にあるものを処理
    for (identity, src_component) in &source_map {
        // サポートチェック
        if !dest.supports(identity) {
            unsupported.push((*src_component).clone());
            continue;
        }

        match dest_map.get(identity) {
            None => to_create.push((*src_component).clone()),
            Some(dest_component) => {
                // 変更判定
                if needs_update(src_component, dest_component, fs)? {
                    to_update.push((*src_component).clone());
                } else {
                    skipped.push((*src_component).clone());
                }
            }
        }
    }

    // 4. Destination にしかないものを削除対象に
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

    // 5. 実行
    execute_sync(source, dest, to_create, to_update, to_delete, fs)
}

/// 更新が必要かを判定（mtime または内容比較）
fn needs_update(
    src: &PlacedComponent,
    dest: &PlacedComponent,
    fs: &dyn FileSystem,
) -> Result<bool> {
    // ファイルが存在しない場合は更新不要（新規作成になる）
    if !fs.exists(&src.path) || !fs.exists(&dest.path) {
        return Ok(true);
    }

    // mtime 比較（高速）
    let src_mtime = fs.mtime(&src.path)?;
    let dest_mtime = fs.mtime(&dest.path)?;

    if src_mtime > dest_mtime {
        return Ok(true);
    }

    // mtime が同じか古い場合でも内容が違う可能性があるのでハッシュ比較
    Ok(fs.content_hash(&src.path)? != fs.content_hash(&dest.path)?)
}

/// 同期を実行
fn execute_sync(
    source: &SyncSource,
    dest: &SyncDestination,
    to_create: Vec<PlacedComponent>,
    to_update: Vec<PlacedComponent>,
    to_delete: Vec<PlacedComponent>,
    fs: &dyn FileSystem,
) -> Result<SyncResult> {
    let mut result = SyncResult::default();

    // Create
    for component in to_create {
        match execute_create(source, dest, &component, fs) {
            Ok(()) => result.created.push(component),
            Err(e) => result.failed.push(SyncFailure::new(
                component,
                SyncAction::Create,
                e.to_string(),
            )),
        }
    }

    // Update
    for component in to_update {
        match execute_update(source, dest, &component, fs) {
            Ok(()) => result.updated.push(component),
            Err(e) => result.failed.push(SyncFailure::new(
                component,
                SyncAction::Update,
                e.to_string(),
            )),
        }
    }

    // Delete
    for component in to_delete {
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
    } else {
        fs.copy_file(&src_path, &dst_path)
    }
}

/// 更新を実行
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
