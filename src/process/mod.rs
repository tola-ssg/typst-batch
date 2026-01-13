//! Document processing pipeline.
//!
//! - [`Compiler`] - Builder-based compilation API
//! - [`Batcher`] - Batch compilation API for parallel processing
//! - [`Scanner`] - Builder-based scanning API (Eval only, skips Layout)

mod common;
mod inputs;
mod session;
pub mod compile;
#[cfg(feature = "batch")]
pub mod batch;
#[cfg(feature = "scan")]
pub mod scan;

pub use inputs::WithInputs;
pub use session::{AccessedDeps, CompileSession};

#[cfg(feature = "batch")]
pub use batch::{Batcher, BatchScanner};
