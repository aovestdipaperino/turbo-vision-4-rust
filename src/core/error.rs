// (C) 2025 - Enzo Lombardi

//! Error types for Turbo Vision operations.
//!
//! This module provides the main error types used throughout the library,
//! with proper backtrace support and context preservation.

use std::backtrace::Backtrace;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

/// Error type for Turbo Vision operations.
///
/// Wraps error kinds with backtrace support for debugging.
///
/// # Examples
///
/// ```rust,no_run
/// use turbo_vision::core::error::Result;
///
/// fn init_app() -> Result<()> {
///     // Operations that can fail
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct TurboVisionError {
    kind: ErrorKind,
    backtrace: Backtrace,
}

/// The specific kind of error that occurred.
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum ErrorKind {
    /// I/O operation failed
    Io(std::io::Error),

    /// Terminal initialization failed
    TerminalInit(String),

    /// Invalid input provided
    InvalidInput(String),

    /// Parse error
    Parse(String),

    /// File operation failed
    FileOperation {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl TurboVisionError {
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            backtrace: Backtrace::capture(),
        }
    }

    /// Creates a terminal initialization error.
    #[allow(dead_code)]
    pub(crate) fn terminal_init(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::TerminalInit(msg.into()))
    }

    /// Creates an invalid input error.
    #[allow(dead_code)]
    pub(crate) fn invalid_input(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::InvalidInput(msg.into()))
    }

    /// Creates a parse error.
    #[allow(dead_code)]
    pub(crate) fn parse(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::Parse(msg.into()))
    }

    /// Creates a file operation error.
    #[allow(dead_code)]
    pub(crate) fn file_operation(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::new(ErrorKind::FileOperation {
            path: path.into(),
            source,
        })
    }

    /// Returns `true` if this error is an I/O error.
    pub fn is_io(&self) -> bool {
        matches!(self.kind, ErrorKind::Io(_))
    }

    /// Returns `true` if this error is a terminal initialization error.
    pub fn is_terminal_init(&self) -> bool {
        matches!(self.kind, ErrorKind::TerminalInit(_))
    }

    /// Returns `true` if this error is an invalid input error.
    pub fn is_invalid_input(&self) -> bool {
        matches!(self.kind, ErrorKind::InvalidInput(_))
    }

    /// Returns `true` if this error is a parse error.
    pub fn is_parse(&self) -> bool {
        matches!(self.kind, ErrorKind::Parse(_))
    }

    /// Returns `true` if this error is a file operation error.
    pub fn is_file_operation(&self) -> bool {
        matches!(self.kind, ErrorKind::FileOperation { .. })
    }

    /// Returns the file path if this is a file operation error.
    pub fn file_path(&self) -> Option<&std::path::Path> {
        match &self.kind {
            ErrorKind::FileOperation { path, .. } => Some(path),
            _ => None,
        }
    }
}

impl Display for TurboVisionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ErrorKind::Io(e) => write!(f, "I/O error: {}", e)?,
            ErrorKind::TerminalInit(msg) => write!(f, "Terminal initialization failed: {}", msg)?,
            ErrorKind::InvalidInput(msg) => write!(f, "Invalid input: {}", msg)?,
            ErrorKind::Parse(msg) => write!(f, "Parse error: {}", msg)?,
            ErrorKind::FileOperation { path, source } => {
                write!(
                    f,
                    "File operation failed for '{}': {}",
                    path.display(),
                    source
                )?
            }
        }

        // Include backtrace if captured
        if self.backtrace.status() == std::backtrace::BacktraceStatus::Captured {
            write!(f, "\n\nBacktrace:\n{}", self.backtrace)?;
        }

        Ok(())
    }
}

impl std::error::Error for TurboVisionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Io(e) => Some(e),
            ErrorKind::FileOperation { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<std::io::Error> for TurboVisionError {
    fn from(e: std::io::Error) -> Self {
        Self::new(ErrorKind::Io(e))
    }
}

/// Result type for Turbo Vision operations.
///
/// This is a type alias for `Result<T, TurboVisionError>`.
pub type Result<T> = std::result::Result<T, TurboVisionError>;
