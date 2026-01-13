//! Stable HTML document wrappers for traversal.

mod document;
mod element;
mod frame;
mod node;

pub use document::HtmlDocument;
pub use element::HtmlElement;
pub use frame::HtmlFrame;
pub use node::{HtmlNode, NodeKind};
