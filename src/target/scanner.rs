//! 3層ディレクトリ構造のスキャン
//!
//! `<kind>/<marketplace>/<plugin>/<component>` 階層を再帰的に走査する。

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{PlmError, Result};
use crate::target::PluginOrigin;

/// 3層構造の階層定数（マジックナンバー回避）
const LEVEL_MARKETPLACE: usize = 0;
const LEVEL_PLUGIN: usize = 1;
const LEVEL_COMPONENT: usize = 2;

/// スキャンで発見したコンポーネント
#[derive(Debug, Clone)]
pub struct ScannedComponent {
    pub origin: PluginOrigin,
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

/// 3層ディレクトリ構造をスキャンしてコンポーネント一覧を取得
///
/// # 引数
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
    collect_recursively(base_dir, &[], &mut results)?;
    Ok(results)
}

/// 再帰的にディレクトリを走査してコンポーネントを収集
fn collect_recursively(
    dir: &Path,
    path_parts: &[String],
    results: &mut Vec<ScannedComponent>,
) -> Result<()> {
    let current_level = path_parts.len();

    fs::read_dir(dir)?.try_for_each(|result| -> Result<()> {
        let entry = result?;
        let path = entry.path();
        // 既存挙動維持: path.is_dir()はメタデータエラーを握りつぶす
        let is_dir = path.is_dir();

        match current_level {
            LEVEL_COMPONENT => {
                // 最下層: コンポーネントとして収集
                results.push(ScannedComponent {
                    origin: PluginOrigin::from_marketplace(
                        &path_parts[LEVEL_MARKETPLACE],
                        &path_parts[LEVEL_PLUGIN],
                    ),
                    name: entry_name_lossy(&entry),
                    path,
                    is_dir,
                });
            }
            _ if is_dir => {
                // 中間層: ディレクトリのみ再帰
                let mut next_parts = path_parts.to_vec();
                next_parts.push(entry_name_lossy(&entry));
                collect_recursively(&path, &next_parts, results)?;
            }
            _ => {
                // 中間層のファイル: スキップ
            }
        }
        Ok(())
    })
}

/// ファイル名を取得（lossy変換、エラーにしない）
fn entry_name_lossy(entry: &fs::DirEntry) -> String {
    entry.file_name().to_string_lossy().to_string()
}

#[cfg(test)]
#[path = "scanner_test.rs"]
mod tests;
