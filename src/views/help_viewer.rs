// (C) 2025 - Enzo Lombardi

//! HelpViewer view - scrollable help content viewer with cross-reference navigation.
// HelpViewer - Help content viewer based on TextView
//
// Matches Borland: THelpViewer (help.h)
//
// Displays help topic content with scrolling support.

use crate::core::geometry::{Rect, Point};
use crate::core::event::{Event, EventType, KB_UP, KB_DOWN, KB_PGUP, KB_PGDN, KB_HOME, KB_END};
use crate::core::state::{StateFlags, SF_FOCUSED};
use crate::core::palette::{Attr, TvColor};
use crate::core::draw::DrawBuffer;
use crate::terminal::Terminal;
use super::view::{View, write_line_to_terminal};
use super::scrollbar::ScrollBar;
use super::help_file::{HelpTopic};

/// HelpViewer - Displays help topic content
///
/// Matches Borland: THelpViewer
pub struct HelpViewer {
    bounds: Rect,
    state: StateFlags,
    delta: Point,           // Current scroll offset
    limit: Point,           // Maximum scroll values
    vscrollbar: Option<Box<ScrollBar>>,
    lines: Vec<String>,
    current_topic: Option<String>,
    owner: Option<*const dyn View>,
}

impl HelpViewer {
    /// Create a new help viewer
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            state: 0,
            delta: Point::new(0, 0),
            limit: Point::new(0, 0),
            vscrollbar: None,
            lines: Vec::new(),
            current_topic: None,
            owner: None,
        }
    }

    /// Create a help viewer with scrollbar
    pub fn with_scrollbar(mut self) -> Self {
        let sb_bounds = Rect::new(
            self.bounds.b.x - 1,
            self.bounds.a.y,
            self.bounds.b.x,
            self.bounds.b.y,
        );
        self.vscrollbar = Some(Box::new(ScrollBar::new_vertical(sb_bounds)));
        self
    }

    /// Set the help topic to display
    pub fn set_topic(&mut self, topic: &HelpTopic) {
        self.lines = topic.get_formatted_content();
        self.current_topic = Some(topic.id.clone());

        // Update limits
        let max_y = if self.lines.len() > self.bounds.height() as usize {
            self.lines.len() as i16 - self.bounds.height()
        } else {
            0
        };
        self.limit = Point::new(self.bounds.width(), max_y);
        self.delta = Point::new(0, 0);

        self.update_scrollbar();
    }

    /// Get the current topic ID
    pub fn current_topic(&self) -> Option<&str> {
        self.current_topic.as_deref()
    }

    /// Clear the viewer
    pub fn clear(&mut self) {
        self.lines.clear();
        self.current_topic = None;
        self.limit = Point::new(0, 0);
        self.delta = Point::new(0, 0);
        self.update_scrollbar();
    }

    /// Update scrollbar position
    fn update_scrollbar(&mut self) {
        if let Some(ref mut sb) = self.vscrollbar {
            let size = self.bounds.height();

            sb.set_params(
                self.delta.y as i32,
                0,
                self.limit.y as i32,
                (size - 1) as i32,
                1,
            );
        }
    }

    /// Scroll by delta
    fn scroll_by(&mut self, dx: i16, dy: i16) {
        let new_x = (self.delta.x + dx).max(0).min(self.limit.x);
        let new_y = (self.delta.y + dy).max(0).min(self.limit.y);

        self.delta = Point::new(new_x, new_y);
        self.update_scrollbar();
    }
}

impl View for HelpViewer {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;

        // Update scrollbar position if present
        if self.vscrollbar.is_some() {
            let sb_bounds = Rect::new(
                bounds.b.x - 1,
                bounds.a.y,
                bounds.b.x,
                bounds.b.y,
            );
            if let Some(ref mut sb) = self.vscrollbar {
                sb.set_bounds(sb_bounds);
            }
        }

        // Recalculate limits
        let max_y = if self.lines.len() > self.bounds.height() as usize {
            self.lines.len() as i16 - self.bounds.height()
        } else {
            0
        };
        self.limit = Point::new(self.bounds.width(), max_y);
        self.update_scrollbar();
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let start_line = self.delta.y as usize;

        // Determine display width (leave room for scrollbar if present)
        let display_width = if self.vscrollbar.is_some() {
            (self.bounds.width() - 1) as usize
        } else {
            self.bounds.width() as usize
        };

        // Determine color based on focus
        let color = if self.state & SF_FOCUSED != 0 {
            Attr::new(TvColor::Black, TvColor::White)
        } else {
            Attr::new(TvColor::Black, TvColor::LightGray)
        };

        for row in 0..self.bounds.height() {
            let line_idx = start_line + row as usize;
            let line = if line_idx < self.lines.len() {
                &self.lines[line_idx]
            } else {
                ""
            };

            let mut buf = DrawBuffer::new(display_width);
            buf.move_char(0, ' ', color, display_width);
            buf.move_str(0, line, color);
            write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + row, &buf);
        }

        // Draw scrollbar if present
        if let Some(ref mut sb) = self.vscrollbar {
            sb.draw(terminal);
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        if event.what != EventType::Keyboard {
            return;
        }

        let page_size = self.bounds.height();

        match event.key_code {
            KB_UP => {
                self.scroll_by(0, -1);
                event.clear();
            }
            KB_DOWN => {
                self.scroll_by(0, 1);
                event.clear();
            }
            KB_PGUP => {
                self.scroll_by(0, -(page_size - 1));
                event.clear();
            }
            KB_PGDN => {
                self.scroll_by(0, page_size - 1);
                event.clear();
            }
            KB_HOME => {
                self.delta = Point::new(0, 0);
                self.update_scrollbar();
                event.clear();
            }
            KB_END => {
                self.delta = Point::new(0, self.limit.y);
                self.update_scrollbar();
                event.clear();
            }
            _ => {}
        }
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn state(&self) -> StateFlags {
        self.state
    }

    fn set_state(&mut self, state: StateFlags) {
        self.state = state;
    }

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        None  // HelpViewer uses hardcoded colors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::help_file::HelpTopic;

    #[test]
    fn test_help_viewer_creation() {
        let bounds = Rect::new(0, 0, 80, 25);
        let viewer = HelpViewer::new(bounds);

        assert_eq!(viewer.bounds(), bounds);
        assert!(viewer.current_topic().is_none());
        assert!(viewer.can_focus());
    }

    #[test]
    fn test_help_viewer_with_scrollbar() {
        let bounds = Rect::new(0, 0, 80, 25);
        let viewer = HelpViewer::new(bounds).with_scrollbar();

        assert!(viewer.vscrollbar.is_some());
    }

    #[test]
    fn test_set_topic() {
        let bounds = Rect::new(0, 0, 80, 25);
        let mut viewer = HelpViewer::new(bounds);

        let mut topic = HelpTopic::new("test".to_string(), "Test Topic".to_string());
        topic.add_line("Line 1".to_string());
        topic.add_line("Line 2".to_string());

        viewer.set_topic(&topic);

        assert_eq!(viewer.current_topic(), Some("test"));
        assert!(viewer.lines.len() > 0);
    }

    #[test]
    fn test_clear() {
        let bounds = Rect::new(0, 0, 80, 25);
        let mut viewer = HelpViewer::new(bounds);

        let topic = HelpTopic::new("test".to_string(), "Test".to_string());
        viewer.set_topic(&topic);
        assert!(viewer.current_topic().is_some());

        viewer.clear();
        assert!(viewer.current_topic().is_none());
        assert_eq!(viewer.lines.len(), 0);
    }
}
