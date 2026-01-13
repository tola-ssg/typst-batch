//! High-level compilation API for Typst to HTML.
//!
//! # Example
//!
//! ```ignore
//! use typst_batch::Compiler;
//! use std::path::Path;
//!
//! // Simple compilation
//! let result = Compiler::new(Path::new("."))
//!     .with_path(Path::new("doc.typ"))
//!     .compile()?;
//!
//! // With sys.inputs
//! let result = Compiler::new(Path::new("."))
//!     .with_inputs([("title", "Hello")])
//!     .with_path(Path::new("doc.typ"))
//!     .compile()?;
//!
//! // Custom World (advanced)
//! let result = Compiler::new(Path::new("."))
//!     .with_path(Path::new("doc.typ"))
//!     .with_world(|main, root| {
//!         TypstWorld::builder(main, root)
//!             .with_local_cache()
//!             .with_fonts()
//!             .build()
//!     })
//!     .compile()?;
//!
//! // Batch compilation (requires `batch` feature)
//! let batcher = Compiler::new(Path::new("."))
//!     .into_batch()
//!     .with_snapshot_from(&[path1, path2, path3])?;
//!
//! let results = batcher.batch_compile(&[path1, path2, path3])?;
//! ```

use std::path::{Path, PathBuf};

use typst::foundations::Dict;

use crate::diagnostic::{filter_html_warnings, has_errors, CompileError, Diagnostics};
use crate::html::HtmlDocument;
use crate::world::TypstWorld;

use super::inputs::WithInputs;
use super::session::{AccessedDeps, CompileSession};
use crate::resource::file::PackageId;

/// Type alias for custom World builder function.
type WorldBuilderFn<'a> = Box<dyn FnOnce(MainPath<'_>, RootPath<'_>) -> TypstWorld + 'a>;



/// Wrapper for the main file path, ensuring type safety.
///
/// Users cannot construct this directly; it's only created internally
/// and passed to closures in `with_world()`.
pub struct MainPath<'a>(&'a Path);

impl<'a> MainPath<'a> {
    /// Get the underlying path.
    pub fn as_path(&self) -> &'a Path {
        self.0
    }
}

impl AsRef<Path> for MainPath<'_> {
    fn as_ref(&self) -> &Path {
        self.0
    }
}

/// Wrapper for the root directory path, ensuring type safety.
///
/// Users cannot construct this directly; it's only created internally
/// and passed to closures in `with_world()`.
pub struct RootPath<'a>(&'a Path);

impl<'a> RootPath<'a> {
    /// Get the underlying path.
    pub fn as_path(&self) -> &'a Path {
        self.0
    }
}

impl AsRef<Path> for RootPath<'_> {
    fn as_ref(&self) -> &Path {
        self.0
    }
}



/// Builder for Typst compilation.
///
/// This is the entry point for all compilation operations. Use method chaining
/// to configure and then either:
/// - Call `with_path()` for single-file compilation → [`SingleCompiler`]
/// - Call `into_batch()` for parallel batch compilation → [`BatchCompiler`]
///
/// # Example
///
/// ```ignore
/// // Single file
/// Compiler::new(root)
///     .with_inputs([("key", "value")])
///     .with_path(path)
///     .compile()?;
///
/// // Batch
/// Compiler::new(root)
///     .into_batch()
///     .with_snapshot_from(&files)?
///     .batch_compile(&files)?;
/// ```
pub struct Compiler<'a> {
    root: &'a Path,
    inputs: Option<Dict>,
    preludes: Vec<String>,
    postludes: Vec<String>,
}

impl<'a> WithInputs for Compiler<'a> {
    fn inputs_mut(&mut self) -> &mut Option<Dict> {
        &mut self.inputs
    }
}

impl<'a> Compiler<'a> {
    /// Create a new compiler with the given root directory.
    pub fn new(root: &'a Path) -> Self {
        Self {
            root,
            inputs: None,
            preludes: Vec::new(),
            postludes: Vec::new(),
        }
    }

    /// Add prelude code to inject at the beginning of the main file.
    pub fn with_prelude(mut self, prelude: impl Into<String>) -> Self {
        self.preludes.push(prelude.into());
        self
    }

    /// Add postlude code to inject at the end of the main file.
    pub fn with_postlude(mut self, postlude: impl Into<String>) -> Self {
        self.postludes.push(postlude.into());
        self
    }

    /// Set the file to compile, returning a [`SingleCompiler`].
    pub fn with_path<P: AsRef<Path>>(self, path: P) -> SingleCompiler<'a> {
        SingleCompiler {
            root: self.root,
            path: path.as_ref().to_path_buf(),
            inputs: self.inputs,
            preludes: self.preludes,
            postludes: self.postludes,
            world_builder: None,
        }
    }

    /// Convert to batch compilation mode.
    ///
    /// Returns a [`Batcher`](super::batch::Batcher) for parallel compilation with snapshot optimization.
    /// Any `with_inputs()` settings are inherited.
    ///
    /// **Note**: Batch mode uses lock-free snapshot caching internally.
    /// Custom `with_world()` settings from single-file mode do not apply.
    #[cfg(feature = "batch")]
    pub fn into_batch(self) -> super::batch::Batcher<'a> {
        let mut batcher = super::batch::Batcher::new(self.root);
        if let Some(inputs) = self.inputs {
            batcher = batcher.with_inputs_dict(inputs);
        }
        batcher.preludes = self.preludes;
        batcher.postludes = self.postludes;
        batcher
    }
}



/// Builder for single-file compilation.
///
/// Created via `Compiler::new(root).with_path(path)`.
///
/// # Custom World
///
/// By default, uses shared cache mode (suitable for serve/hot-reload).
/// For custom caching strategies, use `with_world()`:
///
/// ```ignore
/// Compiler::new(root)
///     .with_path(path)
///     .with_world(|main, root| {
///         TypstWorld::builder(main, root)
///             .with_local_cache()  // No shared state
///             .with_fonts()
///             .build()
///     })
///     .compile()?;
/// ```
pub struct SingleCompiler<'a> {
    root: &'a Path,
    path: PathBuf,
    inputs: Option<Dict>,
    preludes: Vec<String>,
    postludes: Vec<String>,
    world_builder: Option<WorldBuilderFn<'a>>,
}

impl<'a> WithInputs for SingleCompiler<'a> {
    fn inputs_mut(&mut self) -> &mut Option<Dict> {
        &mut self.inputs
    }
}

impl<'a> SingleCompiler<'a> {
    /// Add prelude code to inject at the beginning of the main file.
    pub fn with_prelude(mut self, prelude: impl Into<String>) -> Self {
        self.preludes.push(prelude.into());
        self
    }

    /// Add postlude code to inject at the end of the main file.
    pub fn with_postlude(mut self, postlude: impl Into<String>) -> Self {
        self.postludes.push(postlude.into());
        self
    }

    /// Provide a custom World builder.
    ///
    /// The closure receives type-safe path wrappers that can only be obtained
    /// through this API, ensuring the World's paths match the compiler's paths.
    ///
    /// # Example
    ///
    /// ```ignore
    /// Compiler::new(root)
    ///     .with_path(path)
    ///     .with_world(|main, root| {
    ///         TypstWorld::builder(main, root)
    ///             .with_local_cache()
    ///             .with_fonts()
    ///             .build()
    ///     })
    ///     .compile()?;
    /// ```
    pub fn with_world<F>(mut self, f: F) -> Self
    where
        F: FnOnce(MainPath<'_>, RootPath<'_>) -> TypstWorld + 'a,
    {
        self.world_builder = Some(Box::new(f));
        self
    }

    /// Compile the file.
    pub fn compile(self) -> Result<CompileResult, CompileError> {
        let world = match self.world_builder {
            Some(builder) => builder(MainPath(&self.path), RootPath(self.root)),
            None => self.default_world(),
        };
        compile_with_world(&world)
    }

    fn default_world(&self) -> TypstWorld {
        let mut builder = TypstWorld::builder(&self.path, self.root)
            .with_shared_cache()
            .with_fonts();

        if let Some(inputs) = &self.inputs {
            builder = builder.with_inputs_dict(inputs.clone());
        }

        // Build combined prelude: styles + scripts + user preludes
        let combined_prelude = self.build_prelude();
        if !combined_prelude.is_empty() {
            builder = builder.with_prelude(combined_prelude);
        }

        // Build combined postlude
        let combined_postlude = self.postludes.join("\n");
        if !combined_postlude.is_empty() {
            builder = builder.with_postlude(combined_postlude);
        }

        builder.build()
    }

    fn build_prelude(&self) -> String {
        self.preludes.join("\n")
    }
}



/// Result of a successful compilation.
#[derive(Debug)]
pub struct CompileResult {
    document: HtmlDocument,
    accessed: AccessedDeps,
    diagnostics: Diagnostics,
}

impl CompileResult {
    /// Get the compiled HTML document.
    pub fn document(&self) -> &HtmlDocument {
        &self.document
    }

    /// Convert the document to HTML bytes.
    pub fn html(&self) -> Result<Vec<u8>, CompileError> {
        typst_html::html(self.document.as_inner())
            .map(|s| s.into_bytes())
            .map_err(|e| CompileError::html_export(format!("{e:?}")))
    }

    /// Get files and packages accessed during compilation.
    pub fn accessed(&self) -> &AccessedDeps {
        &self.accessed
    }

    /// Get files accessed during compilation.
    pub fn accessed_files(&self) -> &[PathBuf] {
        &self.accessed.files
    }

    /// Get packages accessed during compilation.
    ///
    /// Useful for detecting virtual package usage (e.g., `@myapp/data`).
    pub fn accessed_packages(&self) -> &[PackageId] {
        &self.accessed.packages
    }

    /// Get compilation diagnostics (warnings).
    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }

    /// Take ownership of the document.
    pub fn into_document(self) -> HtmlDocument {
        self.document
    }

    /// Destructure into components.
    pub fn into_parts(self) -> (HtmlDocument, AccessedDeps, Diagnostics) {
        (self.document, self.accessed, self.diagnostics)
    }
}



pub(crate) fn compile_with_world(world: &TypstWorld) -> Result<CompileResult, CompileError> {
    let session = CompileSession::start();
    let line_offset = world.prelude_line_count();

    let result = typst::compile(world);

    if has_errors(&result.warnings) {
        return Err(CompileError::compilation_with_offset(world, result.warnings.to_vec(), line_offset));
    }

    let document = result.output.map_err(|errors| {
        let all_diags: Vec<_> = errors.iter().chain(&result.warnings).cloned().collect();
        let filtered = filter_html_warnings(&all_diags);
        CompileError::compilation_with_offset(world, filtered, line_offset)
    })?;

    let document = HtmlDocument::new(document);

    let accessed = session.finish(world.root());
    let filtered_warnings = filter_html_warnings(&result.warnings);
    let diagnostics = Diagnostics::resolve_with_offset(world, &filtered_warnings, line_offset);

    Ok(CompileResult {
        document,
        accessed,
        diagnostics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_simple_compile() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.typ");
        fs::write(&file, "= Hello World").unwrap();

        let result = Compiler::new(dir.path()).with_path(&file).compile();
        assert!(result.is_ok());

        let html = result.unwrap().html().unwrap();
        assert!(String::from_utf8_lossy(&html).contains("Hello World"));
    }

    #[test]
    fn test_compile_with_inputs() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.typ");
        fs::write(
            &file,
            r#"#let title = sys.inputs.at("title", default: "Default")
= #title"#,
        )
        .unwrap();

        let result = Compiler::new(dir.path())
            .with_inputs([("title", "Custom Title")])
            .with_path(&file)
            .compile();

        assert!(result.is_ok());
        let html = result.unwrap().html().unwrap();
        assert!(String::from_utf8_lossy(&html).contains("Custom Title"));
    }

    #[test]
    fn test_query_metadata() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.typ");
        fs::write(
            &file,
            r#"#metadata((title: "Test", draft: false)) <post-meta>
= Content"#,
        )
        .unwrap();

        let result = Compiler::new(dir.path())
            .with_path(&file)
            .compile()
            .unwrap();
        let meta = result.document().query_metadata("post-meta");

        assert!(meta.is_some());
        let meta = meta.unwrap();
        assert_eq!(
            meta.get("title")
                .and_then(|v: &serde_json::Value| v.as_str()),
            Some("Test")
        );
    }

    #[test]
    #[cfg(feature = "batch")]
    fn test_batch_compile() {
        let dir = TempDir::new().unwrap();

        let file1 = dir.path().join("test1.typ");
        let file2 = dir.path().join("test2.typ");
        fs::write(&file1, "= File One").unwrap();
        fs::write(&file2, "= File Two").unwrap();

        let results = Compiler::new(dir.path())
            .into_batch()
            .batch_compile(&[&file1, &file2])
            .unwrap();

        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());

        let html1 = results[0].as_ref().unwrap().html().unwrap();
        let html2 = results[1].as_ref().unwrap().html().unwrap();
        assert!(String::from_utf8_lossy(&html1).contains("File One"));
        assert!(String::from_utf8_lossy(&html2).contains("File Two"));
    }

    #[test]
    #[cfg(feature = "batch")]
    fn test_batch_with_snapshot_from() {
        let dir = TempDir::new().unwrap();

        let file1 = dir.path().join("test1.typ");
        let file2 = dir.path().join("test2.typ");
        fs::write(&file1, "= File One").unwrap();
        fs::write(&file2, "= File Two").unwrap();

        let batch = Compiler::new(dir.path())
            .into_batch()
            .with_snapshot_from(&[&file1, &file2])
            .unwrap();

        // First compile
        let results1 = batch.batch_compile(&[&file1, &file2]).unwrap();
        assert_eq!(results1.len(), 2);

        // Second compile (reuses snapshot)
        let results2 = batch.batch_compile(&[&file1]).unwrap();
        assert_eq!(results2.len(), 1);
    }
}
