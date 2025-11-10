// (C) 2025 - Enzo Lombardi

//! Change Directory Dialog - specialized dialog for directory selection
//!
//! Matches Borland: TChDirDialog
//!
//! A simplified wrapper around FileDialog that only shows directories.
//! Provides a cleaner interface for changing the working directory.

use crate::core::geometry::Rect;
use crate::core::event::Event;
use crate::core::command::{CM_OK, CommandId};
use crate::terminal::Terminal;
use super::file_dialog::FileDialog;
use super::View;
use std::path::PathBuf;

/// Change Directory Dialog
/// Matches Borland: TChDirDialog
///
/// This is a thin wrapper around FileDialog configured for directory selection only.
/// In Rust, we leverage the existing FileDialog infrastructure rather than
/// duplicating code, which is more maintainable and idiomatic.
pub struct ChDirDialog {
    file_dialog: FileDialog,
    selected_directory: Option<PathBuf>,
}

impl ChDirDialog {
    /// Create a new change directory dialog
    ///
    /// # Arguments
    /// * `bounds` - The dialog bounds
    /// * `title` - The dialog title
    /// * `initial_dir` - Initial directory to show (defaults to current directory)
    pub fn new(bounds: Rect, title: &str, initial_dir: Option<PathBuf>) -> Self {
        // Use FileDialog with "*" wildcard to show all directories
        // User navigates by selecting directories (they're shown as [dirname])
        let file_dialog = FileDialog::new(bounds, title, "*", initial_dir);

        Self {
            file_dialog,
            selected_directory: None,
        }
    }

    /// Execute the dialog modally
    ///
    /// Returns the selected directory if OK was pressed, None if cancelled
    pub fn execute(&mut self, app: &mut crate::app::Application) -> Option<PathBuf> {
        // FileDialog.execute() returns Option<PathBuf> for selected file
        // For directory selection, we ignore the return value and use get_current_directory
        let _file_selection = self.file_dialog.execute(app);

        // Check end state to see if OK or Cancel was pressed
        let end_state = self.file_dialog.get_end_state();

        if end_state == CM_OK {
            // Get the current directory from the file dialog
            let directory = self.file_dialog.get_current_directory();
            self.selected_directory = Some(directory.clone());
            Some(directory)
        } else {
            None
        }
    }

    /// Get the selected directory
    ///
    /// Returns Some(path) if OK was pressed, None if cancelled or not executed
    pub fn get_directory(&self) -> Option<PathBuf> {
        self.selected_directory.clone()
    }

    /// Get the end state (command that closed the dialog)
    pub fn get_end_state(&self) -> CommandId {
        self.file_dialog.get_end_state()
    }
}

impl View for ChDirDialog {
    fn bounds(&self) -> Rect {
        self.file_dialog.bounds()
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.file_dialog.set_bounds(bounds);
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        self.file_dialog.draw(terminal);
    }

    fn handle_event(&mut self, event: &mut Event) {
        self.file_dialog.handle_event(event);
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn state(&self) -> crate::core::state::StateFlags {
        self.file_dialog.state()
    }

    fn set_state(&mut self, state: crate::core::state::StateFlags) {
        self.file_dialog.set_state(state);
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        self.file_dialog.get_palette()
    }
}

/// Builder for creating change directory dialogs with a fluent API.
///
/// # Examples
///
/// ```ignore
/// use turbo_vision::views::chdir_dialog::ChDirDialogBuilder;
/// use turbo_vision::core::geometry::Rect;
/// use std::path::PathBuf;
///
/// // Create a basic change directory dialog
/// let dialog = ChDirDialogBuilder::new()
///     .bounds(Rect::new(10, 5, 70, 20))
///     .title("Change Directory")
///     .build();
///
/// // Create a dialog with initial directory
/// let dialog = ChDirDialogBuilder::new()
///     .bounds(Rect::new(10, 5, 70, 20))
///     .title("Select Directory")
///     .initial_dir(PathBuf::from("/home/user"))
///     .build();
/// ```
pub struct ChDirDialogBuilder {
    bounds: Option<Rect>,
    title: Option<String>,
    initial_dir: Option<PathBuf>,
}

impl ChDirDialogBuilder {
    /// Creates a new ChDirDialogBuilder with default values.
    pub fn new() -> Self {
        Self {
            bounds: None,
            title: None,
            initial_dir: None,
        }
    }

    /// Sets the dialog bounds (required).
    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Sets the dialog title (required).
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the initial directory (optional).
    /// If not set, uses the current working directory.
    #[must_use]
    pub fn initial_dir(mut self, dir: PathBuf) -> Self {
        self.initial_dir = Some(dir);
        self
    }

    /// Builds the ChDirDialog.
    ///
    /// # Panics
    ///
    /// Panics if required fields (bounds, title) are not set.
    pub fn build(self) -> ChDirDialog {
        let bounds = self.bounds.expect("ChDirDialog bounds must be set");
        let title = self.title.expect("ChDirDialog title must be set");
        ChDirDialog::new(bounds, &title, self.initial_dir)
    }

    /// Builds the ChDirDialog as a Box.
    pub fn build_boxed(self) -> Box<ChDirDialog> {
        Box::new(self.build())
    }
}

impl Default for ChDirDialogBuilder {
    fn default() -> Self {
        Self::new()
    }
}
