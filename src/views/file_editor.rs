// (C) 2025 - Enzo Lombardi

//! FileEditor view - text editor with file loading, saving, and modified state tracking.
// FileEditor - Editor with file management and save prompts
//
// Matches Borland: TFileEditor (tfileedi.h, tfileedi.cc)
//
// Extends Editor with:
// - File name tracking
// - Modified flag tracking
// - valid(cmClose) for save prompts
// - Load/Save/SaveAs operations

use std::path::PathBuf;
use crate::core::geometry::Rect;
use crate::core::event::Event;
use crate::core::command::{CommandId, CM_YES, CM_NO};
use crate::core::state::StateFlags;
use crate::terminal::Terminal;
use crate::app::Application;
use super::editor::Editor;
use super::view::View;
use super::msgbox::confirmation_box;

/// FileEditor - Editor with file management
///
/// Matches Borland: TFileEditor
pub struct FileEditor {
    editor: Editor,
    filename: Option<PathBuf>,
}

impl FileEditor {
    /// Create a new file editor
    ///
    /// Matches Borland: TFileEditor(bounds, hScrollBar, vScrollBar, indicator, fileName)
    /// Note: In the Rust version, scrollbars should be managed by the parent Window
    pub fn new(bounds: Rect) -> Self {
        Self {
            editor: Editor::new(bounds),
            filename: None,
        }
    }

    /// Load a file
    ///
    /// Matches Borland: TFileEditor::loadFile()
    pub fn load_file(&mut self, path: PathBuf) -> std::io::Result<()> {
        self.editor.load_file(path.to_str().unwrap())?;
        self.filename = Some(path);
        Ok(())
    }

    /// Save the current file
    ///
    /// Matches Borland: TFileEditor::save()
    pub fn save(&mut self) -> std::io::Result<bool> {
        if self.filename.is_some() {
            self.editor.save_file()?;
            Ok(true)
        } else {
            Ok(false) // Need to call save_as
        }
    }

    /// Save as a new file
    ///
    /// Matches Borland: TFileEditor::saveAs()
    pub fn save_as(&mut self, path: PathBuf) -> std::io::Result<()> {
        self.editor.save_as(path.to_str().unwrap())?;
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
        self.editor.is_modified()
    }

    /// Set text content
    pub fn set_text(&mut self, text: &str) {
        self.editor.set_text(text);
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

    /// Get mutable reference to the underlying editor
    pub fn editor_mut(&mut self) -> &mut Editor {
        &mut self.editor
    }

    /// Get reference to the underlying editor
    pub fn editor(&self) -> &Editor {
        &self.editor
    }
}

impl View for FileEditor {
    fn bounds(&self) -> Rect {
        self.editor.bounds()
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.editor.set_bounds(bounds);
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        self.editor.draw(terminal);
    }

    fn handle_event(&mut self, event: &mut Event) {
        self.editor.handle_event(event);
    }

    fn can_focus(&self) -> bool {
        self.editor.can_focus()
    }

    fn state(&self) -> StateFlags {
        self.editor.state()
    }

    fn set_state(&mut self, state: StateFlags) {
        self.editor.set_state(state);
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        self.editor.get_palette()
    }
}

/// Builder for creating file editors with a fluent API.
pub struct FileEditorBuilder {
    bounds: Option<Rect>,
}

impl FileEditorBuilder {
    pub fn new() -> Self {
        Self { bounds: None }
    }

    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    pub fn build(self) -> FileEditor {
        let bounds = self.bounds.expect("FileEditor bounds must be set");
        FileEditor::new(bounds)
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
