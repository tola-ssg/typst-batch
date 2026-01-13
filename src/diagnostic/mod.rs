//! Diagnostic formatting for Typst compilation errors and warnings.

mod error;
mod filter;
mod format;
mod info;

// Re-export all public types
pub use error::CompileError;
pub use filter::{filter_html_warnings, DiagnosticFilter, FilterType, PackageKind};
pub use format::{
    format_diagnostics, format_diagnostics_with_options, DiagnosticOptions, DisplayStyle,
};
pub use info::{
    count_diagnostics, has_errors, resolve_diagnostic, resolve_diagnostic_with_offset,
    resolve_diagnostics, DiagnosticInfo, DiagnosticInfoDisplay, DiagnosticSummary, Diagnostics,
    DiagnosticsDisplay, SourceLine, TraceInfo,
};

// Re-export from typst for user convenience
pub use typst::diag::{Severity as DiagnosticSeverity, SourceDiagnostic};
