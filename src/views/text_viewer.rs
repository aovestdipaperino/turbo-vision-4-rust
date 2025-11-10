// (C) 2025 - Enzo Lombardi

//! TextViewer view - scrollable text display for viewing large text content.

use crate::core::geometry::{Point, Rect};
use crate::core::event::{Event, EventType, KB_UP, KB_DOWN, KB_LEFT, KB_RIGHT, KB_PGUP, KB_PGDN, KB_HOME, KB_END};
use crate::core::draw::DrawBuffer;
use crate::terminal::Terminal;
use super::view::{View, write_line_to_terminal};
use super::scrollbar::ScrollBar;
use super::indicator::Indicator;
use std::cmp::min;

/// TextViewer displays text content with scrolling support.
/// Useful for viewing files, logs, or any multi-line text.
pub struct TextViewer {
    bounds: Rect,
    lines: Vec<String>,
    delta: Point,           // Current scroll offset
    cursor: Point,          // Current cursor position (0-based)
    h_scrollbar: Option<Box<ScrollBar>>,
    v_scrollbar: Option<Box<ScrollBar>>,
    indicator: Option<Box<Indicator>>,
    show_line_numbers: bool,
    owner: Option<*const dyn View>,
}

impl TextViewer {
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            lines: Vec::new(),
            delta: Point::zero(),
            cursor: Point::zero(),
            h_scrollbar: None,
            v_scrollbar: None,
            indicator: None,
            show_line_numbers: false,
            owner: None,
        }
    }

    /// Create a TextViewer with scrollbars
    pub fn with_scrollbars(mut self, add_scrollbars: bool) -> Self {
        if add_scrollbars {
            // Vertical scrollbar on the right edge
            let v_bounds = Rect::new(
                self.bounds.b.x - 1,
                self.bounds.a.y + 1,  // Below indicator
                self.bounds.b.x,
                self.bounds.b.y - 1,  // Above horizontal scrollbar
            );
            self.v_scrollbar = Some(Box::new(ScrollBar::new_vertical(v_bounds)));

            // Horizontal scrollbar on the bottom edge
            let h_bounds = Rect::new(
                self.bounds.a.x,
                self.bounds.b.y - 1,
                self.bounds.b.x - 1,  // Before vertical scrollbar
                self.bounds.b.y,
            );
            self.h_scrollbar = Some(Box::new(ScrollBar::new_horizontal(h_bounds)));
        }
        self
    }

    /// Create a TextViewer with indicator
    pub fn with_indicator(mut self, add_indicator: bool) -> Self {
        if add_indicator {
            let indicator_bounds = Rect::new(
                self.bounds.a.x,
                self.bounds.a.y,
                self.bounds.b.x,
                self.bounds.a.y + 1,
            );
            self.indicator = Some(Box::new(Indicator::new(indicator_bounds)));
        }
        self
    }

    pub fn set_show_line_numbers(&mut self, show: bool) {
        self.show_line_numbers = show;
    }

    /// Set the text content
    pub fn set_text(&mut self, text: &str) {
        self.lines = text.lines().map(|s| s.to_string()).collect();
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.cursor = Point::zero();
        self.delta = Point::zero();
        self.update_scrollbars();
        self.update_indicator();
    }

    /// Load text from a file
    pub fn load_file(&mut self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        let content = std::fs::read_to_string(path.as_ref())?;
        self.set_text(&content);
        Ok(())
    }

    /// Get the maximum line length
    fn max_line_length(&self) -> i16 {
        self.lines
            .iter()
            .map(|line| line.len() as i16)
            .max()
            .unwrap_or(0)
    }

    /// Get the visible area (excluding scrollbars and indicator)
    fn get_content_area(&self) -> Rect {
        let mut area = self.bounds;

        // Account for indicator at top
        if self.indicator.is_some() {
            area.a.y += 1;
        }

        // Account for scrollbars
        if self.v_scrollbar.is_some() {
            area.b.x -= 1;
        }
        if self.h_scrollbar.is_some() {
            area.b.y -= 1;
        }

        area
    }

    fn update_scrollbars(&mut self) {
        let content_area = self.get_content_area();
        let max_x = self.max_line_length();
        let max_y = self.lines.len() as i16;

        if let Some(ref mut h_bar) = self.h_scrollbar {
            h_bar.set_params(
                self.delta.x as i32,
                0,
                max_x.saturating_sub(content_area.width()) as i32,
                content_area.width() as i32,
                1,
            );
        }

        if let Some(ref mut v_bar) = self.v_scrollbar {
            v_bar.set_params(
                self.delta.y as i32,
                0,
                max_y.saturating_sub(content_area.height()) as i32,
                content_area.height() as i32,
                1,
            );
        }
    }

    fn update_indicator(&mut self) {
        if let Some(ref mut indicator) = self.indicator {
            // Display 1-based line and column
            indicator.set_value(
                Point::new(self.cursor.x + 1, self.cursor.y + 1),
                false,
            );
        }
    }

    fn scroll_to(&mut self, x: i16, y: i16) {
        let content_area = self.get_content_area();
        let max_x = self.max_line_length().saturating_sub(content_area.width());
        let max_y = (self.lines.len() as i16).saturating_sub(content_area.height());

        self.delta.x = x.max(0).min(max_x.max(0));
        self.delta.y = y.max(0).min(max_y.max(0));

        // Update cursor to match scroll position (for indicator display)
        self.cursor.x = self.delta.x;
        self.cursor.y = self.delta.y;

        self.update_scrollbars();
        self.update_indicator();
    }
}

impl View for TextViewer {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;

        // Update scrollbar positions
        if let Some(ref mut v_bar) = self.v_scrollbar {
            let v_bounds = Rect::new(
                bounds.b.x - 1,
                bounds.a.y + if self.indicator.is_some() { 1 } else { 0 },
                bounds.b.x,
                bounds.b.y - if self.h_scrollbar.is_some() { 1 } else { 0 },
            );
            v_bar.set_bounds(v_bounds);
        }

        if let Some(ref mut h_bar) = self.h_scrollbar {
            let h_bounds = Rect::new(
                bounds.a.x,
                bounds.b.y - 1,
                bounds.b.x - if self.v_scrollbar.is_some() { 1 } else { 0 },
                bounds.b.y,
            );
            h_bar.set_bounds(h_bounds);
        }

        if let Some(ref mut indicator) = self.indicator {
            let indicator_bounds = Rect::new(
                bounds.a.x,
                bounds.a.y,
                bounds.b.x,
                bounds.a.y + 1,
            );
            indicator.set_bounds(indicator_bounds);
        }

        self.update_scrollbars();
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let content_area = self.get_content_area();
        let width = content_area.width() as usize;
        let height = content_area.height() as usize;

        // Line number width (if shown)
        let line_num_width = if self.show_line_numbers {
            5  // " 999 "
        } else {
            0
        };

        for y in 0..height {
            use crate::core::palette::colors::DIALOG_NORMAL;
            let line_idx = (self.delta.y + y as i16) as usize;
            let mut buf = DrawBuffer::new(width);

            // Fill with spaces
            buf.move_char(0, ' ', DIALOG_NORMAL, width);

            if line_idx < self.lines.len() {
                let line = &self.lines[line_idx];
                let mut x_offset = 0;

                // Draw line number if enabled
                if self.show_line_numbers {
                    let line_num = format!("{:4} ", line_idx + 1);
                    buf.move_str(0, &line_num, DIALOG_NORMAL);
                    x_offset = line_num_width;
                }

                // Draw line content
                let start_col = self.delta.x as usize;
                if start_col < line.len() {
                    let visible_width = width - x_offset;
                    let end_col = min(start_col + visible_width, line.len());
                    let visible_text = &line[start_col..end_col];
                    buf.move_str(x_offset, visible_text, DIALOG_NORMAL);
                }
            }

            write_line_to_terminal(
                terminal,
                content_area.a.x,
                content_area.a.y + y as i16,
                &buf,
            );
        }

        // Draw indicator
        if let Some(ref mut indicator) = self.indicator {
            indicator.draw(terminal);
        }

        // Draw scrollbars
        if let Some(ref mut h_bar) = self.h_scrollbar {
            h_bar.draw(terminal);
        }
        if let Some(ref mut v_bar) = self.v_scrollbar {
            v_bar.draw(terminal);
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        match event.what {
            EventType::Keyboard => {
                let content_area = self.get_content_area();

                match event.key_code {
                    KB_UP => {
                        self.scroll_to(self.delta.x, self.delta.y - 1);
                        event.clear();
                    }
                    KB_DOWN => {
                        self.scroll_to(self.delta.x, self.delta.y + 1);
                        event.clear();
                    }
                    KB_LEFT => {
                        self.scroll_to(self.delta.x - 1, self.delta.y);
                        event.clear();
                    }
                    KB_RIGHT => {
                        self.scroll_to(self.delta.x + 1, self.delta.y);
                        event.clear();
                    }
                    KB_PGUP => {
                        self.scroll_to(self.delta.x, self.delta.y - content_area.height());
                        event.clear();
                    }
                    KB_PGDN => {
                        self.scroll_to(self.delta.x, self.delta.y + content_area.height());
                        event.clear();
                    }
                    KB_HOME => {
                        self.scroll_to(0, self.delta.y);
                        event.clear();
                    }
                    KB_END => {
                        self.scroll_to(self.max_line_length(), self.delta.y);
                        event.clear();
                    }
                    _ => {}
                }
            }
            EventType::MouseWheelUp => {
                let mouse_pos = event.mouse.pos;
                let content_area = self.get_content_area();
                // Check if mouse is within the text viewer content area
                if mouse_pos.x >= content_area.a.x && mouse_pos.x < content_area.b.x &&
                   mouse_pos.y >= content_area.a.y && mouse_pos.y < content_area.b.y {
                    self.scroll_to(self.delta.x, self.delta.y - 1);
                    event.clear();
                }
            }
            EventType::MouseWheelDown => {
                let mouse_pos = event.mouse.pos;
                let content_area = self.get_content_area();
                // Check if mouse is within the text viewer content area
                if mouse_pos.x >= content_area.a.x && mouse_pos.x < content_area.b.x &&
                   mouse_pos.y >= content_area.a.y && mouse_pos.y < content_area.b.y {
                    self.scroll_to(self.delta.x, self.delta.y + 1);
                    event.clear();
                }
            }
            _ => {}
        }

        // Let scrollbars handle events too
        let old_delta = self.delta;

        if let Some(ref mut h_bar) = self.h_scrollbar {
            h_bar.handle_event(event);
            self.delta.x = h_bar.get_value() as i16;
        }

        if let Some(ref mut v_bar) = self.v_scrollbar {
            v_bar.handle_event(event);
            self.delta.y = v_bar.get_value() as i16;
        }

        if old_delta != self.delta {
            event.clear();
        }
    }

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        None  // TextViewer uses hardcoded dialog colors
    }
}

/// Builder for creating text viewers with a fluent API.
pub struct TextViewerBuilder {
    bounds: Option<Rect>,
    with_scrollbars: bool,
    with_indicator: bool,
    show_line_numbers: bool,
}

impl TextViewerBuilder {
    pub fn new() -> Self {
        Self {
            bounds: None,
            with_scrollbars: false,
            with_indicator: false,
            show_line_numbers: false,
        }
    }

    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    #[must_use]
    pub fn with_scrollbars(mut self, with_scrollbars: bool) -> Self {
        self.with_scrollbars = with_scrollbars;
        self
    }

    #[must_use]
    pub fn with_indicator(mut self, with_indicator: bool) -> Self {
        self.with_indicator = with_indicator;
        self
    }

    #[must_use]
    pub fn show_line_numbers(mut self, show_line_numbers: bool) -> Self {
        self.show_line_numbers = show_line_numbers;
        self
    }

    pub fn build(self) -> TextViewer {
        let bounds = self.bounds.expect("TextViewer bounds must be set");
        let mut viewer = TextViewer::new(bounds)
            .with_scrollbars(self.with_scrollbars)
            .with_indicator(self.with_indicator);
        viewer.set_show_line_numbers(self.show_line_numbers);
        viewer
    }

    pub fn build_boxed(self) -> Box<TextViewer> {
        Box::new(self.build())
    }
}

impl Default for TextViewerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
