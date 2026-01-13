//! HTML node wrapper.

use super::{HtmlElement, HtmlFrame};

/// An HTML node (element, text, or frame).
///
/// Use `kind()` to determine the node type and access its data.
#[derive(Debug, Clone, Copy)]
pub struct HtmlNode<'a>(pub(crate) &'a typst_html::HtmlNode);

/// The kind of an HTML node.
#[derive(Debug, Clone, Copy)]
pub enum NodeKind<'a> {
    /// An HTML element with tag, attributes, and children.
    Element(HtmlElement<'a>),
    /// Plain text content.
    Text(&'a str),
    /// A frame that should be rendered as SVG.
    Frame(HtmlFrame<'a>),
    /// An introspection tag (usually ignored during conversion).
    Tag,
}

impl<'a> HtmlNode<'a> {
    /// Get the kind of this node.
    ///
    /// # Example
    ///
    /// ```ignore
    /// match node.kind() {
    ///     NodeKind::Element(elem) => {
    ///         println!("Element: {}", elem.tag());
    ///     }
    ///     NodeKind::Text(text) => {
    ///         println!("Text: {}", text);
    ///     }
    ///     NodeKind::Frame(frame) => {
    ///         #[cfg(feature = "svg")]
    ///         let svg = frame.to_svg(&doc);
    ///     }
    ///     NodeKind::Tag => {
    ///         // Introspection tag, usually ignored
    ///     }
    /// }
    /// ```
    #[inline]
    pub fn kind(&self) -> NodeKind<'a> {
        match self.0 {
            typst_html::HtmlNode::Tag(_) => NodeKind::Tag,
            typst_html::HtmlNode::Text(text, _span) => NodeKind::Text(text.as_str()),
            typst_html::HtmlNode::Element(elem) => NodeKind::Element(HtmlElement(elem)),
            typst_html::HtmlNode::Frame(frame) => NodeKind::Frame(HtmlFrame(frame)),
        }
    }

    /// Try to get this node as an element.
    #[inline]
    pub fn as_element(&self) -> Option<HtmlElement<'a>> {
        match self.0 {
            typst_html::HtmlNode::Element(elem) => Some(HtmlElement(elem)),
            _ => None,
        }
    }

    /// Try to get this node as text.
    #[inline]
    pub fn as_text(&self) -> Option<&'a str> {
        match self.0 {
            typst_html::HtmlNode::Text(text, _) => Some(text.as_str()),
            _ => None,
        }
    }

    /// Try to get this node as a frame.
    #[inline]
    pub fn as_frame(&self) -> Option<HtmlFrame<'a>> {
        match self.0 {
            typst_html::HtmlNode::Frame(frame) => Some(HtmlFrame(frame)),
            _ => None,
        }
    }

    /// Check if this is an element node.
    #[inline]
    pub fn is_element(&self) -> bool {
        matches!(self.0, typst_html::HtmlNode::Element(_))
    }

    /// Check if this is a text node.
    #[inline]
    pub fn is_text(&self) -> bool {
        matches!(self.0, typst_html::HtmlNode::Text(_, _))
    }

    /// Check if this is a frame node.
    #[inline]
    pub fn is_frame(&self) -> bool {
        matches!(self.0, typst_html::HtmlNode::Frame(_))
    }

    /// Check if this is a tag node (introspection).
    #[inline]
    pub fn is_tag(&self) -> bool {
        matches!(self.0, typst_html::HtmlNode::Tag(_))
    }
}
