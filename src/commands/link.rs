//! plm link コマンド
//!
//! ソースファイル/ディレクトリへのシンボリックリンクをデスティネーションに作成する。

use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Source path (the actual file/directory)
    pub src: PathBuf,
    /// Destination path (where the symlink will be created)
    pub dest: PathBuf,
    /// Overwrite existing file at dest
    #[arg(long)]
    pub force: bool,
}

/// # Arguments
///
/// * `args` - Parsed CLI arguments for `plm link`.
pub async fn run(args: Args) -> Result<(), String> {
    if !cfg!(unix) {
        return Err("Symbolic links are not supported on Windows".to_string());
    }

    let abs_src = absolutize(&args.src);
    let abs_dest = absolutize(&args.dest);

    // Use `symlink_metadata` to avoid following symlinks when validating src.
    fs::symlink_metadata(&abs_src)
        .map_err(|_| format!("Source not found: {}", args.src.display()))?;

    if abs_src == abs_dest {
        return Err("Source and dest are the same path".to_string());
    }

    if abs_dest.exists() || abs_dest.symlink_metadata().is_ok() {
        if !args.force {
            return Err(format!(
                "Dest already exists: {} (use --force to overwrite)",
                args.dest.display()
            ));
        }

        let meta = fs::symlink_metadata(&abs_dest)
            .map_err(|e| format!("Failed to read dest metadata: {}", e))?;

        if meta.is_symlink() || meta.is_file() {
            fs::remove_file(&abs_dest)
                .map_err(|e| format!("Failed to remove existing dest: {}", e))?;
        } else if meta.is_dir() {
            let is_empty = fs::read_dir(&abs_dest)
                .map_err(|e| format!("Failed to read dest directory: {}", e))?
                .next()
                .is_none();

            if is_empty {
                fs::remove_dir(&abs_dest)
                    .map_err(|e| format!("Failed to remove empty directory: {}", e))?;
            } else {
                return Err("Dest is a non-empty directory. Remove it manually.".to_string());
            }
        }
    }

    if let Some(parent) = abs_dest.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create parent directory: {}", e))?;
    }

    let dest_parent = abs_dest
        .parent()
        .ok_or_else(|| "Dest has no parent directory".to_string())?;
    let relative_path = relative_path_from(&abs_src, dest_parent);

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&relative_path, &abs_dest)
            .map_err(|e| format!("Failed to create symlink: {}", e))?;
    }

    println!(
        "Linked: {} -> {}",
        args.dest.display(),
        relative_path.display()
    );

    Ok(())
}

/// Convert a potentially relative path to an absolute path without resolving symlinks.
///
/// Unlike `canonicalize()`, this function does not resolve symlinks or require
/// the path to exist. It cleans up `.` and `..` components.
///
/// # Arguments
///
/// * `path` - Path to absolutize. May be relative or absolute.
pub(crate) fn absolutize(path: &Path) -> PathBuf {
    let abs = if path.is_relative() {
        match std::env::current_dir() {
            Ok(cwd) => cwd.join(path),
            // If we cannot determine the current directory, fall back to the
            // original path instead of panicking.
            Err(_) => path.to_path_buf(),
        }
    } else {
        path.to_path_buf()
    };

    let mut components = Vec::new();
    for component in abs.components() {
        match component {
            Component::CurDir => {
                // Skip `.`
            }
            Component::ParentDir => {
                // Pop the last normal component if possible, and clamp at root.
                match components.last() {
                    Some(Component::Normal(_)) => {
                        components.pop();
                    }
                    Some(Component::RootDir) => {
                        // Don't allow `..` to go above filesystem root.
                    }
                    _ => {
                        components.push(component);
                    }
                }
            }
            _ => {
                components.push(component);
            }
        }
    }

    components.iter().collect()
}

/// Compute a relative path from `dest_parent` to `src`.
///
/// Both `src` and `dest_parent` must be absolute paths. The result is a
/// relative path that, when resolved from `dest_parent`, points to `src`.
///
/// # Arguments
///
/// * `src` - Absolute path to the file the symlink should target.
/// * `dest_parent` - Absolute directory that contains the symlink.
///
/// # Examples
///
/// ```text
/// src=/a/CLAUDE.md, dest_parent=/a/.github  -> ../CLAUDE.md
/// src=/a/b.md,      dest_parent=/a          -> b.md
/// ```
pub(crate) fn relative_path_from(src: &Path, dest_parent: &Path) -> PathBuf {
    let src_components: Vec<_> = src.components().collect();
    let dest_components: Vec<_> = dest_parent.components().collect();

    let common_len = src_components
        .iter()
        .zip(dest_components.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let mut result = PathBuf::new();

    for _ in common_len..dest_components.len() {
        result.push("..");
    }

    for component in &src_components[common_len..] {
        result.push(component.as_os_str());
    }

    // If src and dest_parent are identical, result would be empty.
    // Return "." to avoid creating a symlink with an empty target.
    if result.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        result
    }
}

#[cfg(all(test, unix))]
#[path = "link_test.rs"]
mod tests;
