//! HTML element wrapper.

use super::node::HtmlNode;

/// An HTML element.
///
/// Provides stable access to element tag, attributes, and children
/// without exposing internal typst types like `PicoStr` or `EcoVec`.
#[derive(Debug, Clone, Copy)]
pub struct HtmlElement<'a>(pub(crate) &'a typst_html::HtmlElement);

impl<'a> HtmlElement<'a> {
    /// Get the element's tag name as an owned String.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let tag = elem.tag();
    /// assert_eq!(tag, "div");
    /// ```
    #[inline]
    pub fn tag(&self) -> String {
        self.0.tag.resolve().to_string()
    }

    /// Iterate over the element's attributes as owned (String, String) pairs.
    ///
    /// # Example
    ///
    /// ```ignore
    /// for (key, value) in elem.attrs() {
    ///     println!("{}: {}", key, value);
    /// }
    /// ```
    #[inline]
    pub fn attrs(&self) -> impl Iterator<Item = (String, String)> + '_ {
        self.0
            .attrs
            .0
            .iter()
            .map(|(k, v)| (k.resolve().to_string(), v.to_string()))
    }

    /// Collect attributes into a Vec for convenience.
    ///
    /// This is useful when you need to iterate multiple times or store the attributes.
    #[inline]
    pub fn attrs_vec(&self) -> Vec<(String, String)> {
        self.attrs().collect()
    }

    /// Get an attribute value by name.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(class) = elem.get_attr("class") {
    ///     println!("class: {}", class);
    /// }
    /// ```
    pub fn get_attr(&self, name: &str) -> Option<String> {
        self.0.attrs.0.iter().find_map(|(k, v)| {
            if k.resolve().as_str() == name {
                Some(v.to_string())
            } else {
                None
            }
        })
    }

    /// Check if the element has an attribute.
    #[inline]
    pub fn has_attr(&self, name: &str) -> bool {
        self.0
            .attrs
            .0
            .iter()
            .any(|(k, _)| k.resolve().as_str() == name)
    }

    /// Get the element's id attribute.
    #[inline]
    pub fn id(&self) -> Option<String> {
        self.get_attr("id")
    }

    /// Get the element's class attribute.
    #[inline]
    pub fn class(&self) -> Option<String> {
        self.get_attr("class")
    }

    /// Iterate over the element's children.
    ///
    /// # Example
    ///
    /// ```ignore
    /// for child in elem.children() {
    ///     match child.kind() {
    ///         NodeKind::Element(e) => println!("Child element: {}", e.tag()),
    ///         NodeKind::Text(t) => println!("Text: {}", t),
    ///         _ => {}
    ///     }
    /// }
    /// ```
    #[inline]
    pub fn children(&self) -> impl Iterator<Item = HtmlNode<'a>> {
        self.0.children.iter().map(HtmlNode)
    }

    /// Get the number of children.
    #[inline]
    pub fn children_count(&self) -> usize {
        self.0.children.len()
    }

    /// Check if the element has no children.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.children.is_empty()
    }
}
