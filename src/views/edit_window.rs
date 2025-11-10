// (C) 2025 - Enzo Lombardi

//! EditWindow view - window container for editor with title bar showing filename.
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
use std::rc::Rc;
use std::cell::RefCell;

/// Wrapper that allows Editor to be shared between window and EditWindow
struct SharedEditor(Rc<RefCell<Editor>>);

impl View for SharedEditor {
    fn bounds(&self) -> Rect {
        self.0.borrow().bounds()
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.0.borrow_mut().set_bounds(bounds);
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        self.0.borrow_mut().draw(terminal);
    }

    fn handle_event(&mut self, event: &mut Event) {
        self.0.borrow_mut().handle_event(event);
    }

    fn can_focus(&self) -> bool {
        self.0.borrow().can_focus()
    }

    fn set_focus(&mut self, focused: bool) {
        self.0.borrow_mut().set_focus(focused);
    }

    fn is_focused(&self) -> bool {
        self.0.borrow().is_focused()
    }

    fn options(&self) -> u16 {
        self.0.borrow().options()
    }

    fn set_options(&mut self, options: u16) {
        self.0.borrow_mut().set_options(options);
    }

    fn state(&self) -> StateFlags {
        self.0.borrow().state()
    }

    fn set_state(&mut self, state: StateFlags) {
        self.0.borrow_mut().set_state(state);
    }

    fn update_cursor(&self, terminal: &mut Terminal) {
        self.0.borrow().update_cursor(terminal);
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        self.0.borrow().get_palette()
    }

    fn get_owner_type(&self) -> super::view::OwnerType {
        self.0.borrow().get_owner_type()
    }

    fn set_owner_type(&mut self, owner_type: super::view::OwnerType) {
        self.0.borrow_mut().set_owner_type(owner_type);
    }
}

/// EditWindow - Window containing an Editor
///
/// Matches Borland: TEditWindow (parent-child hierarchy)
/// The Editor is inserted as a child of the Window, matching Borland's structure.
pub struct EditWindow {
    window: Window,
    editor: Rc<RefCell<Editor>>,  // Shared reference for API access
}

impl EditWindow {
    /// Create a new edit window
    ///
    /// Matches Borland: TEditWindow constructor creates TWindow and inserts TEditor as child
    pub fn new(bounds: Rect, title: &str) -> Self {
        let mut window = Window::new(bounds, title);

        // Editor fills the window interior
        let editor_bounds = Rect::new(1, 1, bounds.width() - 1, bounds.height() - 1);
        let editor = Rc::new(RefCell::new(
            Editor::new(editor_bounds).with_scrollbars_and_indicator()
        ));

        // Insert editor as a child of window (matches Borland's window->insert(editor))
        window.add(Box::new(SharedEditor(Rc::clone(&editor))));

        Self { window, editor }
    }

    /// Load a file into the editor
    pub fn load_file(&mut self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        self.editor.borrow_mut().load_file(path)
    }

    /// Save the editor content
    pub fn save_file(&mut self) -> std::io::Result<()> {
        self.editor.borrow_mut().save_file()
    }

    /// Save as a different file
    pub fn save_as(&mut self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        self.editor.borrow_mut().save_as(path)
    }

    /// Get the editor's filename
    pub fn get_filename(&self) -> Option<String> {
        self.editor.borrow().get_filename().map(|s| s.to_string())
    }

    /// Check if editor is modified
    pub fn is_modified(&self) -> bool {
        self.editor.borrow().is_modified()
    }

    /// Get a cloned Rc to the editor for advanced access
    pub fn editor_rc(&self) -> Rc<RefCell<Editor>> {
        Rc::clone(&self.editor)
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
        self.editor.borrow_mut().set_bounds(editor_bounds);
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        // Window draws itself and all children (including editor)
        self.window.draw(terminal);
    }

    fn handle_event(&mut self, event: &mut Event) {
        // Window handles events and dispatches to children
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
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        self.window.get_palette()
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

        assert_eq!(window.get_filename(), Some(path.to_string()));
        assert!(!window.is_modified());

        // Save as
        let file2 = NamedTempFile::new().unwrap();
        let path2 = file2.path().to_str().unwrap();
        window.save_as(path2).unwrap();

        assert_eq!(window.get_filename(), Some(path2.to_string()));
    }

    #[test]
    fn test_edit_window_editor_access() {
        let bounds = Rect::new(0, 0, 80, 25);
        let window = EditWindow::new(bounds, "Test Editor");

        // Test access via Rc
        let editor = window.editor_rc();
        editor.borrow_mut().set_text("Hello, World!");
        assert_eq!(editor.borrow().get_text(), "Hello, World!");
    }
}

/// Builder for creating edit windows with a fluent API.
pub struct EditWindowBuilder {
    bounds: Option<Rect>,
    title: Option<String>,
}

impl EditWindowBuilder {
    pub fn new() -> Self {
        Self { bounds: None, title: None }
    }

    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn build(self) -> EditWindow {
        let bounds = self.bounds.expect("EditWindow bounds must be set");
        let title = self.title.expect("EditWindow title must be set");
        EditWindow::new(bounds, &title)
    }

    pub fn build_boxed(self) -> Box<EditWindow> {
        Box::new(self.build())
    }
}

impl Default for EditWindowBuilder {
    fn default() -> Self {
        Self::new()
    }
}
