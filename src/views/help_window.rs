// (C) 2025 - Enzo Lombardi

//! HelpWindow view - window container for displaying context-sensitive help.
// HelpWindow - Help display window
//
// Matches Borland: THelpWindow (help.h)
//
// A window containing a HelpViewer with navigation and topic selection.

use crate::core::geometry::Rect;
use crate::core::event::{Event, EventType, KB_ESC, KB_CTRL_X};
use crate::core::state::StateFlags;
use crate::core::command::{CM_CANCEL, CM_CLOSE, CommandId};
use crate::terminal::Terminal;
use super::view::View;
use super::window::Window;
use super::help_viewer::HelpViewer;
use super::help_file::HelpFile;
use std::rc::Rc;
use std::cell::RefCell;

/// Wrapper that allows HelpViewer to be shared between window and HelpWindow
struct SharedHelpViewer(Rc<RefCell<HelpViewer>>);

impl View for SharedHelpViewer {
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

    fn state(&self) -> StateFlags {
        self.0.borrow().state()
    }

    fn set_state(&mut self, state: StateFlags) {
        self.0.borrow_mut().set_state(state);
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

/// HelpWindow - Window containing help viewer
///
/// Matches Borland: THelpWindow (parent-child hierarchy)
pub struct HelpWindow {
    window: Window,
    viewer: Rc<RefCell<HelpViewer>>,  // Shared reference for API access
    help_file: Rc<RefCell<HelpFile>>,
    /// Topic history for back/forward navigation
    history: Vec<String>,
    /// Current position in history
    history_pos: usize,
}

impl HelpWindow {
    /// Create a new help window
    ///
    /// Matches Borland: THelpWindow constructor creates TWindow and inserts THelp Viewer as child
    pub fn new(bounds: Rect, title: &str, help_file: Rc<RefCell<HelpFile>>) -> Self {
        let mut window = Window::new(bounds, title);

        // Get the window's interior bounds to calculate viewer size
        // Window interior is inset by 1 on all sides for the frame
        let window_bounds = bounds;
        let interior_width = window_bounds.width().saturating_sub(2);
        let interior_height = window_bounds.height().saturating_sub(2);

        // Viewer uses (0, 0) as relative coords (will be offset by window's interior group)
        // Leave 1 character width for scrollbar (on the right edge)
        let viewer_bounds = Rect::new(
            0,
            0,
            interior_width.saturating_sub(1),  // Width minus 1 for scrollbar
            interior_height,                     // Full interior height
        );
        let viewer = Rc::new(RefCell::new(
            HelpViewer::new(viewer_bounds).with_scrollbar()
        ));

        // Insert viewer as a child of window's interior
        // When added to window's interior group, bounds (0,0,...) will be offset to absolute coords
        window.add(Box::new(SharedHelpViewer(Rc::clone(&viewer))));

        Self {
            window,
            viewer,
            help_file,
            history: Vec::new(),
            history_pos: 0,
        }
    }

    /// Show a topic by ID
    /// Does not add to history (use switchToTopic for navigation with history)
    pub fn show_topic(&mut self, topic_id: &str) -> bool {
        let help = self.help_file.borrow();
        if let Some(topic) = help.get_topic(topic_id) {
            self.viewer.borrow_mut().set_topic(topic);
            true
        } else {
            false
        }
    }

    /// Show the default topic
    pub fn show_default_topic(&mut self) {
        let help = self.help_file.borrow();
        if let Some(topic) = help.get_default_topic() {
            self.viewer.borrow_mut().set_topic(topic);
        }
    }

    /// Get the current topic ID
    pub fn current_topic(&self) -> Option<String> {
        self.viewer.borrow().current_topic().map(|s| s.to_string())
    }

    /// Get a cloned Rc to the viewer for advanced access
    pub fn viewer_rc(&self) -> Rc<RefCell<HelpViewer>> {
        Rc::clone(&self.viewer)
    }

    /// Get reference to the help file
    pub fn help_file(&self) -> &Rc<RefCell<HelpFile>> {
        &self.help_file
    }

    /// Switch to a topic (with history tracking)
    /// Matches Borland: THelpViewer::switchToTopic()
    /// This is the method to use for hyperlink navigation
    pub fn switch_to_topic(&mut self, topic_id: &str) -> bool {
        // Only proceed if topic exists
        let help = self.help_file.borrow();
        if help.get_topic(topic_id).is_none() {
            return false;
        }
        drop(help);

        // If we're not at the end of history, truncate future history
        if self.history_pos < self.history.len() {
            self.history.truncate(self.history_pos);
        }

        // Add current topic to history before switching
        if let Some(current) = self.viewer.borrow().current_topic() {
            self.history.push(current.to_string());
        }

        // Show the new topic
        let success = self.show_topic(topic_id);
        if success {
            self.history_pos = self.history.len();
        }
        success
    }

    /// Navigate back in history
    /// Returns true if navigation occurred
    pub fn go_back(&mut self) -> bool {
        if self.history_pos > 0 {
            self.history_pos -= 1;
            let topic_id = self.history[self.history_pos].clone();
            self.show_topic(&topic_id);
            true
        } else {
            false
        }
    }

    /// Navigate forward in history
    /// Returns true if navigation occurred
    pub fn go_forward(&mut self) -> bool {
        if self.history_pos < self.history.len() {
            let topic_id = self.history[self.history_pos].clone();
            self.history_pos += 1;
            self.show_topic(&topic_id);
            true
        } else {
            false
        }
    }

    /// Check if we can go back
    pub fn can_go_back(&self) -> bool {
        self.history_pos > 0
    }

    /// Check if we can go forward
    pub fn can_go_forward(&self) -> bool {
        self.history_pos < self.history.len()
    }

    /// Create and show a topic selection dialog
    /// Matches Borland: THelpViewer::makeSelectTopic()
    /// Returns the selected topic ID, or None if cancelled
    pub fn make_select_topic(&self) -> Option<String> {
        // Get all available topics from help file
        let help = self.help_file.borrow();
        let topics = help.get_topic_ids();

        if topics.is_empty() {
            return None;
        }

        // For now, return the first topic as a placeholder
        // TODO: Show a proper selection dialog (ListBox in a Dialog)
        // This would require access to Application to show modal dialog
        Some(topics[0].clone())
    }

    /// Execute the help window modally with proper event loop
    /// Unlike Window::execute(), this calls HelpWindow::handle_event() which provides
    /// ESC handling and other help-specific features
    pub fn execute(&mut self, app: &mut crate::app::Application) -> CommandId {
        use std::time::Duration;

        // Constrain window position to desktop bounds
        self.window.constrain_to_limits();

        // Set explicit drag limits from desktop bounds
        let desktop_bounds = app.desktop.get_bounds();
        self.window.set_drag_limits(desktop_bounds);

        loop {
            // Draw desktop first (clears background)
            app.desktop.draw(&mut app.terminal);

            // Draw menu bar if present
            if let Some(ref mut menu_bar) = app.menu_bar {
                menu_bar.draw(&mut app.terminal);
            }

            // Draw status line if present
            if let Some(ref mut status_line) = app.status_line {
                status_line.draw(&mut app.terminal);
            }

            // Draw the help window on top of everything
            self.draw(&mut app.terminal);

            // Draw overlay widgets
            for widget in &mut app.overlay_widgets {
                widget.draw(&mut app.terminal);
            }

            self.update_cursor(&mut app.terminal);
            let _ = app.terminal.flush();

            // Poll for events with timeout
            match app.terminal.poll_event(Duration::from_millis(20)).ok().flatten() {
                Some(mut event) => {
                    // CRITICAL: Call help_window.handle_event(), not window.handle_event()
                    // This ensures ESC handling and other help-specific features work
                    self.handle_event(&mut event);

                    // Check if help window should close
                    if self.window.get_end_state() != 0 {
                        return self.window.get_end_state();
                    }
                }
                None => {
                    // Timeout - call idle
                    app.idle();

                    // Check if help window should close even without events
                    if self.window.get_end_state() != 0 {
                        return self.window.get_end_state();
                    }
                }
            }
        }
    }

    /// End the modal event loop
    pub fn end_modal(&mut self, command: CommandId) {
        self.window.end_modal(command);
    }
}

impl View for HelpWindow {
    fn bounds(&self) -> Rect {
        self.window.bounds()
    }

    fn set_bounds(&mut self, bounds: Rect) {
        // Delegate to window - it will update the interior group and all children
        self.window.set_bounds(bounds);
        // The window's interior group will automatically call set_bounds() on the viewer
        // via the SharedHelpViewer wrapper, so we don't need to do anything here
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        // Window draws itself and all children (including viewer)
        self.window.draw(terminal);
    }

    fn handle_event(&mut self, event: &mut Event) {
        // Close button, ESC, or Ctrl+X closes the help window
        if event.what == EventType::Keyboard {
            if event.key_code == KB_ESC || event.key_code == KB_CTRL_X {
                self.window.end_modal(CM_CANCEL);
                event.clear();
                return;
            }
        }

        // Handle close button click (CM_CLOSE generated by Frame when close button is clicked)
        if event.what == EventType::Command && event.command == CM_CLOSE {
            self.window.end_modal(CM_CANCEL);
            event.clear();
            return;
        }

        // Window handles events and dispatches to children (including viewer)
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
        self.viewer.borrow_mut().set_state(state);
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        self.window.get_palette()
    }

    fn update_cursor(&self, terminal: &mut crate::terminal::Terminal) {
        self.window.update_cursor(terminal);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_help_file() -> (NamedTempFile, Rc<RefCell<HelpFile>>) {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "# Test Topic {{#test}}").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "This is test content.").unwrap();
        file.flush().unwrap();

        let help = HelpFile::new(file.path().to_str().unwrap()).unwrap();
        (file, Rc::new(RefCell::new(help)))
    }

    #[test]
    fn test_help_window_creation() {
        let (_file, help) = create_test_help_file();
        let bounds = Rect::new(10, 5, 70, 20);
        let window = HelpWindow::new(bounds, "Help", help);

        assert_eq!(window.bounds(), bounds);
    }

    #[test]
    fn test_show_topic() {
        let (_file, help) = create_test_help_file();
        let bounds = Rect::new(10, 5, 70, 20);
        let mut window = HelpWindow::new(bounds, "Help", help);

        assert!(window.show_topic("test"));
        assert_eq!(window.current_topic(), Some("test".to_string()));
    }

    #[test]
    fn test_show_default_topic() {
        let (_file, help) = create_test_help_file();
        let bounds = Rect::new(10, 5, 70, 20);
        let mut window = HelpWindow::new(bounds, "Help", help);

        window.show_default_topic();
        assert_eq!(window.current_topic(), Some("test".to_string()));
    }

    #[test]
    fn test_show_nonexistent_topic() {
        let (_file, help) = create_test_help_file();
        let bounds = Rect::new(10, 5, 70, 20);
        let mut window = HelpWindow::new(bounds, "Help", help);

        assert!(!window.show_topic("nonexistent"));
    }
}

/// Builder for creating help windows with a fluent API.
pub struct HelpWindowBuilder {
    bounds: Option<Rect>,
    title: Option<String>,
    help_file: Option<Rc<RefCell<HelpFile>>>,
}

impl HelpWindowBuilder {
    pub fn new() -> Self {
        Self { bounds: None, title: None, help_file: None }
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

    #[must_use]
    pub fn help_file(mut self, help_file: Rc<RefCell<HelpFile>>) -> Self {
        self.help_file = Some(help_file);
        self
    }

    pub fn build(self) -> HelpWindow {
        let bounds = self.bounds.expect("HelpWindow bounds must be set");
        let title = self.title.expect("HelpWindow title must be set");
        let help_file = self.help_file.expect("HelpWindow help_file must be set");
        HelpWindow::new(bounds, &title, help_file)
    }

    pub fn build_boxed(self) -> Box<HelpWindow> {
        Box::new(self.build())
    }
}

impl Default for HelpWindowBuilder {
    fn default() -> Self {
        Self::new()
    }
}
