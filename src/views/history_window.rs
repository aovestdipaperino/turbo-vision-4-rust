// (C) 2025 - Enzo Lombardi
// HistoryWindow - Popup window displaying history items
//
// Matches Borland: THistoryWindow (modal window with THistoryViewer)
//
// A modal popup window that displays history items and allows selection.
// Returns the selected history item when dismissed with Enter.
//
// Usage:
//   let mut window = HistoryWindow::new(Point::new(10, 5), history_id, 15);
//   if let Some(selected) = window.execute(terminal) {
//       // User selected an item
//   }

use crate::core::geometry::{Point, Rect};
use crate::core::event::{EventType, KB_ENTER, KB_ESC};
use crate::terminal::Terminal;
use super::history_viewer::HistoryViewer;
use super::view::View;
use super::window::Window;

/// HistoryWindow - Modal popup for selecting from history
///
/// Matches Borland: THistoryWindow
pub struct HistoryWindow {
    window: Window,
    viewer: HistoryViewer,
}

impl HistoryWindow {
    /// Create a new history window at the given position
    ///
    /// # Arguments
    /// * `pos` - Top-left position of the window
    /// * `history_id` - History list ID to display
    /// * `width` - Width of the window (height auto-calculated based on items, max 10)
    pub fn new(pos: Point, history_id: u16, width: i16) -> Self {
        use crate::core::history::HistoryManager;

        // Calculate height based on number of items (max 10, min 3)
        let item_count = HistoryManager::count(history_id);
        let viewer_height = item_count.min(10).max(1) as i16;
        let window_height = viewer_height + 2; // +2 for frame

        let window_bounds = Rect::new(pos.x, pos.y, pos.x + width, pos.y + window_height);
        let viewer_bounds = Rect::new(1, 1, width - 2, viewer_height + 1);

        let window = Window::new(window_bounds, "History");
        let mut viewer = HistoryViewer::new(viewer_bounds, history_id);

        // Focus the viewer
        viewer.set_focus(true);

        Self { window, viewer }
    }

    /// Execute the history window modally
    ///
    /// Returns the selected history item, or None if cancelled.
    pub fn execute(&mut self, terminal: &mut Terminal) -> Option<String> {
        loop {
            // Draw window and viewer
            self.window.draw(terminal);
            self.viewer.draw(terminal);
            let _ = terminal.flush();

            // Handle events
            if let Ok(Some(mut event)) = terminal.poll_event(std::time::Duration::from_millis(50)) {
                // Let viewer handle navigation first
                self.viewer.handle_event(&mut event);

                // Handle Enter and Esc
                match event.what {
                    EventType::Keyboard => {
                        if event.key_code == KB_ENTER {
                            // Return selected item
                            return self.viewer.get_selected_item().map(|s| s.to_string());
                        } else if event.key_code == KB_ESC {
                            // Cancel
                            return None;
                        }
                    }
                    EventType::MouseDown => {
                        // Check if double-click on viewer
                        if event.mouse.double_click && self.viewer.bounds().contains(event.mouse.pos) {
                            return self.viewer.get_selected_item().map(|s| s.to_string());
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::history::HistoryManager;

    #[test]
    fn test_history_window_creation() {
        HistoryManager::clear_all();
        HistoryManager::add(10, "test1".to_string());
        HistoryManager::add(10, "test2".to_string());
        HistoryManager::add(10, "test3".to_string());

        let window = HistoryWindow::new(Point::new(10, 5), 10, 30);

        // Window should be sized based on items
        assert_eq!(window.viewer.item_count(), 3);
    }

    #[test]
    fn test_history_window_empty() {
        HistoryManager::clear_all();

        let window = HistoryWindow::new(Point::new(10, 5), 99, 30);
        assert_eq!(window.viewer.item_count(), 0);
    }

    #[test]
    fn test_history_window_many_items() {
        HistoryManager::clear_all();

        // Add 15 items
        for i in 1..=15 {
            HistoryManager::add(20, format!("item{}", i));
        }

        let window = HistoryWindow::new(Point::new(10, 5), 20, 30);

        // Should have all 15 items but viewer height capped at 10
        assert_eq!(window.viewer.item_count(), 15);
        // Viewer bounds height should be at most 10
        let viewer_height = window.viewer.bounds().height();
        assert!(viewer_height >= 1 && viewer_height <= 11, "viewer height was {}", viewer_height);
    }
}
