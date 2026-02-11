use std::fs;
use std::path::PathBuf;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Path to the symbolic link to remove
    pub path: PathBuf,
}

pub async fn run(args: Args) -> Result<(), String> {
    let path = &args.path;

    // Use symlink_metadata() to check path exists (detects broken symlinks too)
    let metadata =
        fs::symlink_metadata(path).map_err(|_| format!("Path not found: {}", path.display()))?;

    // Check that it is actually a symlink
    if !metadata.is_symlink() {
        return Err(format!(
            "Not a symlink: {}. Use rm to remove regular files or directories.",
            path.display()
        ));
    }

    // Remove the symlink
    fs::remove_file(path).map_err(|e| format!("Failed to remove symlink: {}", e))?;

    println!("Unlinked: {}", path.display());
    Ok(())
}

#[cfg(test)]
#[path = "unlink_test.rs"]
mod tests;
