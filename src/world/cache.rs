//! Cache types for file and source storage.

use std::cell::RefCell;
use std::sync::RwLock;

use rustc_hash::FxHashMap;
use typst::foundations::Bytes;
use typst::syntax::{FileId, Source};

// ============================================================================
// Local Cache
// ============================================================================

/// Task-local cache storage.
pub struct LocalCache {
    pub(crate) sources: RwLock<FxHashMap<FileId, Source>>,
    pub(crate) files: RwLock<FxHashMap<FileId, Bytes>>,
}

impl LocalCache {
    /// Creates a new empty local cache.
    pub fn new() -> Self {
        Self {
            sources: RwLock::new(FxHashMap::default()),
            files: RwLock::new(FxHashMap::default()),
        }
    }
}

impl Default for LocalCache {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Thread-Local Extension Cache
// ============================================================================

thread_local! {
    pub(crate) static THREAD_LOCAL_SOURCES: RefCell<FxHashMap<FileId, Source>> =
        RefCell::new(FxHashMap::default());
    pub(crate) static THREAD_LOCAL_FILES: RefCell<FxHashMap<FileId, Bytes>> =
        RefCell::new(FxHashMap::default());
}

/// Clear thread-local extension caches.
pub fn clear_thread_local_cache() {
    THREAD_LOCAL_SOURCES.with(|c| c.borrow_mut().clear());
    THREAD_LOCAL_FILES.with(|c| c.borrow_mut().clear());
}
