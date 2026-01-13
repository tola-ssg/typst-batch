//! Builder pattern for `TypstWorld`.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use typst::foundations::Dict;

use super::core::TypstWorld;
use super::snapshot::FileSnapshot;
use super::strategy::{CacheStrategy, FontStrategy, LibraryStrategy};
use crate::resource::library::create_library_with_inputs;

/// Builder for configuring `TypstWorld`.
///
/// Use `TypstWorld::builder()` to create a builder.
pub struct WorldBuilder {
    main_path: PathBuf,
    root: PathBuf,
    cache: Option<CacheStrategy>,
    fonts: Option<FontStrategy>,
    library: LibraryStrategy,
    prelude: Option<String>,
    postlude: Option<String>,
}

impl WorldBuilder {
    /// Create a new builder.
    pub(crate) fn new(main_path: &Path, root: &Path) -> Self {
        Self {
            main_path: main_path.to_path_buf(),
            root: root.to_path_buf(),
            cache: None,
            fonts: None,
            library: LibraryStrategy::Global,
            prelude: None,
            postlude: None,
        }
    }

    // =========================================================================
    // Cache Strategy
    // =========================================================================

    /// Use task-local cache (no sharing between compilations).
    ///
    /// Best for: isolated compilations, scanning operations.
    pub fn with_local_cache(mut self) -> Self {
        self.cache = Some(CacheStrategy::local());
        self
    }

    /// Use global shared cache with lock-based synchronization.
    ///
    /// Best for: hot reload, incremental updates where files change frequently.
    pub fn with_shared_cache(mut self) -> Self {
        self.cache = Some(CacheStrategy::shared());
        self
    }

    /// Use pre-built immutable snapshot for lock-free parallel access.
    ///
    /// Best for: batch compilation where files are pre-scanned.
    pub fn with_snapshot(mut self, snapshot: Arc<FileSnapshot>) -> Self {
        self.cache = Some(CacheStrategy::snapshot(snapshot));
        self
    }

    // =========================================================================
    // Font Strategy
    // =========================================================================

    /// Disable font loading.
    ///
    /// Best for: scanning/query operations that don't require layout.
    pub fn no_fonts(mut self) -> Self {
        self.fonts = Some(FontStrategy::None);
        self
    }

    /// Use shared font cache.
    ///
    /// Best for: compilation operations that require layout/rendering.
    pub fn with_fonts(mut self) -> Self {
        self.fonts = Some(FontStrategy::Shared);
        self
    }

    /// Configure `sys.inputs` for the compilation.
    pub fn with_inputs<I, K, V>(mut self, inputs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<typst::foundations::Str>,
        V: typst::foundations::IntoValue,
    {
        let dict: Dict = inputs
            .into_iter()
            .map(|(k, v)| (k.into(), v.into_value()))
            .collect();
        self.library = LibraryStrategy::Custom(create_library_with_inputs(dict));
        self
    }

    /// Configure `sys.inputs` from a pre-built `Dict`.
    pub fn with_inputs_dict(mut self, inputs: Dict) -> Self {
        self.library = LibraryStrategy::Custom(create_library_with_inputs(inputs));
        self
    }

    // =========================================================================
    // Prelude
    // =========================================================================

    /// Set Typst code to prepend to the main file.
    ///
    /// The prelude is injected at the beginning of the main source file
    /// before compilation. Useful for injecting show rules, imports, etc.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let world = TypstWorld::builder(path, root)
    ///     .with_shared_cache()
    ///     .with_fonts()
    ///     .with_prelude(r#"
    ///         #show math.equation: eq => html.frame(eq)
    ///     "#)
    ///     .build();
    /// ```
    pub fn with_prelude(mut self, prelude: impl Into<String>) -> Self {
        self.prelude = Some(prelude.into());
        self
    }

    /// Set Typst code to append to the main file.
    ///
    /// The postlude is injected at the end of the main source file
    /// before compilation. Useful for injecting query operations, cleanup, etc.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let world = TypstWorld::builder(path, root)
    ///     .with_shared_cache()
    ///     .with_fonts()
    ///     .with_postlude(r#"
    ///         #context {
    ///             let eqs = query(math.equation)
    ///             // Process equations...
    ///         }
    ///     "#)
    ///     .build();
    /// ```
    pub fn with_postlude(mut self, postlude: impl Into<String>) -> Self {
        self.postlude = Some(postlude.into());
        self
    }

    /// Build the `TypstWorld`.
    ///
    /// # Panics
    ///
    /// Panics if cache or fonts strategy is not set.
    pub fn build(self) -> TypstWorld {
        let cache = self.cache.expect("cache strategy must be set");
        let fonts = self.fonts.expect("fonts strategy must be set");
        TypstWorld::new(&self.main_path, &self.root, cache, fonts, self.library, self.prelude, self.postlude)
    }
}
