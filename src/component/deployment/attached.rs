//! Plugin 付属リソースの Skill 配置ディレクトリへの overlay
//!
//! `deploy_skill` の `replace_dir` **後**に呼び、相対パスを保って複製する。
//! Skill 側に同名がある場合は Skill を優先（上書きしない）し警告する。

use crate::error::Result;
use crate::fs::FileSystem;
use crate::placement_names::ATTACHED_RESOURCES_MAX_BYTES;
use crate::scan::AttachedEntry;
use std::path::{Path, PathBuf};

/// Plugin 付属リソース配置時の警告。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttachedResourceWarning {
    /// Skill 内に同相対パスが既にあり、Plugin 側をスキップした。
    SkillPreferred { relative: PathBuf },
    /// 付属リソース総量が閾値を超えたため一切配置しなかった。
    SkippedTooLarge { total_bytes: u64, limit_bytes: u64 },
}

impl std::fmt::Display for AttachedResourceWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SkillPreferred { relative } => write!(
                f,
                "plugin attached resource skipped (skill already has '{relative}'): skill wins",
                relative = relative.display()
            ),
            Self::SkippedTooLarge {
                total_bytes,
                limit_bytes,
            } => write!(
                f,
                "plugin attached resources skipped: total size {total_bytes} bytes exceeds limit {limit_bytes} bytes"
            ),
        }
    }
}

/// Plugin 付属リソースを Skill 配置ディレクトリへ overlay する。
pub fn overlay_attached_resources(
    fs: &dyn FileSystem,
    plugin_root: &Path,
    entries: &[AttachedEntry],
    skill_target: &Path,
) -> Result<Vec<AttachedResourceWarning>> {
    overlay_attached_resources_with_limit(
        fs,
        plugin_root,
        entries,
        skill_target,
        ATTACHED_RESOURCES_MAX_BYTES,
    )
}

/// サイズ上限を指定して Plugin 付属リソースを overlay する（テスト用に閾値を注入可能）。
pub fn overlay_attached_resources_with_limit(
    fs: &dyn FileSystem,
    plugin_root: &Path,
    entries: &[AttachedEntry],
    skill_target: &Path,
    max_bytes: u64,
) -> Result<Vec<AttachedResourceWarning>> {
    if entries.is_empty() {
        return Ok(Vec::new());
    }

    let total = measure_attached_bytes(fs, plugin_root, entries);
    if total > max_bytes {
        return Ok(vec![AttachedResourceWarning::SkippedTooLarge {
            total_bytes: total,
            limit_bytes: max_bytes,
        }]);
    }

    let mut warnings = Vec::new();
    for entry in entries {
        let src = plugin_root.join(&entry.relative);
        let dst = skill_target.join(&entry.relative);

        if !fs.exists(&src) {
            continue;
        }

        if fs.is_dir(&src) {
            overlay_dir(fs, &src, &dst, &entry.relative, &mut warnings)?;
        } else if fs.exists(&dst) {
            warnings.push(AttachedResourceWarning::SkillPreferred {
                relative: entry.relative.clone(),
            });
        } else {
            fs.copy_file(&src, &dst)?;
        }
    }

    Ok(warnings)
}

fn overlay_dir(
    fs: &dyn FileSystem,
    src_dir: &Path,
    dst_dir: &Path,
    rel_prefix: &Path,
    warnings: &mut Vec<AttachedResourceWarning>,
) -> Result<()> {
    if !fs.exists(dst_dir) {
        fs.create_dir_all(dst_dir)?;
    }

    for node in fs.read_dir(src_dir)? {
        let name = match node.path.file_name() {
            Some(n) => n.to_os_string(),
            None => continue,
        };
        if node.is_symlink() {
            continue;
        }
        let child_rel = rel_prefix.join(&name);
        let src_child = node.path.clone();
        let dst_child = dst_dir.join(&name);

        if node.is_dir() {
            overlay_dir(fs, &src_child, &dst_child, &child_rel, warnings)?;
        } else if fs.exists(&dst_child) {
            warnings.push(AttachedResourceWarning::SkillPreferred {
                relative: child_rel,
            });
        } else {
            fs.copy_file(&src_child, &dst_child)?;
        }
    }
    Ok(())
}

fn measure_attached_bytes(
    fs: &dyn FileSystem,
    plugin_root: &Path,
    entries: &[AttachedEntry],
) -> u64 {
    let mut total = 0u64;
    for entry in entries {
        let path = plugin_root.join(&entry.relative);
        total = total.saturating_add(measure_path_bytes(fs, &path));
    }
    total
}

fn measure_path_bytes(fs: &dyn FileSystem, path: &Path) -> u64 {
    if !fs.exists(path) {
        return 0;
    }
    if fs.is_dir(path) {
        let Ok(nodes) = fs.read_dir(path) else {
            return 0;
        };
        return nodes
            .into_iter()
            .filter(|n| !n.is_symlink())
            .map(|n| measure_path_bytes(fs, &n.path))
            .fold(0u64, u64::saturating_add);
    }
    fs.file_size(path).unwrap_or(0)
}

#[cfg(test)]
#[path = "attached_test.rs"]
mod tests;
