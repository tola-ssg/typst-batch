//! File reading with virtual file system and package support.

use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::sync::LazyLock;

use typst::diag::{FileError, FileResult};
use typst::syntax::{FileId, VirtualPath};
use typst_kit::download::{DownloadState, Progress};

use super::access::record_file_access;
use super::vfs::{read_virtual, read_virtual_package, NoVirtualFS, VirtualFileSystem};
use crate::resource::package;



/// Virtual `FileId` for stdin input.
pub static STDIN_ID: LazyLock<FileId> =
    LazyLock::new(|| FileId::new_fake(VirtualPath::new("<stdin>")));

/// Virtual `FileId` for empty/no input.
pub static EMPTY_ID: LazyLock<FileId> =
    LazyLock::new(|| FileId::new_fake(VirtualPath::new("<empty>")));



/// Create a `FileId` for a project file.
pub fn file_id(path: impl AsRef<Path>) -> FileId {
    FileId::new(None, VirtualPath::new(path.as_ref()))
}

/// Create a `FileId` from absolute path within project root.
pub fn file_id_from_path(file_path: &Path, root: &Path) -> Option<FileId> {
    VirtualPath::within_root(file_path, root).map(|vpath| FileId::new(None, vpath))
}

/// Create a virtual/fake `FileId` for dynamically generated content.
pub fn virtual_file_id(name: &str) -> FileId {
    FileId::new_fake(VirtualPath::new(name))
}



/// Read file content from a `FileId` (no virtual support).
pub fn read_file(id: FileId, project_root: &Path) -> FileResult<Vec<u8>> {
    read_with_virtual(id, project_root, &NoVirtualFS)
}

/// Read file content using the global virtual file system.
///
/// Resolution order:
/// 1. Special IDs (EMPTY, STDIN)
/// 2. Virtual packages (`@myapp/data:0.0.0`)
/// 3. Virtual paths (`/_data/*.json`)
/// 4. Physical files
pub fn read_with_global_virtual(id: FileId, project_root: &Path) -> FileResult<Vec<u8>> {
    // Handle special file IDs
    if id == *EMPTY_ID {
        return Ok(Vec::new());
    }
    if id == *STDIN_ID {
        return read_stdin();
    }

    // Check virtual package first (VPS support)
    if let Some(spec) = id.package()
        && let Some(content) = read_virtual_package(spec, id.vpath()) {
            record_file_access(id);
            return Ok(content);
        }

    // Check virtual path (VFS support)
    let vpath = id.vpath().as_rooted_path();
    if let Some(content) = read_virtual(vpath) {
        record_file_access(id);
        return Ok(content);
    }

    // Resolve and read from disk
    let path = resolve_path(project_root, id)?;
    read_disk(&path)
}

/// Read file content with explicit virtual file system.
pub fn read_with_virtual<V: VirtualFileSystem>(
    id: FileId,
    project_root: &Path,
    virtual_fs: &V,
) -> FileResult<Vec<u8>> {
    // Handle special file IDs
    if id == *EMPTY_ID {
        return Ok(Vec::new());
    }
    if id == *STDIN_ID {
        return read_stdin();
    }

    // Check virtual package first (VPS support)
    if let Some(spec) = id.package() {
        let pkg = super::vfs::PackageId::from_spec(spec);
        let path = id.vpath().as_rooted_path().to_string_lossy();
        if let Some(content) = virtual_fs.read_package(&pkg, &path) {
            record_file_access(id);
            return Ok(content);
        }
    }

    // Check virtual path (VFS support)
    let vpath = id.vpath().as_rooted_path();
    if let Some(data) = virtual_fs.read(vpath) {
        record_file_access(id);
        return Ok(data);
    }

    // Resolve and read from disk
    let path = resolve_path(project_root, id)?;
    read_disk(&path)
}

/// Decode bytes as UTF-8, stripping BOM if present.
pub fn decode_utf8(buf: &[u8]) -> FileResult<&str> {
    let buf = buf.strip_prefix(b"\xef\xbb\xbf").unwrap_or(buf);
    std::str::from_utf8(buf).map_err(|_| FileError::InvalidUtf8)
}



/// Resolve file path, downloading package if needed.
fn resolve_path(project_root: &Path, id: FileId) -> FileResult<std::path::PathBuf> {
    let root = id
        .package()
        .map(|spec| package::storage().prepare_package(spec, &mut SilentProgress))
        .transpose()?
        .unwrap_or_else(|| project_root.to_path_buf());

    id.vpath().resolve(&root).ok_or(FileError::AccessDenied)
}

/// Read file from disk.
fn read_disk(path: &Path) -> FileResult<Vec<u8>> {
    let map_err = |e| FileError::from_io(e, path);
    fs::metadata(path).map_err(map_err).and_then(|m| {
        if m.is_dir() {
            Err(FileError::IsDirectory)
        } else {
            fs::read(path).map_err(map_err)
        }
    })
}

/// Read all data from stdin.
fn read_stdin() -> FileResult<Vec<u8>> {
    let mut buf = Vec::new();
    io::stdin()
        .read_to_end(&mut buf)
        .or_else(|e| {
            if e.kind() == io::ErrorKind::BrokenPipe {
                Ok(0)
            } else {
                Err(FileError::from_io(e, Path::new("<stdin>")))
            }
        })?;
    Ok(buf)
}

/// No-op progress reporter for silent package downloads.
struct SilentProgress;

impl Progress for SilentProgress {
    fn print_start(&mut self) {}
    fn print_progress(&mut self, _: &DownloadState) {}
    fn print_finish(&mut self, _: &DownloadState) {}
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_decode_utf8_valid() {
        let text = "Hello, 世界!";
        assert_eq!(decode_utf8(text.as_bytes()).unwrap(), text);
    }

    #[test]
    fn test_decode_utf8_strips_bom() {
        let mut bytes = vec![0xef, 0xbb, 0xbf];
        bytes.extend_from_slice(b"Hello");
        assert_eq!(decode_utf8(&bytes).unwrap(), "Hello");
    }

    #[test]
    fn test_decode_utf8_invalid() {
        let invalid = vec![0xff, 0xfe];
        assert!(decode_utf8(&invalid).is_err());
    }

    #[test]
    fn test_read_disk() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, "test content").unwrap();

        assert_eq!(read_disk(&path).unwrap(), b"test content");
    }

    #[test]
    fn test_read_disk_directory() {
        let dir = TempDir::new().unwrap();
        assert!(read_disk(dir.path()).is_err());
    }

    #[test]
    fn test_read_disk_nonexistent() {
        assert!(read_disk(Path::new("/nonexistent/file.txt")).is_err());
    }

    #[test]
    fn test_empty_id() {
        let dir = TempDir::new().unwrap();
        let result = read_file(*EMPTY_ID, dir.path());
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
