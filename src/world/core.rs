//! Unified `TypstWorld` implementation.
//!
//! A single World implementation with configurable strategies for:
//! - **Cache**: Local (task-local), Shared (global RwLock), Snapshot (pre-built + thread_local)
//! - **Fonts**: None (scan/query), Shared (build/serve)
//! - **Library**: Global or Custom (with sys.inputs)
//!
//! # Usage
//!
//! ## Builder Pattern (explicit configuration)
//!
//! ```ignore
//! // Scan: no fonts, local cache
//! let world = TypstWorld::builder(path, root)
//!     .with_local_cache()
//!     .no_fonts()
//!     .build();
//!
//! // Build: with fonts, snapshot cache
//! let world = TypstWorld::builder(path, root)
//!     .with_snapshot(snapshot)
//!     .with_fonts()
//!     .build();
//!
//! // Serve: with fonts, shared cache
//! let world = TypstWorld::builder(path, root)
//!     .with_shared_cache()
//!     .with_fonts()
//!     .build();
//!
//! // With sys.inputs
//! let world = TypstWorld::builder(path, root)
//!     .with_local_cache()
//!     .no_fonts()
//!     .with_inputs([("key", "value")])
//!     .build();
//! ```

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use chrono::{DateTime, Datelike, FixedOffset, Local, Utc};
use typst::diag::FileResult;
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, World};

use super::builder::WorldBuilder;
use super::cache::{THREAD_LOCAL_FILES, THREAD_LOCAL_SOURCES};
use super::path::normalize_path;
use super::strategy::{CacheStrategy, FontStrategy, LibraryStrategy};
use crate::resource::file::{
    decode_utf8, file_id_from_path, read_with_global_virtual, record_file_access, FileSlot,
    GLOBAL_FILE_CACHE,
};
use crate::resource::font::get_fonts;
use crate::resource::library::GLOBAL_LIBRARY;

// =============================================================================
// Empty FontBook (for scan/query)
// =============================================================================

static EMPTY_FONTBOOK: OnceLock<LazyHash<FontBook>> = OnceLock::new();

fn empty_fontbook() -> &'static LazyHash<FontBook> {
    EMPTY_FONTBOOK.get_or_init(|| LazyHash::new(FontBook::new()))
}

// =============================================================================
// TypstWorld
// =============================================================================

/// Fixed timestamp for reproducible builds.
///
/// If set, `datetime.today()` returns this fixed time.
/// If not set, `datetime.today()` returns `None`.
pub type Timestamp = DateTime<Utc>;

/// Unified Typst World with configurable strategies.
///
/// Use `TypstWorld::builder()` for explicit configuration,
/// or convenience methods like `for_scan()`, `for_build()`, `for_serve()`.
pub struct TypstWorld {
    root: PathBuf,
    main: FileId,
    cache: CacheStrategy,
    fonts: FontStrategy,
    library: LibraryStrategy,
    prelude: Option<String>,
    postlude: Option<String>,
    timestamp: Option<Timestamp>,
}

impl TypstWorld {
    /// Create a builder for explicit configuration.
    pub fn builder(main_path: &Path, root: &Path) -> WorldBuilder {
        WorldBuilder::new(main_path, root)
    }

    // =========================================================================
    // Internal Constructor
    // =========================================================================

    pub(crate) fn new(
        main_path: &Path,
        root: &Path,
        cache: CacheStrategy,
        fonts: FontStrategy,
        library: LibraryStrategy,
        prelude: Option<String>,
        postlude: Option<String>,
        timestamp: Option<Timestamp>,
    ) -> Self {
        let root = normalize_path(root);
        let main_abs = normalize_path(main_path);
        let main = file_id_from_path(&main_abs, &root).unwrap_or_else(|| {
            // Fallback: use filename only if path is outside root
            let filename = main_path.file_name().unwrap_or_default();
            FileId::new(None, VirtualPath::new(filename))
        });

        Self {
            root,
            main,
            cache,
            fonts,
            library,
            prelude,
            postlude,
            timestamp,
        }
    }

    /// Get the project root directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Get the number of lines in the prelude (for diagnostic line offset).
    ///
    /// Returns 0 if no prelude is set. The returned count includes the
    /// trailing newline that is added after the prelude during injection.
    pub fn prelude_line_count(&self) -> usize {
        self.prelude
            .as_ref()
            .map(|p| p.matches('\n').count() + 1) // +1 for the trailing newline added during injection
            .unwrap_or(0)
    }

    // =========================================================================
    // Cache Operations
    // =========================================================================

    fn get_source(&self, id: FileId) -> FileResult<Source> {
        match &self.cache {
            CacheStrategy::Local(local) => {
                if let Some(source) = local.sources.read().unwrap().get(&id) {
                    return Ok(source.clone());
                }
                let source = self.load_source(id)?;
                local.sources.write().unwrap().insert(id, source.clone());
                Ok(source)
            }
            CacheStrategy::Shared => {
                // For main file with prelude/postlude, use load_source to inject them
                // (global cache doesn't know about per-world prelude settings)
                if id == self.main && (self.prelude.is_some() || self.postlude.is_some()) {
                    return self.load_source(id);
                }
                let mut cache = GLOBAL_FILE_CACHE.write();
                let slot = cache.entry(id).or_insert_with(|| FileSlot::new(id));
                slot.source_with_global_virtual(&self.root)
            }
            CacheStrategy::Snapshot(snapshot) => {
                if let Some(source) = snapshot.get_source(id) {
                    record_file_access(id);
                    return Ok(source);
                }
                let local_hit =
                    THREAD_LOCAL_SOURCES.with(|c| c.borrow().get(&id).cloned());
                if let Some(source) = local_hit {
                    return Ok(source);
                }
                let source = self.load_source(id)?;
                THREAD_LOCAL_SOURCES.with(|c| c.borrow_mut().insert(id, source.clone()));
                Ok(source)
            }
        }
    }

    fn get_file(&self, id: FileId) -> FileResult<Bytes> {
        match &self.cache {
            CacheStrategy::Local(local) => {
                if let Some(bytes) = local.files.read().unwrap().get(&id) {
                    return Ok(bytes.clone());
                }
                let bytes = self.load_file(id)?;
                local.files.write().unwrap().insert(id, bytes.clone());
                Ok(bytes)
            }
            CacheStrategy::Shared => {
                let mut cache = GLOBAL_FILE_CACHE.write();
                let slot = cache.entry(id).or_insert_with(|| FileSlot::new(id));
                slot.file_with_global_virtual(&self.root)
            }
            CacheStrategy::Snapshot(snapshot) => {
                if let Some(bytes) = snapshot.get_file(id) {
                    record_file_access(id);
                    return Ok(bytes);
                }
                let local_hit = THREAD_LOCAL_FILES.with(|c| c.borrow().get(&id).cloned());
                if let Some(bytes) = local_hit {
                    return Ok(bytes);
                }
                let bytes = self.load_file(id)?;
                THREAD_LOCAL_FILES.with(|c| c.borrow_mut().insert(id, bytes.clone()));
                Ok(bytes)
            }
        }
    }

    fn load_source(&self, id: FileId) -> FileResult<Source> {
        record_file_access(id);
        let bytes = read_with_global_virtual(id, &self.root)?;
        let text = decode_utf8(&bytes)?;

        // Inject prelude/postlude for main file (fallback for non-snapshot usage)
        let text = if id == self.main {
            let mut result = String::new();
            if let Some(prelude) = &self.prelude {
                result.push_str(prelude);
                result.push('\n');
            }
            result.push_str(text);
            if let Some(postlude) = &self.postlude {
                result.push('\n');
                result.push_str(postlude);
            }
            result
        } else {
            text.into()
        };

        Ok(Source::new(id, text))
    }

    fn load_file(&self, id: FileId) -> FileResult<Bytes> {
        record_file_access(id);
        let data = read_with_global_virtual(id, &self.root)?;
        Ok(Bytes::new(data))
    }
}

// =============================================================================
// World Trait Implementation
// =============================================================================

impl World for TypstWorld {
    fn library(&self) -> &LazyHash<Library> {
        match &self.library {
            LibraryStrategy::Global => &GLOBAL_LIBRARY,
            LibraryStrategy::Custom(lib) => lib,
        }
    }

    fn book(&self) -> &LazyHash<FontBook> {
        match self.fonts {
            FontStrategy::None => empty_fontbook(),
            FontStrategy::Shared => &get_fonts(&[]).1,
        }
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        self.get_source(id)
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.get_file(id)
    }

    fn font(&self, index: usize) -> Option<Font> {
        match self.fonts {
            FontStrategy::None => None,
            FontStrategy::Shared => get_fonts(&[]).0.fonts.get(index)?.get(),
        }
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        // Return None if no timestamp is set (for reproducible builds)
        let now = self.timestamp.as_ref()?;

        let with_offset = match offset {
            None => now.with_timezone(&Local).fixed_offset(),
            Some(hours) => {
                let seconds = i32::try_from(hours).ok()?.checked_mul(3600)?;
                now.with_timezone(&FixedOffset::east_opt(seconds)?)
            }
        };

        Datetime::from_ymd(
            with_offset.year(),
            with_offset.month().try_into().ok()?,
            with_offset.day().try_into().ok()?,
        )
    }
}
