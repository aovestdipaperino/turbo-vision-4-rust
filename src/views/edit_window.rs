// (C) 2025 - Enzo Lombardi

//! EditWindow view - window container for editor with title bar showing filename.
// EditWindow - Window wrapper for Editor
//
// Matches Borland: TEditWindow (teditor.h)
//
// A simple window that contains an Editor with scrollbars and indicator.
// Provides a ready-to-use editor window for text editing.

use crate::core::geometry::{Point, Rect};
use crate::core::event::{Event, EventType};
use crate::core::state::StateFlags;
use crate::terminal::Terminal;
use super::window::Window;
use super::editor::Editor;
use super::scrollbar::ScrollBar;
use super::indicator::Indicator;
use super::view::View;
use std::rc::Rc;
use std::cell::RefCell;

/// Wrapper that allows ScrollBar to be a child view
struct SharedScrollBar(Rc<RefCell<ScrollBar>>);

impl View for SharedScrollBar {
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

/// Wrapper that allows Indicator to be a child view
struct SharedIndicator(Rc<RefCell<Indicator>>);

impl View for SharedIndicator {
    fn bounds(&self) -> Rect {
        self.0.borrow().bounds()
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.0.borrow_mut().set_bounds(bounds);
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        self.0.borrow_mut().draw(terminal);
    }

    fn handle_event(&mut self, _event: &mut Event) {
        // Indicator doesn't handle events
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
    #[allow(dead_code)] // Used for initialization, stored for lifetime management
    h_scrollbar: Rc<RefCell<ScrollBar>>,
    #[allow(dead_code)] // Used for initialization, stored for lifetime management
    v_scrollbar: Rc<RefCell<ScrollBar>>,
    #[allow(dead_code)] // Used for initialization, stored for lifetime management
    indicator: Rc<RefCell<Indicator>>,
    // Indices in window.frame_children for direct updates
    h_scrollbar_idx: usize,
    v_scrollbar_idx: usize,
    indicator_idx: usize,
}

impl EditWindow {
    /// Create a new edit window
    ///
    /// Matches Borland: TEditWindow constructor creates TWindow and inserts scrollbars+editor as children
    pub fn new(bounds: Rect, title: &str) -> Self {
        let mut window = Window::new(bounds, title);

        // Calculate window size (matching Borland's size.x, size.y)
        let window_width = bounds.width();
        let window_height = bounds.height();

        // Calculate interior size (relative coordinates)
        let interior_width = window_width - 2;  // Subtract frame
        let interior_height = window_height - 2;

        // Create scrollbars at frame edges (matching Borland's TEditWindow)
        // Positions are relative to window frame (0,0 = top-left of frame)
        let h_bounds = Rect::new(18, window_height - 1, window_width - 2, window_height);
        let h_scrollbar = Rc::new(RefCell::new(ScrollBar::new_horizontal(h_bounds)));

        let v_bounds = Rect::new(window_width - 1, 1, window_width, window_height - 2);
        let v_scrollbar = Rc::new(RefCell::new(ScrollBar::new_vertical(v_bounds)));

        let ind_bounds = Rect::new(2, window_height - 1, 16, window_height);
        let indicator = Rc::new(RefCell::new(Indicator::new(ind_bounds)));

        // Create editor with Borland bounds: r.grow(-1,-1) from interior
        // This means (1, 1, interior_width - 1, interior_height - 1)
        // The editor will overlap with scrollbars at the edges, scrollbars draw on top
        let editor_bounds = Rect::new(1, 1, interior_width - 1, interior_height - 1);
        let editor = Rc::new(RefCell::new(Editor::with_scrollbars(
            editor_bounds,
            Some(Rc::clone(&h_scrollbar)),
            Some(Rc::clone(&v_scrollbar)),
            Some(Rc::clone(&indicator)),
        )));

        // IMPORTANT: Insert editor into interior (relative to interior bounds)
        // But insert scrollbars/indicator as frame children (relative to window frame)
        window.add(Box::new(SharedEditor(Rc::clone(&editor))));
        let h_scrollbar_idx = window.add_frame_child(Box::new(SharedScrollBar(Rc::clone(&h_scrollbar))));
        let v_scrollbar_idx = window.add_frame_child(Box::new(SharedScrollBar(Rc::clone(&v_scrollbar))));
        let indicator_idx = window.add_frame_child(Box::new(SharedIndicator(Rc::clone(&indicator))));

        // Set initial indicator value to cursor position (1:1)
        // Editor cursor starts at (0,0) internally, displayed as (1,1) for user
        indicator.borrow_mut().set_value(
            Point::new(1, 1),
            false,
        );

        Self {
            window,
            editor,
            h_scrollbar,
            v_scrollbar,
            indicator,
            h_scrollbar_idx,
            v_scrollbar_idx,
            indicator_idx,
        }
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

    /// Synchronize frame children (scrollbars, indicator) positions with window bounds
    /// Called from draw() to ensure positions are always correct, preventing visual lag during resize
    /// IMPORTANT: Always update positions regardless of size to prevent elements "staying behind"
    fn sync_frame_children_positions(&mut self) {
        let bounds = self.window.bounds();
        let window_width = bounds.width();
        let window_height = bounds.height();

        // Always update horizontal scrollbar position (even if window is too small to display it properly)
        // This prevents it from "staying behind" during rapid resizing near minimum size
        if window_height >= 3 {
            let h_bounds = Rect::new(
                bounds.a.x + 18.min(window_width.saturating_sub(2)),
                bounds.a.y + window_height - 1,
                bounds.a.x + window_width - 2,
                bounds.a.y + window_height,
            );
            self.window.update_frame_child(self.h_scrollbar_idx, h_bounds);
        }

        // Always update vertical scrollbar position
        if window_width >= 3 && window_height >= 4 {
            let v_bounds = Rect::new(
                bounds.a.x + window_width - 1,
                bounds.a.y + 1,
                bounds.a.x + window_width,
                bounds.a.y + window_height - 2,
            );
            self.window.update_frame_child(self.v_scrollbar_idx, v_bounds);
        }

        // Always update indicator position (even if window is too small)
        if window_height >= 3 {
            let ind_bounds = Rect::new(
                bounds.a.x + 2,
                bounds.a.y + window_height - 1,
                bounds.a.x + 16.min(window_width - 2),
                bounds.a.y + window_height,
            );
            self.window.update_frame_child(self.indicator_idx, ind_bounds);
            // Note: Indicator value (cursor position) is updated by the Editor itself
        }
    }
}

impl View for EditWindow {
    fn bounds(&self) -> Rect {
        self.window.bounds()
    }

    fn set_bounds(&mut self, bounds: Rect) {
        // Window handles updating all children (including scrollbars, indicator, and editor)
        self.window.set_bounds(bounds);

        // Recalculate scrollbar positions based on NEW window size
        let window_width = bounds.width();
        let window_height = bounds.height();

        // Update scrollbar bounds to new window edges
        let h_bounds = Rect::new(
            bounds.a.x + 18,
            bounds.a.y + window_height - 1,
            bounds.a.x + window_width - 2,
            bounds.a.y + window_height,
        );
        self.window.update_frame_child(self.h_scrollbar_idx, h_bounds);

        let v_bounds = Rect::new(
            bounds.a.x + window_width - 1,
            bounds.a.y + 1,
            bounds.a.x + window_width,
            bounds.a.y + window_height - 2,
        );
        self.window.update_frame_child(self.v_scrollbar_idx, v_bounds);

        let ind_bounds = Rect::new(
            bounds.a.x + 2,
            bounds.a.y + window_height - 1,
            bounds.a.x + 16,
            bounds.a.y + window_height,
        );
        self.window.update_frame_child(self.indicator_idx, ind_bounds);
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        // IMPORTANT: Update frame children positions BEFORE drawing to prevent visual lag
        // During rapid resizing, this ensures scrollbars are always at correct positions
        self.sync_frame_children_positions();

        // Draw frame and interior first
        self.window.frame_mut().draw(terminal);
        self.window.interior_mut().draw(terminal);

        // Check if scrollbars are needed based on content size
        let editor = self.editor.borrow();
        let needs_h_scrollbar = editor.needs_horizontal_scrollbar();
        let needs_v_scrollbar = editor.needs_vertical_scrollbar();
        drop(editor); // Release borrow before drawing

        // Conditionally draw scrollbars only if needed
        if needs_h_scrollbar {
            if let Some(child) = self.window.get_frame_child_mut(self.h_scrollbar_idx) {
                child.draw(terminal);
            }
        }
        if needs_v_scrollbar {
            if let Some(child) = self.window.get_frame_child_mut(self.v_scrollbar_idx) {
                child.draw(terminal);
            }
        }
        // Always draw indicator
        if let Some(child) = self.window.get_frame_child_mut(self.indicator_idx) {
            child.draw(terminal);
        }

        // Draw shadow if enabled
        if self.window.has_shadow() {
            self.window.draw_shadow(terminal);
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        // Save old bounds before Window processes the event
        let old_bounds = self.window.bounds();

        // Pass mouse events to scrollbars (if they're visible and the event hasn't been handled)
        // IMPORTANT: Only mouse events! Keyboard events (UP/DOWN/etc.) should go to the Editor,
        // not to scrollbars. This allows cursor movement before scrolling.
        if event.what == EventType::MouseDown || event.what == EventType::MouseMove || event.what == EventType::MouseUp {
            let editor = self.editor.borrow();
            let needs_h_scrollbar = editor.needs_horizontal_scrollbar();
            let needs_v_scrollbar = editor.needs_vertical_scrollbar();
            drop(editor);

            let mut scrollbar_handled = false;

            // Let horizontal scrollbar handle event if visible
            if needs_h_scrollbar {
                if let Some(child) = self.window.get_frame_child_mut(self.h_scrollbar_idx) {
                    child.handle_event(event);
                    if event.what == EventType::Nothing {
                        scrollbar_handled = true;
                    }
                }
            }

            // Let vertical scrollbar handle event if visible (and not already handled)
            if !scrollbar_handled && needs_v_scrollbar {
                if let Some(child) = self.window.get_frame_child_mut(self.v_scrollbar_idx) {
                    child.handle_event(event);
                    if event.what == EventType::Nothing {
                        scrollbar_handled = true;
                    }
                }
            }

            // If scrollbar handled the event, sync editor delta from scrollbar values
            if scrollbar_handled {
                self.editor.borrow_mut().sync_from_scrollbars();
                return;
            }
        }

        // Let Window handle the event (drag, resize, etc.)
        self.window.handle_event(event);

        // Check if bounds changed (resize or move)
        let new_bounds = self.window.bounds();
        if old_bounds != new_bounds {
            // Bounds changed - update editor size and frame children positions
            let window_width = new_bounds.width();
            let window_height = new_bounds.height();

            // Update Editor bounds to match new interior size
            // Editor is a child of the interior Group, so it needs ABSOLUTE coordinates
            let interior_width = window_width.saturating_sub(2);  // Subtract frame
            let interior_height = window_height.saturating_sub(2);

            if interior_width > 2 && interior_height > 2 {
                let interior_a = Point::new(new_bounds.a.x + 1, new_bounds.a.y + 1);  // Interior top-left
                let editor_bounds = Rect::new(
                    interior_a.x + 1,
                    interior_a.y + 1,
                    interior_a.x + interior_width - 1,
                    interior_a.y + interior_height - 1,
                );
                self.editor.borrow_mut().set_bounds(editor_bounds);
            }

            // Note: Frame children (scrollbars, indicator) will be synced in draw()
            // No need to update them here - this prevents duplicate work
        }
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

    fn get_end_state(&self) -> crate::core::command::CommandId {
        self.window.get_end_state()
    }

    fn set_end_state(&mut self, command: crate::core::command::CommandId) {
        self.window.set_end_state(command);
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
