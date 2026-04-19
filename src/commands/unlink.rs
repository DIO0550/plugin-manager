use std::fs;
use std::path::PathBuf;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Path to the symbolic link to remove
    pub path: PathBuf,
}

/// # Arguments
///
/// * `args` - Parsed CLI arguments for `plm unlink`.
pub async fn run(args: Args) -> Result<(), String> {
    let path = &args.path;

    // Use symlink_metadata() so broken symlinks are still detected as existing.
    let metadata = fs::symlink_metadata(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            format!("Path not found: {}", path.display())
        } else {
            format!("Cannot access {}: {}", path.display(), e)
        }
    })?;

    if !metadata.is_symlink() {
        return Err(format!(
            "Not a symlink: {}. Use rm to remove regular files or directories.",
            path.display()
        ));
    }

    fs::remove_file(path).map_err(|e| format!("Failed to remove symlink: {}", e))?;

    println!("Unlinked: {}", path.display());
    Ok(())
}

#[cfg(all(test, unix))]
#[path = "unlink_test.rs"]
mod tests;
