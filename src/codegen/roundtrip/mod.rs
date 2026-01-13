//! Roundtrip tests for JSON ↔ Content conversion.
//!
//! Tests verify that:
//! - JSON → Content → JSON produces semantically equivalent JSON
//! - Content → JSON → Content → JSON produces identical JSON
//! - Various element types are correctly handled

#[cfg(test)]
pub(crate) mod common;

#[cfg(test)]
mod complex;

#[cfg(test)]
mod edge;

#[cfg(test)]
mod math;

#[cfg(test)]
mod model;

#[cfg(test)]
mod primitive;

#[cfg(test)]
mod value;
