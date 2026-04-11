// (C) 2025 - Enzo Lombardi

//! FileEditor view - text editor with file loading, saving, and modified state tracking.
// FileEditor - EditWindow with file management and save prompts
//
// Matches Borland: TFileEditor (tfileedi.h, tfileedi.cc)
//
// Extends EditWindow with:
// - File name tracking
// - Modified flag tracking
// - valid(cmClose) for save prompts
// - Load/Save/SaveAs operations
//
// Architecture:
// Editor (core editing) -> EditWindow (adds frame/scrollbars) -> FileEditor (adds file I/O)

use std::path::PathBuf;
use crate::core::geometry::Rect;
use crate::core::event::Event;
use crate::core::command::{CommandId, CM_YES, CM_NO};
use crate::core::state::StateFlags;
use crate::terminal::Terminal;
use crate::app::Application;
use super::edit_window::EditWindow;
use super::view::View;
use super::msgbox::confirmation_box;

/// FileEditor - EditWindow with file management
///
/// Matches Borland: TFileEditor
pub struct FileEditor {
    edit_window: EditWindow,
    filename: Option<PathBuf>,
}

impl FileEditor {
    /// Create a new file editor window
    ///
    /// Matches Borland: TFileEditor(bounds, hScrollBar, vScrollBar, indicator, fileName)
    pub fn new(bounds: Rect, title: &str) -> Self {
        Self {
            edit_window: EditWindow::new(bounds, title),
            filename: None,
        }
    }

    /// Load a file
    ///
    /// Matches Borland: TFileEditor::loadFile()
    pub fn load_file(&mut self, path: PathBuf) -> std::io::Result<()> {
        self.edit_window.load_file(&path)?;
        self.filename = Some(path);
        Ok(())
    }

    /// Save the current file
    ///
    /// Matches Borland: TFileEditor::save()
    pub fn save(&mut self) -> std::io::Result<bool> {
        if self.filename.is_some() {
            self.edit_window.save_file()?;
            Ok(true)
        } else {
            Ok(false) // Need to call save_as
        }
    }

    /// Save as a new file
    ///
    /// Matches Borland: TFileEditor::saveAs()
    pub fn save_as(&mut self, path: PathBuf) -> std::io::Result<()> {
        self.edit_window.save_as(&path)?;
        self.filename = Some(path);
        Ok(())
    }

    /// Get the filename
    pub fn filename(&self) -> Option<&PathBuf> {
        self.filename.as_ref()
    }

    /// Get display name for title
    ///
    /// Returns "Untitled" if no filename
    pub fn get_title(&self) -> String {
        self.filename
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled")
            .to_string()
    }

    /// Check if modified
    pub fn is_modified(&self) -> bool {
        self.edit_window.is_modified()
    }

    /// Refresh the window title based on current filename
    ///
    /// Updates the window's title bar to show the current filename
    /// (or "Untitled" if no file is loaded)
    pub fn refresh_title(&mut self) {
        let title = self.get_title();
        self.edit_window.set_title(&title);
    }

    /// Set text content
    pub fn set_text(&mut self, text: &str) {
        self.edit_window.editor_rc().borrow_mut().set_text(text);
    }

    /// Validate before close
    ///
    /// Matches Borland: TFileEditor::valid(command)
    /// Returns true if close is allowed, false if cancelled
    pub fn valid(&mut self, app: &mut Application, command: CommandId) -> bool {
        // Only prompt for cmClose when modified
        if command == crate::core::command::CM_CLOSE && self.is_modified() {
            let message = format!("Save changes to {}?", self.get_title());
            match confirmation_box(app, &message) {
                cmd if cmd == CM_YES => {
                    // Try to save
                    if let Some(_) = &self.filename {
                        self.save().is_ok()
                    } else {
                        // TODO: Need to show save_as dialog
                        // For now, just allow close
                        true
                    }
                }
                cmd if cmd == CM_NO => {
                    // Don't save, allow close
                    true
                }
                _ => {
                    // Cancel
                    false
                }
            }
        } else {
            // Not modified or not closing, allow
            true
        }
    }

    /// Get mutable reference to the underlying edit window
    pub fn edit_window_mut(&mut self) -> &mut EditWindow {
        &mut self.edit_window
    }

    /// Get reference to the underlying edit window
    pub fn edit_window(&self) -> &EditWindow {
        &self.edit_window
    }
}

impl View for FileEditor {
    fn bounds(&self) -> Rect {
        self.edit_window.bounds()
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.edit_window.set_bounds(bounds);
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        self.edit_window.draw(terminal);
    }

    fn handle_event(&mut self, event: &mut Event) {
        self.edit_window.handle_event(event);
    }

    fn can_focus(&self) -> bool {
        self.edit_window.can_focus()
    }

    fn options(&self) -> u16 {
        self.edit_window.options()
    }

    fn set_options(&mut self, options: u16) {
        self.edit_window.set_options(options);
    }

    fn state(&self) -> StateFlags {
        self.edit_window.state()
    }

    fn set_state(&mut self, state: StateFlags) {
        self.edit_window.set_state(state);
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        self.edit_window.get_palette()
    }

    fn set_palette_chain(&mut self, node: Option<crate::core::palette_chain::PaletteChainNode>) {
        self.edit_window.set_palette_chain(node);
    }

    fn get_palette_chain(&self) -> Option<&crate::core::palette_chain::PaletteChainNode> {
        self.edit_window.get_palette_chain()
    }
}

/// Builder for creating file editors with a fluent API.
pub struct FileEditorBuilder {
    bounds: Option<Rect>,
    title: String,
}

impl FileEditorBuilder {
    pub fn new() -> Self {
        Self {
            bounds: None,
            title: "Untitled".to_string(),
        }
    }

    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    #[must_use]
    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn build(self) -> FileEditor {
        let bounds = self.bounds.expect("FileEditor bounds must be set");
        FileEditor::new(bounds, &self.title)
    }

    pub fn build_boxed(self) -> Box<FileEditor> {
        Box::new(self.build())
    }
}

impl Default for FileEditorBuilder {
    fn default() -> Self {
        Self::new()
    }
}
