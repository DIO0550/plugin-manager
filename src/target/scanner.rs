//! フラット 1 階層ディレクトリ構造のスキャン
//!
//! `<kind>/<flattened_name>` 構造を 1 階層走査する。`Component.name` は
//! `flatten_name(plugin_name, original_name)` で平坦化済みのため、中間
//! ディレクトリは存在しない。`origin` は復元できないためプレースホルダ
//! (`PluginOrigin { marketplace: "_", plugin: "_" }`) を埋めて返す。

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{PlmError, Result};
use crate::target::PluginOrigin;

/// スキャンで発見したコンポーネント
#[derive(Debug, Clone)]
pub struct ScannedComponent {
    pub origin: PluginOrigin,
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

/// フラット 1 階層ディレクトリ構造をスキャンしてコンポーネント一覧を取得
///
/// # 引数
///
/// - `base_dir`: `<kind>`ディレクトリ（skills/, agents/等）
///
/// # 既存挙動の維持
/// - `base_dir`が存在しない場合: 空のVecを返す
/// - ファイルを渡した場合: エラー
pub fn scan_components(base_dir: &Path) -> Result<Vec<ScannedComponent>> {
    if !base_dir.exists() {
        return Ok(Vec::new());
    }
    if !base_dir.is_dir() {
        return Err(PlmError::InvalidArgument(format!(
            "Not a directory: {:?}",
            base_dir
        )));
    }

    let mut results = Vec::new();
    for entry in fs::read_dir(base_dir)? {
        let entry = entry?;
        let path = entry.path();
        // 既存挙動維持: path.is_dir() はメタデータエラーを握りつぶす
        let is_dir = path.is_dir();
        results.push(ScannedComponent {
            origin: PluginOrigin::placeholder(),
            name: entry_name_lossy(&entry),
            path,
            is_dir,
        });
    }
    Ok(results)
}

/// ファイル名を取得（lossy変換、エラーにしない）
///
/// # Arguments
///
/// * `entry` - Directory entry whose file name is read.
fn entry_name_lossy(entry: &fs::DirEntry) -> String {
    entry.file_name().to_string_lossy().to_string()
}

#[cfg(test)]
#[path = "scanner_test.rs"]
mod tests;
