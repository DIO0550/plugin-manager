//! `list_placed` 共通骨格ヘルパ

use crate::error::Result;
use crate::target::scanner::{scan_components, ScannedComponent};
use std::path::Path;

/// `base/<subdir>` をスキャンしてフィルタする共通骨格。
pub(crate) fn scan_and_filter(
    base: &Path,
    subdir: &str,
    filter: impl Fn(&ScannedComponent) -> Option<String>,
) -> Result<Vec<String>> {
    scan_and_filter_in(&base.join(subdir), filter)
}

/// 指定ディレクトリを直接スキャンしてフィルタする（Hook の base 直下用）。
pub(crate) fn scan_and_filter_in(
    dir: &Path,
    filter: impl Fn(&ScannedComponent) -> Option<String>,
) -> Result<Vec<String>> {
    let names = scan_components(dir)?
        .into_iter()
        .filter_map(|c| filter(&c))
        .collect();
    Ok(names)
}

/// Instruction ファイルの存在確認（パスは呼び出し元が計算する）。
pub(crate) fn list_instruction_at(path: &Path, filename: &str) -> Vec<String> {
    if path.exists() {
        vec![filename.to_string()]
    } else {
        vec![]
    }
}

#[cfg(test)]
#[path = "list_helpers_test.rs"]
mod tests;
