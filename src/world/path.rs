//! Path utilities.

use std::path::{Path, PathBuf};

/// Normalize a file system path to absolute form.
///
/// Tries `canonicalize()` first (resolves symlinks, `.`, `..`).
/// Falls back to:
/// - Return as-is if already absolute
/// - Join with current directory if relative
#[inline]
pub fn normalize_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir().map_or_else(|_| path.to_path_buf(), |cwd| cwd.join(path))
        }
    })
}
