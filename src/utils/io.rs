use std::path::{Path, PathBuf};

/// Make sure that directory exists.
pub fn ensure_directory(path: &Path) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(path)?;
    Ok(path.to_path_buf())
}

/// Clears all files from the directory, and recreates it.
pub fn clear_directory(path: &Path) -> std::io::Result<PathBuf> {
    std::fs::remove_dir_all(path)?;
    ensure_directory(path)
}
