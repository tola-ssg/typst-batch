//! HTML document wrapper.

use typst::foundations::{Label, Selector};
use typst::introspection::MetadataElem;
use typst::utils::PicoStr;

use super::HtmlElement;
#[cfg(feature = "svg")]
use super::HtmlFrame;

#[cfg(all(feature = "svg", feature = "batch"))]
use rayon::prelude::*;

/// A compiled HTML document.
///
/// This is a stable wrapper around the internal typst HTML document type.
#[derive(Debug, Clone)]
pub struct HtmlDocument(pub(crate) typst_html::HtmlDocument);

impl HtmlDocument {
    /// Create a new HtmlDocument from a typst HtmlDocument.
    #[inline]
    pub fn new(doc: typst_html::HtmlDocument) -> Self {
        Self(doc)
    }

    /// Get the root element of the document.
    #[inline]
    pub fn root(&self) -> HtmlElement<'_> {
        HtmlElement(&self.0.root)
    }

    /// Query metadata by label name.
    ///
    /// In Typst: `#metadata((title: "Hello")) <my-meta>`
    ///
    /// ```ignore
    /// let meta = doc.query_metadata("my-meta");
    /// ```
    pub fn query_metadata(&self, label: &str) -> Option<serde_json::Value> {
        let label = Label::new(PicoStr::intern(label))?;
        let elem = self.0.introspector.query_unique(&Selector::Label(label)).ok()?;
        elem.to_packed::<MetadataElem>()
            .and_then(|meta| serde_json::to_value(&meta.value).ok())
    }

    /// Render a frame to SVG.
    #[cfg(feature = "svg")]
    pub(crate) fn render_frame_svg(&self, frame: &HtmlFrame<'_>) -> String {
        self.render_raw_frame_svg(frame.0)
    }

    /// Render a frame to SVG (internal, takes raw typst frame).
    #[cfg(feature = "svg")]
    #[inline]
    fn render_raw_frame_svg(&self, frame: &typst_html::HtmlFrame) -> String {
        typst_svg::svg_html_frame(
            &frame.inner,
            frame.text_size,
            frame.id.as_deref(),
            &frame.link_points,
            &self.0.introspector,
        )
    }

    /// Render multiple frames to SVG.
    ///
    /// When the `batch` feature is enabled, frames are rendered in parallel
    /// using rayon. Otherwise, they are rendered sequentially.
    ///
    /// This is the recommended way to render multiple frames, as it
    /// automatically uses the best strategy based on available features.
    ///
    /// # Arguments
    ///
    /// * `frames` - Slice of frames to render
    ///
    /// # Returns
    ///
    /// Vector of SVG strings in the same order as input frames.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let frames: Vec<_> = collect_frames(&doc);
    /// let svgs = doc.render_frames(&frames);
    /// ```
    #[cfg(feature = "svg")]
    pub fn render_frames(&self, frames: &[HtmlFrame<'_>]) -> Vec<String> {
        #[cfg(feature = "batch")]
        {
            frames
                .par_iter()
                .map(|frame| self.render_raw_frame_svg(frame.0))
                .collect()
        }

        #[cfg(not(feature = "batch"))]
        {
            frames
                .iter()
                .map(|frame| self.render_raw_frame_svg(frame.0))
                .collect()
        }
    }

    /// Get the inner typst document.
    #[inline]
    pub fn into_inner(self) -> typst_html::HtmlDocument {
        self.0
    }

    /// Get a reference to the inner typst document.
    #[inline]
    pub fn as_inner(&self) -> &typst_html::HtmlDocument {
        &self.0
    }
}

impl From<typst_html::HtmlDocument> for HtmlDocument {
    fn from(doc: typst_html::HtmlDocument) -> Self {
        Self::new(doc)
    }
}
