// (C) 2025 - Enzo Lombardi

//! Test utilities for applications using turbo-vision.
//!
//! This module provides mock implementations and testing helpers that are only
//! available when the `test-util` feature is enabled.
//!
//! # Examples
//!
//! Enable the feature in your `Cargo.toml`:
//! ```toml
//! [dev-dependencies]
//! turbo-vision = { version = "0.3", features = ["test-util"] }
//! ```
//!
//! Then use the mock terminal in tests:
//! ```rust
//! use turbo_vision::test_util::MockTerminal;
//! use turbo_vision::core::geometry::Rect;
//!
//! let mut terminal = MockTerminal::new(80, 25);
//! terminal.put_char(0, 0, 'H');
//! assert_eq!(terminal.get_char(0, 0), Some('H'));
//! ```

use crate::core::draw::Cell;
use crate::core::event::Event;
use crate::core::geometry::{Point, Rect};
use crate::core::palette::Attr;
use std::time::Duration;

/// A mock terminal for testing UI components without a real terminal.
///
/// This allows you to test view rendering and event handling in unit tests.
///
/// # Examples
///
/// ```
/// use turbo_vision::test_util::MockTerminal;
/// use turbo_vision::core::palette::{Attr, TvColor};
///
/// let mut terminal = MockTerminal::new(80, 25);
///
/// // Write to the terminal
/// let attr = Attr::new(TvColor::White, TvColor::Blue);
/// terminal.put_char_with_attr(5, 10, 'X', attr);
///
/// // Verify the character was written
/// assert_eq!(terminal.get_char(5, 10), Some('X'));
/// ```
pub struct MockTerminal {
    width: u16,
    height: u16,
    buffer: Vec<Vec<Cell>>,
    cursor_pos: Point,
    cursor_visible: bool,
    events: Vec<Event>,
}

impl MockTerminal {
    /// Creates a new mock terminal with the specified dimensions.
    pub fn new(width: u16, height: u16) -> Self {
        use crate::core::palette::{Attr, TvColor};
        let default_attr = Attr::new(TvColor::LightGray, TvColor::Black);
        let default_cell = Cell::new(' ', default_attr);
        let buffer = vec![vec![default_cell; width as usize]; height as usize];

        Self {
            width,
            height,
            buffer,
            cursor_pos: Point::zero(),
            cursor_visible: false,
            events: Vec::new(),
        }
    }

    /// Gets the terminal size as (width, height).
    pub fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// Puts a character at the specified position with default attributes.
    pub fn put_char(&mut self, x: i16, y: i16, ch: char) {
        if self.in_bounds(x, y) {
            self.buffer[y as usize][x as usize].ch = ch;
        }
    }

    /// Puts a character at the specified position with custom attributes.
    pub fn put_char_with_attr(&mut self, x: i16, y: i16, ch: char, attr: Attr) {
        if self.in_bounds(x, y) {
            self.buffer[y as usize][x as usize] = Cell::new(ch, attr);
        }
    }

    /// Gets the character at the specified position.
    pub fn get_char(&self, x: i16, y: i16) -> Option<char> {
        if self.in_bounds(x, y) {
            Some(self.buffer[y as usize][x as usize].ch)
        } else {
            None
        }
    }

    /// Gets the full cell (character + attributes) at the specified position.
    pub fn get_cell(&self, x: i16, y: i16) -> Option<Cell> {
        if self.in_bounds(x, y) {
            Some(self.buffer[y as usize][x as usize])
        } else {
            None
        }
    }

    /// Checks if coordinates are within bounds.
    fn in_bounds(&self, x: i16, y: i16) -> bool {
        x >= 0 && y >= 0 && x < self.width as i16 && y < self.height as i16
    }

    /// Sets the cursor position.
    pub fn set_cursor(&mut self, x: i16, y: i16) {
        self.cursor_pos = Point::new(x, y);
    }

    /// Gets the current cursor position.
    pub fn cursor_pos(&self) -> Point {
        self.cursor_pos
    }

    /// Shows or hides the cursor.
    pub fn show_cursor(&mut self, visible: bool) {
        self.cursor_visible = visible;
    }

    /// Returns whether the cursor is visible.
    pub fn is_cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    /// Adds an event to the mock terminal's event queue.
    ///
    /// Use this to simulate user input in tests.
    pub fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    /// Polls for the next event (returns the first queued event).
    pub fn poll_event(&mut self, _timeout: Duration) -> Option<Event> {
        if self.events.is_empty() {
            None
        } else {
            Some(self.events.remove(0))
        }
    }

    /// Clears all pending events.
    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    /// Fills a rectangular region with a character.
    ///
    /// Useful for verifying that views draw to the expected region.
    pub fn fill_rect(&mut self, rect: Rect, ch: char, attr: Attr) {
        for y in rect.a.y..rect.b.y {
            for x in rect.a.x..rect.b.x {
                self.put_char_with_attr(x, y, ch, attr);
            }
        }
    }

    /// Returns the contents of a row as a String.
    ///
    /// Useful for assertions in tests.
    pub fn get_row(&self, y: i16) -> Option<String> {
        if y >= 0 && y < self.height as i16 {
            Some(
                self.buffer[y as usize]
                    .iter()
                    .map(|cell| cell.ch)
                    .collect()
            )
        } else {
            None
        }
    }

    /// Returns a rectangular region as a vector of strings (one per row).
    pub fn get_rect_text(&self, rect: Rect) -> Vec<String> {
        let mut result = Vec::new();
        for y in rect.a.y..rect.b.y {
            if y >= 0 && y < self.height as i16 {
                let row: String = (rect.a.x..rect.b.x)
                    .filter_map(|x| {
                        if x >= 0 && x < self.width as i16 {
                            Some(self.buffer[y as usize][x as usize].ch)
                        } else {
                            None
                        }
                    })
                    .collect();
                result.push(row);
            }
        }
        result
    }

    /// Clears the entire terminal (fills with spaces).
    pub fn clear(&mut self) {
        use crate::core::palette::{Attr, TvColor};
        let default_attr = Attr::new(TvColor::LightGray, TvColor::Black);
        let default_cell = Cell::new(' ', default_attr);
        for row in &mut self.buffer {
            for cell in row {
                *cell = default_cell;
            }
        }
    }
}

/// Compile-time assertions for Send trait.
///
/// These tests ensure that key types can be safely sent between threads.
/// They compile but don't need to run.
#[cfg(test)]
mod send_assertions {
    use crate::core::geometry::{Point, Rect};
    use crate::core::event::Event;
    use crate::core::palette::{Attr, TvColor};
    use crate::core::draw::Cell;

    fn assert_send<T: Send>() {}

    #[test]
    #[allow(dead_code)]
    fn test_core_types_are_send() {
        // Geometry types
        assert_send::<Point>();
        assert_send::<Rect>();

        // Event types
        assert_send::<Event>();

        // Palette types
        assert_send::<Attr>();
        assert_send::<TvColor>();

        // Draw types
        assert_send::<Cell>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::palette::{Attr, TvColor};

    #[test]
    fn test_mock_terminal_basic() {
        let mut terminal = MockTerminal::new(80, 25);
        assert_eq!(terminal.size(), (80, 25));

        terminal.put_char(5, 10, 'X');
        assert_eq!(terminal.get_char(5, 10), Some('X'));
    }

    #[test]
    fn test_mock_terminal_attributes() {
        let mut terminal = MockTerminal::new(80, 25);
        let attr = Attr::new(TvColor::White, TvColor::Blue);

        terminal.put_char_with_attr(0, 0, 'A', attr);

        let cell = terminal.get_cell(0, 0).unwrap();
        assert_eq!(cell.ch, 'A');
        assert_eq!(cell.attr, attr);
    }

    #[test]
    fn test_mock_terminal_bounds() {
        let terminal = MockTerminal::new(10, 10);

        assert_eq!(terminal.get_char(0, 0), Some(' '));
        assert_eq!(terminal.get_char(9, 9), Some(' '));
        assert_eq!(terminal.get_char(10, 10), None);
        assert_eq!(terminal.get_char(-1, -1), None);
    }

    #[test]
    fn test_mock_terminal_cursor() {
        let mut terminal = MockTerminal::new(80, 25);

        terminal.set_cursor(10, 5);
        assert_eq!(terminal.cursor_pos(), Point::new(10, 5));

        terminal.show_cursor(true);
        assert!(terminal.is_cursor_visible());

        terminal.show_cursor(false);
        assert!(!terminal.is_cursor_visible());
    }

    #[test]
    fn test_mock_terminal_events() {
        let mut terminal = MockTerminal::new(80, 25);

        let event1 = Event::keyboard(0x1B);
        let event2 = Event::keyboard(0x0D);

        terminal.push_event(event1);
        terminal.push_event(event2);

        assert_eq!(terminal.poll_event(Duration::from_millis(0)).unwrap().key_code, 0x1B);
        assert_eq!(terminal.poll_event(Duration::from_millis(0)).unwrap().key_code, 0x0D);
        assert!(terminal.poll_event(Duration::from_millis(0)).is_none());
    }

    #[test]
    fn test_mock_terminal_get_row() {
        let mut terminal = MockTerminal::new(10, 5);

        for (i, ch) in "Hello".chars().enumerate() {
            terminal.put_char(i as i16, 0, ch);
        }

        let row = terminal.get_row(0).unwrap();
        assert!(row.starts_with("Hello"));
    }

    #[test]
    fn test_mock_terminal_fill_rect() {
        let mut terminal = MockTerminal::new(20, 10);
        let attr = Attr::new(TvColor::White, TvColor::Blue);

        terminal.fill_rect(Rect::new(5, 5, 10, 8), 'X', attr);

        // Inside the rect
        assert_eq!(terminal.get_char(5, 5), Some('X'));
        assert_eq!(terminal.get_char(9, 7), Some('X'));

        // Outside the rect
        assert_eq!(terminal.get_char(4, 5), Some(' '));
        assert_eq!(terminal.get_char(10, 8), Some(' '));
    }
}
