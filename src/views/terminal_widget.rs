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
//! - Read-only (unlike EditorWindow)

use super::scrollbar::ScrollBar;
use super::view::{View, ViewCore, write_line_to_terminal};
use crate::core::draw::DrawBuffer;
use crate::core::event::{Event, EventType, KB_DOWN, KB_END, KB_HOME, KB_PGDN, KB_PGUP, KB_UP};
use crate::core::geometry::Rect;
use crate::core::palette::Attr;
use crate::core::state::State;
use crate::terminal::Terminal;

/// A run of text within a line, with its own optional colour.
///
/// Spans are how a single line carries more than one attribute — an
/// identifier in one colour beside a keyword in another, a dim prefix in
/// front of bright content. A span with no attribute takes the line's.
#[derive(Clone, Debug)]
pub struct Span {
    /// The text of this run.
    pub text: String,
    /// Its colour; `None` takes the line's own attribute.
    pub attr: Option<Attr>,
}

impl Span {
    /// A run that takes the line's colour.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            attr: None,
        }
    }

    /// A run in its own colour.
    pub fn with_attr(text: impl Into<String>, attr: Attr) -> Self {
        Self {
            text: text.into(),
            attr: Some(attr),
        }
    }
}

/// A line of output with optional color attributes
#[derive(Clone, Debug)]
pub struct OutputLine {
    /// The text content. When the line is built from spans this is their
    /// text joined, so measuring and searching a line needs no knowledge of
    /// how it is coloured.
    pub text: String,
    /// Optional color attribute (if None, uses default)
    pub attr: Option<Attr>,
    /// The coloured runs the line is drawn from. Empty means the whole line
    /// is drawn from `text` in `attr`, which is what the single-attribute
    /// constructors produce.
    pub spans: Vec<Span>,
}

impl OutputLine {
    /// Create a new output line with default color
    pub fn new(text: String) -> Self {
        Self {
            text,
            attr: None,
            spans: Vec::new(),
        }
    }

    /// Create a new output line with specific color
    pub fn with_attr(text: String, attr: Attr) -> Self {
        Self {
            text,
            attr: Some(attr),
            spans: Vec::new(),
        }
    }

    /// A line made of coloured runs, drawn left to right.
    pub fn with_spans(spans: Vec<Span>) -> Self {
        let text = spans.iter().map(|s| s.text.as_str()).collect();
        Self {
            text,
            attr: None,
            spans,
        }
    }
}

/// The longest prefix of `text` that fits `width` columns, and the columns
/// it takes.
///
/// Measured in display columns: a wide glyph (CJK, emoji) counts as two and
/// is dropped rather than half-drawn, and a zero-width mark rides along with
/// the character it follows, matching how `DrawBuffer::move_str` lays them
/// out.
fn truncate_to_width(text: &str, width: usize) -> (String, usize) {
    use unicode_width::UnicodeWidthChar;
    let mut out = String::with_capacity(text.len());
    let mut used = 0;
    for ch in text.chars() {
        let w = ch.width().unwrap_or(0);
        if used + w > width {
            break;
        }
        out.push(ch);
        used += w;
    }
    (out, used)
}

/// Terminal Widget - scrolling output viewer
/// Matches Borland: TTerminal
pub struct TerminalWidget {
    core: ViewCore,
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
}

impl TerminalWidget {
    /// Create a new terminal widget
    pub fn new(bounds: Rect) -> Self {
        Self {
            core: ViewCore {
                bounds,
                state: State::empty(),
                palette_chain: None,
                ..ViewCore::default()
            },
            lines: Vec::new(),
            max_lines: 10000, // Default: 10k lines scrollback
            top_line: 0,
            auto_scroll: true,
            v_scrollbar: None,
        }
    }

    /// Create with vertical scrollbar
    pub fn with_scrollbar(mut self) -> Self {
        let v_bounds = Rect::new(
            self.extent().b.x - 1,
            0,
            self.extent().b.x,
            self.extent().b.y,
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

    /// Append a line made of coloured runs.
    pub fn append_line_spans(&mut self, spans: Vec<Span>) {
        self.lines.push(OutputLine::with_spans(spans));
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
        let mut height = self.core.bounds.height_clamped() as usize;
        if self.v_scrollbar.is_some() {
            // Account for scrollbar taking up space
            height = height.saturating_sub(0); // scrollbar doesn't reduce height
        }
        height
    }

    /// Get the visible width
    fn get_visible_width(&self) -> usize {
        let mut width = self.core.bounds.width_clamped() as usize;
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
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.core.bounds = bounds;
        // Children are laid out in this view's own space
        let bounds = self.extent();

        // Update scrollbar bounds
        if self.v_scrollbar.is_some() {
            let v_bounds = Rect::new(bounds.b.x - 1, bounds.a.y, bounds.b.x, bounds.b.y);
            self.v_scrollbar = Some(Box::new(ScrollBar::new_vertical(v_bounds)));
        }

        self.update_scrollbar();
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let visible_rows = self.get_visible_rows();
        let visible_width = self.get_visible_width();

        // Terminal look: light gray text on black background
        let default_color = Attr::new(
            crate::core::palette::TvColor::LightGray,
            crate::core::palette::TvColor::Black,
        );

        // Draw visible lines
        for i in 0..visible_rows {
            let line_idx = self.top_line + i;
            let mut buf = DrawBuffer::new(visible_width);

            if line_idx < self.lines.len() {
                let line = &self.lines[line_idx];
                let color = line.attr.unwrap_or(default_color);

                // Paint the whole row first, so a line shorter than the view
                // is padded in its own colour rather than left as it was.
                buf.move_char(0, ' ', color, visible_width);

                if line.spans.is_empty() {
                    let (text, _) = truncate_to_width(&line.text, visible_width);
                    buf.move_str(0, &text, color);
                } else {
                    // Each run starts where the previous one ended, measured
                    // in columns rather than characters so a wide glyph does
                    // not push the rest of the line out of step.
                    let mut col = 0;
                    for span in &line.spans {
                        if col >= visible_width {
                            break;
                        }
                        let (text, width) = truncate_to_width(&span.text, visible_width - col);
                        buf.move_str(col, &text, span.attr.unwrap_or(color));
                        col += width;
                    }
                }
            } else {
                // Empty line
                buf.move_char(0, ' ', default_color, visible_width);
            }

            write_line_to_terminal(terminal, 0, i as i16, &buf);
        }

        // Draw scrollbar if present
        if let Some(ref mut v_bar) = self.v_scrollbar {
            crate::views::view::draw_child(terminal, &mut **v_bar);
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        match event.what {
            EventType::Keyboard => match event.key_code {
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
            },
            EventType::MouseWheelUp => {
                if self.extent().contains(event.mouse.pos) {
                    self.scroll_up();
                    event.clear();
                }
            }
            EventType::MouseWheelDown => {
                if self.extent().contains(event.mouse.pos) {
                    self.scroll_down();
                    event.clear();
                }
            }
            _ => {}
        }
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
        use crate::core::palette::{Palette, palettes};
        Some(Palette::from_slice(palettes::CP_SCROLLER))
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

#[cfg(test)]
mod tests {
    use super::{Span, TerminalWidget, truncate_to_width};
    use crate::core::geometry::Rect;
    use crate::core::palette::{Attr, TvColor};
    use crate::test_util::test_terminal;
    use crate::views::View;

    const RED: Attr = Attr::new(TvColor::LightRed, TvColor::Black);
    const GREEN: Attr = Attr::new(TvColor::LightGreen, TvColor::Black);

    /// The characters and attributes of one drawn row.
    fn row(widget: &mut TerminalWidget, y: i16, width: u16) -> (String, Vec<Attr>) {
        let mut terminal = test_terminal(width, 4);
        let width = i16::try_from(width).expect("test widths are small");
        widget.draw(&mut terminal);
        let cells: Vec<_> = (0..width)
            .map(|x| terminal.read_cell(x, y).expect("cell in bounds"))
            .collect();
        (
            cells.iter().map(|c| c.ch).collect(),
            cells.iter().map(|c| c.attr).collect(),
        )
    }

    #[test]
    fn a_single_attribute_line_still_draws_in_one_colour() {
        let mut widget = TerminalWidget::new(Rect::new(0, 0, 8, 2));
        widget.append_line_colored("hi".to_string(), RED);
        let (text, attrs) = row(&mut widget, 0, 8);
        assert_eq!(text, "hi      ");
        // The padding takes the line's colour too.
        assert!(attrs.iter().all(|a| *a == RED), "{attrs:?}");
    }

    #[test]
    fn spans_each_keep_their_own_colour() {
        let mut widget = TerminalWidget::new(Rect::new(0, 0, 8, 2));
        widget.append_line_spans(vec![
            Span::with_attr("ab", RED),
            Span::with_attr("cd", GREEN),
        ]);
        let (text, attrs) = row(&mut widget, 0, 8);
        assert_eq!(text, "abcd    ");
        assert_eq!(attrs[0], RED);
        assert_eq!(attrs[1], RED);
        assert_eq!(attrs[2], GREEN);
        assert_eq!(attrs[3], GREEN);
    }

    #[test]
    fn a_span_without_a_colour_takes_the_lines_own() {
        let mut widget = TerminalWidget::new(Rect::new(0, 0, 6, 2));
        let mut line =
            super::OutputLine::with_spans(vec![Span::new("ab"), Span::with_attr("cd", GREEN)]);
        line.attr = Some(RED);
        widget.lines.push(line);
        let (_, attrs) = row(&mut widget, 0, 6);
        assert_eq!(attrs[0], RED);
        assert_eq!(attrs[2], GREEN);
    }

    #[test]
    fn spans_are_clipped_at_the_right_edge() {
        let mut widget = TerminalWidget::new(Rect::new(0, 0, 3, 2));
        widget.append_line_spans(vec![
            Span::with_attr("ab", RED),
            Span::with_attr("cdef", GREEN),
        ]);
        let (text, attrs) = row(&mut widget, 0, 3);
        assert_eq!(text, "abc");
        assert_eq!(attrs[2], GREEN);
    }

    #[test]
    fn a_spanned_lines_text_is_its_runs_joined() {
        let line = super::OutputLine::with_spans(vec![Span::new("ab"), Span::new("cd")]);
        assert_eq!(line.text, "abcd");
    }

    #[test]
    fn a_wide_glyph_keeps_later_spans_in_step() {
        let mut widget = TerminalWidget::new(Rect::new(0, 0, 6, 2));
        widget.append_line_spans(vec![
            Span::with_attr("漢", RED),
            Span::with_attr("ab", GREEN),
        ]);
        let (_, attrs) = row(&mut widget, 0, 6);
        // The glyph takes two columns, so the second run starts at column 2.
        assert_eq!(attrs[0], RED);
        assert_eq!(attrs[2], GREEN);
        assert_eq!(attrs[3], GREEN);
    }

    #[test]
    fn truncation_measures_columns_not_characters() {
        assert_eq!(truncate_to_width("abc", 2), ("ab".to_string(), 2));
        // A wide glyph that does not fit is dropped whole.
        assert_eq!(truncate_to_width("漢", 1), (String::new(), 0));
        assert_eq!(truncate_to_width("漢", 2), ("漢".to_string(), 2));
    }
}
