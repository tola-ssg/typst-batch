//! Thread-local file access tracking.
//!
//! Tracks which files are accessed during compilation for dependency analysis.

use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};

use rustc_hash::FxHashSet;
use typst::syntax::FileId;

// =============================================================================
// Generation Counter
// =============================================================================

/// Global generation counter for cache invalidation.
///
/// Instead of iterating through all FileSlots to reset access flags (O(n)),
/// we increment this counter (O(1)). Each SlotCell compares its last-access
/// generation against the current generation to determine if it was accessed
/// in the current compilation.
static GENERATION: AtomicU64 = AtomicU64::new(1);

thread_local! {
    /// Thread-local set of accessed file IDs for the current compilation.
    static ACCESSED_FILES: RefCell<FxHashSet<FileId>> = RefCell::new(FxHashSet::default());

    /// Thread-local generation snapshot for the current compilation.
    static CURRENT_GENERATION: RefCell<u64> = const { RefCell::new(0) };
}

// =============================================================================
// Public API
// =============================================================================

/// Get the current generation for this thread's compilation.
pub(crate) fn current_generation() -> u64 {
    CURRENT_GENERATION.with(|g| *g.borrow())
}

/// Clear the thread-local accessed files set and advance the generation counter.
///
/// Call at the start of each file compilation. This is O(1) instead of O(n)
/// because we use a generation counter instead of iterating through all slots.
pub fn reset_access_flags() {
    // Advance global generation
    let new_gen = GENERATION.fetch_add(1, Ordering::Relaxed) + 1;

    // Capture generation for this thread
    CURRENT_GENERATION.with(|current| {
        *current.borrow_mut() = new_gen;
    });

    // Reset thread-local tracking
    ACCESSED_FILES.with(|files| files.borrow_mut().clear());
}

/// Record a file access in the thread-local set.
pub fn record_file_access(id: FileId) {
    ACCESSED_FILES.with(|files| {
        files.borrow_mut().insert(id);
    });
}

/// Get all files accessed during the current compilation.
///
/// Returns a list of `FileId`s that were accessed since last `reset_access_flags()`.
/// Thread-safe: each thread has its own tracking.
pub fn get_accessed_files() -> Vec<FileId> {
    ACCESSED_FILES.with(|files| files.borrow().iter().copied().collect())
}
