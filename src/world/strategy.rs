//! Compilation strategies for cache, fonts, and library.

use std::sync::Arc;

use typst::utils::LazyHash;
use typst::Library;

use super::cache::LocalCache;
use super::snapshot::FileSnapshot;
use crate::resource::library::create_library_with_inputs;

/// Cache strategy for file access.
pub enum CacheStrategy {
    /// Task-local cache, no sharing between tasks.
    Local(LocalCache),
    /// Global shared cache with RwLock.
    Shared,
    /// Pre-built snapshot + thread-local extension.
    Snapshot(Arc<FileSnapshot>),
}

impl CacheStrategy {
    /// Creates a local cache strategy with a fresh cache.
    pub fn local() -> Self {
        Self::Local(LocalCache::new())
    }

    /// Creates a shared cache strategy using the global cache.
    pub fn shared() -> Self {
        Self::Shared
    }

    /// Creates a snapshot cache strategy from a pre-built snapshot.
    pub fn snapshot(snapshot: Arc<FileSnapshot>) -> Self {
        Self::Snapshot(snapshot)
    }
}

/// Font strategy for compilation.
#[derive(Clone, Copy)]
pub enum FontStrategy {
    /// No fonts loaded (for scan/query).
    None,
    /// Shared fonts from global cache.
    Shared,
}

/// Library strategy for sys.inputs.
#[derive(Clone)]
pub enum LibraryStrategy {
    /// Use global library (no sys.inputs).
    Global,
    /// Custom library with sys.inputs.
    Custom(LazyHash<Library>),
}

impl LibraryStrategy {
    /// Creates a custom library strategy with the given sys.inputs.
    pub fn with_inputs(inputs: typst::foundations::Dict) -> Self {
        Self::Custom(create_library_with_inputs(inputs))
    }
}
