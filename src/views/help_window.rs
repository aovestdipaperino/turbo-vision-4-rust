// HelpWindow - Help display window
//
// Matches Borland: THelpWindow (help.h)
//
// A window containing a HelpViewer with navigation and topic selection.

use crate::core::geometry::Rect;
use crate::core::event::{Event, EventType, KB_ESC};
use crate::core::state::StateFlags;
use crate::core::command::{CM_CANCEL, CommandId};
use crate::terminal::Terminal;
use super::view::View;
use super::window::Window;
use super::help_viewer::HelpViewer;
use super::help_file::HelpFile;
use std::rc::Rc;
use std::cell::RefCell;

/// HelpWindow - Window containing help viewer
///
/// Matches Borland: THelpWindow
pub struct HelpWindow {
    window: Window,
    viewer: HelpViewer,
    help_file: Rc<RefCell<HelpFile>>,
}

impl HelpWindow {
    /// Create a new help window
    pub fn new(bounds: Rect, title: &str, help_file: Rc<RefCell<HelpFile>>) -> Self {
        let window = Window::new(bounds, title);

        // Viewer fills the window interior
        let viewer_bounds = Rect::new(1, 1, bounds.width() - 2, bounds.height() - 2);
        let viewer = HelpViewer::new(viewer_bounds).with_scrollbar();

        Self {
            window,
            viewer,
            help_file,
        }
    }

    /// Show a topic by ID
    pub fn show_topic(&mut self, topic_id: &str) -> bool {
        let help = self.help_file.borrow();
        if let Some(topic) = help.get_topic(topic_id) {
            self.viewer.set_topic(topic);
            true
        } else {
            false
        }
    }

    /// Show the default topic
    pub fn show_default_topic(&mut self) {
        let help = self.help_file.borrow();
        if let Some(topic) = help.get_default_topic() {
            self.viewer.set_topic(topic);
        }
    }

    /// Get the current topic ID
    pub fn current_topic(&self) -> Option<String> {
        self.viewer.current_topic().map(|s| s.to_string())
    }

    /// Get mutable reference to the viewer
    pub fn viewer_mut(&mut self) -> &mut HelpViewer {
        &mut self.viewer
    }

    /// Get reference to the viewer
    pub fn viewer(&self) -> &HelpViewer {
        &self.viewer
    }

    /// Get reference to the help file
    pub fn help_file(&self) -> &Rc<RefCell<HelpFile>> {
        &self.help_file
    }

    /// Execute the help window modally
    pub fn execute(&mut self, app: &mut crate::app::Application) -> CommandId {
        self.window.execute(app)
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
        self.window.set_bounds(bounds);
        // Update viewer bounds to match window interior
        let viewer_bounds = Rect::new(1, 1, bounds.width() - 2, bounds.height() - 2);
        self.viewer.set_bounds(viewer_bounds);
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        self.window.draw(terminal);
        self.viewer.draw(terminal);
    }

    fn handle_event(&mut self, event: &mut Event) {
        // ESC closes the help window
        if event.what == EventType::Keyboard && event.key_code == KB_ESC {
            self.window.end_modal(CM_CANCEL);
            event.clear();
            return;
        }

        // Viewer handles most events (scrolling)
        self.viewer.handle_event(event);

        // Window handles frame events (resize, move, etc.)
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
        self.viewer.set_state(state);
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
