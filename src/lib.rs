//! # typst-batch
//!
//! Typst batch compilation library with shared resources.
//!
//! ## Features
//!
//! - Shared fonts, packages, and file cache across compilations
//! - Fast scanning API (skip Layout phase, 5-20x faster)
//! - JSON ↔ Typst Content bidirectional conversion
//! - Colored diagnostics
//!
//! ## Modules
//!
//! - [`process`] - Compile and scan APIs
//! - [`codegen`] - JSON ↔ Typst conversion
//! - [`resource`] - Shared resources (font, package, file, library)
//! - [`world`] - Typst World implementation
//! - [`diagnostic`] - Error formatting
//! - [`html`] - HTML document utilities

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod codegen;
pub mod diagnostic;
pub mod html;
pub mod prelude;
pub mod process;
pub mod resource;
pub mod world;

// Re-export prelude at crate root for `typst_batch::xxx` access
pub use prelude::*;
