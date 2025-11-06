// (C) 2025 - Enzo Lombardi

//! History view - dropdown button control for accessing input line history.
// History - Dropdown button for InputLine history
//
// Matches Borland: THistory (history dropdown button)
//
// A small button (shows '▼') attached to the right side of an InputLine.
// When clicked, displays a HistoryWindow with previous entries.
//
// Usage:
//   let history = History::new(Point::new(x, y), history_id);
//   // Position it to the right of the InputLine

use crate::core::geometry::{Point, Rect};
use crate::core::event::{Event, EventType, MB_LEFT_BUTTON};
use crate::core::palette::colors;
use crate::core::draw::DrawBuffer;
use crate::core::state::StateFlags;
use crate::core::history::HistoryManager;
use crate::terminal::Terminal;
use super::view::{View, write_line_to_terminal};
use super::history_window::HistoryWindow;

/// History - Dropdown button for accessing input history
///
/// Matches Borland: THistory
pub struct History {
    bounds: Rect,
    history_id: u16,
    state: StateFlags,
    pub selected_item: Option<String>, // Public so InputLine can read it
}

impl History {
    /// Create a new history button
    ///
    /// The button is 2 characters wide (shows '▼' or similar).
    pub fn new(pos: Point, history_id: u16) -> Self {
        Self {
            bounds: Rect::new(pos.x, pos.y, pos.x + 2, pos.y + 1),
            history_id,
            state: 0,
            selected_item: None,
        }
    }

    /// Check if this history list has any items
    pub fn has_items(&self) -> bool {
        HistoryManager::has_history(self.history_id)
    }

    /// Show the history window and let user select an item
    #[allow(dead_code)]
    fn show_history(&mut self, terminal: &mut Terminal) {
        if !self.has_items() {
            return;
        }

        // Create history window slightly below and to the left of the button
        let window_pos = Point::new(
            (self.bounds.a.x - 20).max(0),
            self.bounds.a.y + 1,
        );

        let mut window = HistoryWindow::new(window_pos, self.history_id, 30);

        if let Some(selected) = window.execute(terminal) {
            self.selected_item = Some(selected);
        }
    }
}

impl View for History {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let mut buf = DrawBuffer::new(2);

        // Draw down arrow: ▼ (or use 'v' for ASCII-only)
        let arrow = if self.has_items() { "▼" } else { " " };

        let color = if self.is_focused() {
            colors::BUTTON_SELECTED
        } else {
            colors::BUTTON_NORMAL
        };

        buf.move_str(0, arrow, color);

        write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y, &buf);
    }

    fn handle_event(&mut self, event: &mut Event) {
        match event.what {
            EventType::MouseDown => {
                if self.bounds.contains(event.mouse.pos) && event.mouse.buttons & MB_LEFT_BUTTON != 0 {
                    // Button clicked - show history window
                    // We need terminal access, which will be handled by parent
                    event.clear();
                }
            }
            _ => {}
        }
    }

    fn can_focus(&self) -> bool {
        false // History button doesn't take focus
    }

    fn state(&self) -> StateFlags {
        self.state
    }

    fn set_state(&mut self, state: StateFlags) {
        self.state = state;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_button_creation() {
        HistoryManager::clear_all();

        let button = History::new(Point::new(20, 5), 1);
        assert!(!button.has_items());
        assert_eq!(button.bounds.width(), 2);
    }

    #[test]
    fn test_history_button_with_items() {
        HistoryManager::clear_all();
        HistoryManager::add(2, "test".to_string());

        let button = History::new(Point::new(20, 5), 2);
        assert!(button.has_items());
    }
}
