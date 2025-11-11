// (C) 2025 - Enzo Lombardi

//! Terminal Widget - scrolling output viewer for program output, logs, and build results.
//!
//! Matches Borland: TTerminal (from Turbo Vision Professional)
//!
//! This is different from the Terminal backend - this is a UI widget for displaying
//! scrolling text output like:
//! - Build output from compilers
//! - Program execution logs
//! - Debug console output
//! - Command line tool output
//!
//! Key features:
//! - Auto-scroll to bottom when new lines are added
//! - Large scrollback buffer (configurable)
//! - Efficient append operations
//! - Optional ANSI color code support
//! - Read-only (unlike Editor)

use crate::core::geometry::Rect;
use crate::core::event::{Event, EventType, KB_UP, KB_DOWN, KB_PGUP, KB_PGDN, KB_HOME, KB_END};
use crate::core::draw::DrawBuffer;
use crate::core::palette::{colors, Attr};
use crate::core::state::StateFlags;
use crate::terminal::Terminal;
use super::view::{View, write_line_to_terminal};
use super::scrollbar::ScrollBar;

/// A line of output with optional color attributes
#[derive(Clone, Debug)]
pub struct OutputLine {
    /// The text content
    pub text: String,
    /// Optional color attribute (if None, uses default)
    pub attr: Option<Attr>,
}

impl OutputLine {
    /// Create a new output line with default color
    pub fn new(text: String) -> Self {
        Self { text, attr: None }
    }

    /// Create a new output line with specific color
    pub fn with_attr(text: String, attr: Attr) -> Self {
        Self { text, attr: Some(attr) }
    }
}

/// Terminal Widget - scrolling output viewer
/// Matches Borland: TTerminal
pub struct TerminalWidget {
    bounds: Rect,
    state: StateFlags,
    /// Output lines buffer
    lines: Vec<OutputLine>,
    /// Maximum number of lines to keep (scrollback buffer)
    max_lines: usize,
    /// Current scroll position (top visible line)
    top_line: usize,
    /// Auto-scroll to bottom when new lines added
    auto_scroll: bool,
    /// Vertical scrollbar
    v_scrollbar: Option<Box<ScrollBar>>,
    owner: Option<*const dyn View>,
    owner_type: super::view::OwnerType,
}

impl TerminalWidget {
    /// Create a new terminal widget
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            state: 0,
            lines: Vec::new(),
            max_lines: 10000,  // Default: 10k lines scrollback
            top_line: 0,
            auto_scroll: true,
            v_scrollbar: None,
            owner: None,
            owner_type: super::view::OwnerType::None,
        }
    }

    /// Create with vertical scrollbar
    pub fn with_scrollbar(mut self) -> Self {
        let v_bounds = Rect::new(
            self.bounds.b.x - 1,
            self.bounds.a.y,
            self.bounds.b.x,
            self.bounds.b.y,
        );
        self.v_scrollbar = Some(Box::new(ScrollBar::new_vertical(v_bounds)));
        self
    }

    /// Set the maximum scrollback buffer size
    pub fn set_max_lines(&mut self, max_lines: usize) {
        self.max_lines = max_lines;
        self.trim_buffer();
    }

    /// Enable/disable auto-scroll to bottom
    pub fn set_auto_scroll(&mut self, auto_scroll: bool) {
        self.auto_scroll = auto_scroll;
    }

    /// Append a line of output
    pub fn append_line(&mut self, text: String) {
        self.lines.push(OutputLine::new(text));
        self.trim_buffer();

        if self.auto_scroll {
            self.scroll_to_bottom();
        }

        self.update_scrollbar();
    }

    /// Append a line with specific color
    pub fn append_line_colored(&mut self, text: String, attr: Attr) {
        self.lines.push(OutputLine::with_attr(text, attr));
        self.trim_buffer();

        if self.auto_scroll {
            self.scroll_to_bottom();
        }

        self.update_scrollbar();
    }

    /// Append multiple lines at once
    pub fn append_lines(&mut self, lines: Vec<String>) {
        for line in lines {
            self.lines.push(OutputLine::new(line));
        }
        self.trim_buffer();

        if self.auto_scroll {
            self.scroll_to_bottom();
        }

        self.update_scrollbar();
    }

    /// Append text, splitting on newlines
    pub fn append_text(&mut self, text: &str) {
        for line in text.lines() {
            self.lines.push(OutputLine::new(line.to_string()));
        }
        self.trim_buffer();

        if self.auto_scroll {
            self.scroll_to_bottom();
        }

        self.update_scrollbar();
    }

    /// Clear all output
    pub fn clear(&mut self) {
        self.lines.clear();
        self.top_line = 0;
        self.update_scrollbar();
    }

    /// Get the number of lines
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Scroll to the bottom
    pub fn scroll_to_bottom(&mut self) {
        let visible_rows = self.get_visible_rows();
        if self.lines.len() > visible_rows {
            self.top_line = self.lines.len() - visible_rows;
        } else {
            self.top_line = 0;
        }
    }

    /// Scroll to the top
    pub fn scroll_to_top(&mut self) {
        self.top_line = 0;
    }

    /// Trim buffer to max_lines
    fn trim_buffer(&mut self) {
        if self.lines.len() > self.max_lines {
            let excess = self.lines.len() - self.max_lines;
            self.lines.drain(0..excess);

            // Adjust scroll position
            if self.top_line >= excess {
                self.top_line -= excess;
            } else {
                self.top_line = 0;
            }
        }
    }

    /// Get the number of visible rows
    fn get_visible_rows(&self) -> usize {
        let mut height = self.bounds.height() as usize;
        if self.v_scrollbar.is_some() {
            // Account for scrollbar taking up space
            height = height.saturating_sub(0); // scrollbar doesn't reduce height
        }
        height
    }

    /// Get the visible width
    fn get_visible_width(&self) -> usize {
        let mut width = self.bounds.width() as usize;
        if self.v_scrollbar.is_some() {
            width = width.saturating_sub(1); // scrollbar takes 1 column
        }
        width
    }

    /// Update scrollbar state
    fn update_scrollbar(&mut self) {
        // Compute all values before borrowing v_scrollbar mutably
        let visible_rows = self.get_visible_rows();
        let total_lines = self.lines.len();
        let top_line = self.top_line;

        let max_scroll = if total_lines > visible_rows {
            total_lines - visible_rows
        } else {
            0
        };

        if let Some(ref mut v_bar) = self.v_scrollbar {
            v_bar.set_params(
                top_line as i32,
                0,
                max_scroll as i32,
                visible_rows as i32,
                1,
            );
        }
    }

    /// Scroll up by one line
    fn scroll_up(&mut self) {
        if self.top_line > 0 {
            self.top_line -= 1;
            self.auto_scroll = false; // Disable auto-scroll when user scrolls
            self.update_scrollbar();
        }
    }

    /// Scroll down by one line
    fn scroll_down(&mut self) {
        let visible_rows = self.get_visible_rows();
        if self.top_line + visible_rows < self.lines.len() {
            self.top_line += 1;
            self.update_scrollbar();

            // Re-enable auto-scroll if at bottom
            if self.top_line + visible_rows >= self.lines.len() {
                self.auto_scroll = true;
            }
        }
    }

    /// Page up
    fn page_up(&mut self) {
        let visible_rows = self.get_visible_rows();
        self.top_line = self.top_line.saturating_sub(visible_rows);
        self.auto_scroll = false; // Disable auto-scroll when user scrolls
        self.update_scrollbar();
    }

    /// Page down
    fn page_down(&mut self) {
        let visible_rows = self.get_visible_rows();
        let max_scroll = if self.lines.len() > visible_rows {
            self.lines.len() - visible_rows
        } else {
            0
        };

        self.top_line = (self.top_line + visible_rows).min(max_scroll);
        self.update_scrollbar();

        // Re-enable auto-scroll if at bottom
        if self.top_line + visible_rows >= self.lines.len() {
            self.auto_scroll = true;
        }
    }
}

impl View for TerminalWidget {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;

        // Update scrollbar bounds
        if self.v_scrollbar.is_some() {
            let v_bounds = Rect::new(
                bounds.b.x - 1,
                bounds.a.y,
                bounds.b.x,
                bounds.b.y,
            );
            self.v_scrollbar = Some(Box::new(ScrollBar::new_vertical(v_bounds)));
        }

        self.update_scrollbar();
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let visible_rows = self.get_visible_rows();
        let visible_width = self.get_visible_width();

        // Use EDITOR_NORMAL for display (editor uses same color regardless of focus)
        let default_color = colors::EDITOR_NORMAL;

        // Draw visible lines
        for i in 0..visible_rows {
            let line_idx = self.top_line + i;
            let mut buf = DrawBuffer::new(visible_width);

            if line_idx < self.lines.len() {
                let line = &self.lines[line_idx];
                let color = line.attr.unwrap_or(default_color);

                // Truncate or pad line to fit width
                let text = if line.text.len() > visible_width {
                    &line.text[..visible_width]
                } else {
                    &line.text
                };

                buf.move_str(0, text, color);

                // Fill rest with spaces
                if text.len() < visible_width {
                    buf.move_char(text.len(), ' ', color, visible_width - text.len());
                }
            } else {
                // Empty line
                buf.move_char(0, ' ', default_color, visible_width);
            }

            write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + i as i16, &buf);
        }

        // Draw scrollbar if present
        if let Some(ref mut v_bar) = self.v_scrollbar {
            v_bar.draw(terminal);
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        match event.what {
            EventType::Keyboard => {
                match event.key_code {
                    KB_UP => {
                        self.scroll_up();
                        event.clear();
                    }
                    KB_DOWN => {
                        self.scroll_down();
                        event.clear();
                    }
                    KB_PGUP => {
                        self.page_up();
                        event.clear();
                    }
                    KB_PGDN => {
                        self.page_down();
                        event.clear();
                    }
                    KB_HOME => {
                        self.scroll_to_top();
                        self.auto_scroll = false;
                        self.update_scrollbar();
                        event.clear();
                    }
                    KB_END => {
                        self.scroll_to_bottom();
                        self.auto_scroll = true;
                        self.update_scrollbar();
                        event.clear();
                    }
                    _ => {}
                }
            }
            EventType::MouseWheelUp => {
                if self.bounds.contains(event.mouse.pos) {
                    self.scroll_up();
                    event.clear();
                }
            }
            EventType::MouseWheelDown => {
                if self.bounds.contains(event.mouse.pos) {
                    self.scroll_down();
                    event.clear();
                }
            }
            _ => {}
        }
    }

    fn state(&self) -> StateFlags {
        self.state
    }

    fn set_state(&mut self, state: StateFlags) {
        self.state = state;
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{palettes, Palette};
        Some(Palette::from_slice(palettes::CP_SCROLLER))
    }

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }

    fn get_owner_type(&self) -> super::view::OwnerType {
        self.owner_type
    }

    fn set_owner_type(&mut self, owner_type: super::view::OwnerType) {
        self.owner_type = owner_type;
    }
}

/// Builder for creating terminal widgets with a fluent API.
pub struct TerminalWidgetBuilder {
    bounds: Option<Rect>,
    with_scrollbar: bool,
    max_lines: usize,
    auto_scroll: bool,
}

impl TerminalWidgetBuilder {
    pub fn new() -> Self {
        Self {
            bounds: None,
            with_scrollbar: false,
            max_lines: 10000,
            auto_scroll: true,
        }
    }

    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    #[must_use]
    pub fn with_scrollbar(mut self, with_scrollbar: bool) -> Self {
        self.with_scrollbar = with_scrollbar;
        self
    }

    #[must_use]
    pub fn max_lines(mut self, max_lines: usize) -> Self {
        self.max_lines = max_lines;
        self
    }

    #[must_use]
    pub fn auto_scroll(mut self, auto_scroll: bool) -> Self {
        self.auto_scroll = auto_scroll;
        self
    }

    pub fn build(self) -> TerminalWidget {
        let bounds = self.bounds.expect("TerminalWidget bounds must be set");
        let mut widget = TerminalWidget::new(bounds);
        if self.with_scrollbar {
            widget = widget.with_scrollbar();
        }
        widget.set_max_lines(self.max_lines);
        widget.set_auto_scroll(self.auto_scroll);
        widget
    }

    pub fn build_boxed(self) -> Box<TerminalWidget> {
        Box::new(self.build())
    }
}

impl Default for TerminalWidgetBuilder {
    fn default() -> Self {
        Self::new()
    }
}
