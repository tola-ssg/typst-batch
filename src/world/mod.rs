//! World implementations for Typst compilation.

mod builder;
mod cache;
mod core;
mod path;
mod snapshot;
mod strategy;

pub use builder::WorldBuilder;
pub use cache::{clear_thread_local_cache, LocalCache};
pub use core::TypstWorld;
pub use path::normalize_path;
pub use snapshot::{FileSnapshot, SnapshotConfig, SnapshotError};
pub use strategy::{CacheStrategy, FontStrategy, LibraryStrategy};
