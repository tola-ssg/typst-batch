//! Global package storage with caching.

use std::sync::OnceLock;

use typst_kit::download::Downloader;
pub use typst_kit::package::PackageStorage;

/// Options for package storage initialization.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// User-Agent string for package downloads from the Typst registry.
    ///
    /// Default: "typst-batch/{version}"
    pub user_agent: Option<String>,
}

impl Options {
    /// Create new options with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the User-Agent string.
    pub fn with_user_agent(mut self, agent: impl Into<String>) -> Self {
        self.user_agent = Some(agent.into());
        self
    }

    fn user_agent_or_default(&self) -> String {
        self.user_agent
            .clone()
            .unwrap_or_else(|| concat!("typst-batch/", env!("CARGO_PKG_VERSION")).to_string())
    }
}

/// Global shared package storage.
static STORAGE: OnceLock<PackageStorage> = OnceLock::new();

/// Initialize package storage with default settings.
///
/// This can only be called once. Subsequent calls are ignored.
/// Returns `true` if storage was initialized, `false` if already initialized.
pub fn init() -> bool {
    init_with_options(Options::default())
}

/// Initialize package storage with custom options.
///
/// This can only be called once. Subsequent calls are ignored.
/// Returns `true` if storage was initialized, `false` if already initialized.
///
/// # Example
///
/// ```ignore
/// use typst_batch::resource::package;
///
/// package::init_with_options(package::Options {
///     user_agent: Some("my-app/1.0.0".into()),
/// });
/// ```
pub fn init_with_options(options: Options) -> bool {
    STORAGE
        .set(PackageStorage::new(
            None, // Use default cache path
            None, // Use default package path
            Downloader::new(options.user_agent_or_default()),
        ))
        .is_ok()
}

/// Get the global package storage.
///
/// If not explicitly initialized, uses default settings on first access.
pub fn storage() -> &'static PackageStorage {
    STORAGE.get_or_init(|| {
        PackageStorage::new(
            None,
            None,
            Downloader::new(Options::default().user_agent_or_default()),
        )
    })
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options_default() {
        let opts = Options::default();
        assert!(opts.user_agent.is_none());
        assert!(opts.user_agent_or_default().starts_with("typst-batch/"));
    }

    #[test]
    fn test_options_with_user_agent() {
        let opts = Options::new().with_user_agent("test/1.0");
        assert_eq!(opts.user_agent, Some("test/1.0".to_string()));
        assert_eq!(opts.user_agent_or_default(), "test/1.0");
    }

    #[test]
    fn test_storage_initialized() {
        let _storage = storage();
    }

    #[test]
    fn test_storage_is_shared() {
        let storage1 = storage();
        let storage2 = storage();
        assert!(std::ptr::eq(storage1, storage2), "Storage should be shared");
    }
}
