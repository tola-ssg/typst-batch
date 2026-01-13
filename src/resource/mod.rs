//! Shared resources for Typst compilation (fonts, packages, file cache).

pub mod file;
pub mod font;
pub mod library;
pub mod package;

use std::path::Path;
use std::sync::LazyLock;

/// Initialize all global resources at startup.
pub fn warmup(font_dirs: &[&Path]) {
    // Initialize fonts
    font::get_fonts(font_dirs);

    // Initialize library
    LazyLock::force(&library::GLOBAL_LIBRARY);

    // Initialize package storage
    package::storage();

    // Initialize file cache
    LazyLock::force(&file::GLOBAL_FILE_CACHE);
}
