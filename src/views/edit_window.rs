// EditWindow - Window wrapper for Editor
//
// Matches Borland: TEditWindow (teditor.h)
//
// A simple window that contains an Editor with scrollbars and indicator.
// Provides a ready-to-use editor window for text editing.

use crate::core::geometry::Rect;
use crate::core::event::Event;
use crate::core::state::StateFlags;
use crate::terminal::Terminal;
use super::window::Window;
use super::editor::Editor;
use super::view::View;

/// EditWindow - Window containing an Editor
///
/// Matches Borland: TEditWindow
pub struct EditWindow {
    window: Window,
    editor: Editor,
}

impl EditWindow {
    /// Create a new edit window
    pub fn new(bounds: Rect, title: &str) -> Self {
        let window = Window::new(bounds, title);

        // Editor fills the window interior
        let editor_bounds = Rect::new(1, 1, bounds.width() - 1, bounds.height() - 1);
        let editor = Editor::new(editor_bounds).with_scrollbars_and_indicator();

        Self { window, editor }
    }

    /// Load a file into the editor
    pub fn load_file(&mut self, path: &str) -> std::io::Result<()> {
        self.editor.load_file(path)
    }

    /// Save the editor content
    pub fn save_file(&mut self) -> std::io::Result<()> {
        self.editor.save_file()
    }

    /// Save as a different file
    pub fn save_as(&mut self, path: &str) -> std::io::Result<()> {
        self.editor.save_as(path)
    }

    /// Get the editor's filename
    pub fn get_filename(&self) -> Option<&str> {
        self.editor.get_filename()
    }

    /// Check if editor is modified
    pub fn is_modified(&self) -> bool {
        self.editor.is_modified()
    }

    /// Get mutable reference to the editor
    pub fn editor_mut(&mut self) -> &mut Editor {
        &mut self.editor
    }

    /// Get reference to the editor
    pub fn editor(&self) -> &Editor {
        &self.editor
    }
}

impl View for EditWindow {
    fn bounds(&self) -> Rect {
        self.window.bounds()
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.window.set_bounds(bounds);
        // Update editor bounds to match window interior
        let editor_bounds = Rect::new(1, 1, bounds.width() - 1, bounds.height() - 1);
        self.editor.set_bounds(editor_bounds);
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        self.window.draw(terminal);
        self.editor.draw(terminal);
    }

    fn handle_event(&mut self, event: &mut Event) {
        // Editor handles most events
        self.editor.handle_event(event);

        // Window handles frame events (resize, move, etc.)
        // Only if event wasn't handled by editor
        self.window.handle_event(event);
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn state(&self) -> StateFlags {
        self.window.state()
    }

    fn set_state(&mut self, state: StateFlags) {
        self.window.set_state(state);
        self.editor.set_state(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_edit_window_creation() {
        let bounds = Rect::new(0, 0, 80, 25);
        let window = EditWindow::new(bounds, "Test Editor");

        assert_eq!(window.bounds(), bounds);
        assert!(!window.is_modified());
    }

    #[test]
    fn test_edit_window_file_operations() {
        let bounds = Rect::new(0, 0, 80, 25);
        let mut window = EditWindow::new(bounds, "Test Editor");

        // Create temp file
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Test content").unwrap();
        file.flush().unwrap();

        // Load file
        let path = file.path().to_str().unwrap();
        window.load_file(path).unwrap();

        assert_eq!(window.get_filename(), Some(path));
        assert!(!window.is_modified());

        // Save as
        let file2 = NamedTempFile::new().unwrap();
        let path2 = file2.path().to_str().unwrap();
        window.save_as(path2).unwrap();

        assert_eq!(window.get_filename(), Some(path2));
    }

    #[test]
    fn test_edit_window_editor_access() {
        let bounds = Rect::new(0, 0, 80, 25);
        let mut window = EditWindow::new(bounds, "Test Editor");

        // Test mutable access
        window.editor_mut().set_text("Hello, World!");

        // Test immutable access
        assert_eq!(window.editor().get_text(), "Hello, World!");
    }
}
