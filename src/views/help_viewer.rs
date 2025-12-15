// (C) 2025 - Enzo Lombardi

//! HelpViewer view - scrollable help content viewer with cross-reference navigation.
// HelpViewer - Help content viewer based on TextView
//
// Matches Borland: THelpViewer (help.h)
//
// Displays help topic content with scrolling support and clickable links.

use super::help_file::{CrossRef, HelpTopic, TextSegment};
use super::scrollbar::ScrollBar;
use super::view::{write_line_to_terminal, View};
use crate::core::draw::DrawBuffer;
use crate::core::event::{
    Event, EventType, KB_DOWN, KB_END, KB_ENTER, KB_HOME, KB_LEFT, KB_PGDN, KB_PGUP, KB_RIGHT,
    KB_SHIFT_TAB, KB_TAB, KB_UP, MB_LEFT_BUTTON,
};
use crate::core::geometry::{Point, Rect};
use crate::core::state::StateFlags;
use crate::terminal::Terminal;

/// HelpViewer - Displays help topic content with cross-reference navigation
///
/// Matches Borland: THelpViewer (help.cc)
/// Features:
/// - Scrollable content display
/// - Cross-reference links highlighted in different color
/// - TAB/Shift+TAB cycles through links
/// - ENTER follows the selected link
pub struct HelpViewer {
    bounds: Rect,
    state: StateFlags,
    delta: Point, // Current scroll offset
    limit: Point, // Maximum scroll values
    vscrollbar: Option<Box<ScrollBar>>,
    styled_lines: Vec<Vec<TextSegment>>,  // Lines with styled segments
    cross_refs: Vec<CrossRef>,            // Cross-references with position info
    selected: usize,                      // Currently selected cross-ref (1-based like Borland)
    current_topic: Option<String>,
    owner: Option<*const dyn View>,
    owner_type: super::view::OwnerType,
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
            styled_lines: Vec::new(),
            cross_refs: Vec::new(),
            selected: 1, // 1-based, like Borland
            current_topic: None,
            owner: None,
            owner_type: super::view::OwnerType::None,
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
        // Get content with styled segments and cross-references
        let (styled_lines, refs) = topic.get_styled_content();
        self.styled_lines = styled_lines;
        self.cross_refs = refs;
        self.selected = 1; // Reset to first link
        self.current_topic = Some(topic.id.clone());

        // Calculate maximum line width for horizontal scrolling
        let max_line_width = self.styled_lines.iter()
            .map(|segments| segments.iter().map(|s| s.len()).sum::<usize>())
            .max()
            .unwrap_or(0) as i16;

        // Update limits (vertical and horizontal)
        let display_width = if self.vscrollbar.is_some() {
            self.bounds.width() - 1
        } else {
            self.bounds.width()
        };
        let max_x = (max_line_width - display_width).max(0);
        let max_y = if self.styled_lines.len() > self.bounds.height_clamped() as usize {
            self.styled_lines.len() as i16 - self.bounds.height()
        } else {
            0
        };
        self.limit = Point::new(max_x, max_y);
        self.delta = Point::new(0, 0);

        self.update_scrollbar();
    }

    /// Get the number of cross-references in current topic
    pub fn num_cross_refs(&self) -> usize {
        self.cross_refs.len()
    }

    /// Get the currently selected cross-reference (if any)
    pub fn get_selected_cross_ref(&self) -> Option<&CrossRef> {
        if self.selected > 0 && self.selected <= self.cross_refs.len() {
            Some(&self.cross_refs[self.selected - 1])
        } else {
            None
        }
    }

    /// Get the target topic ID of the currently selected link
    pub fn get_selected_target(&self) -> Option<&str> {
        self.get_selected_cross_ref().map(|r| r.target.as_str())
    }

    /// Make the selected cross-reference visible by scrolling if needed
    /// Matches Borland: THelpViewer::makeSelectVisible()
    fn make_select_visible(&mut self) {
        if let Some(cross_ref) = self.get_selected_cross_ref() {
            let key_point_y = cross_ref.line;
            let mut d = self.delta;

            // Scroll to make the link visible
            if key_point_y <= d.y {
                d.y = key_point_y - 1;
            }
            if key_point_y > d.y + self.bounds.height() {
                d.y = key_point_y - self.bounds.height();
            }

            if d.y != self.delta.y {
                self.delta.y = d.y.max(0).min(self.limit.y);
                self.update_scrollbar();
            }
        }
    }

    /// Select next cross-reference (TAB key)
    fn select_next(&mut self) {
        if self.cross_refs.is_empty() {
            return;
        }
        self.selected += 1;
        if self.selected > self.cross_refs.len() {
            self.selected = 1; // Wrap around
        }
        self.make_select_visible();
    }

    /// Select previous cross-reference (Shift+TAB key)
    fn select_prev(&mut self) {
        if self.cross_refs.is_empty() {
            return;
        }
        if self.selected <= 1 {
            self.selected = self.cross_refs.len(); // Wrap around
        } else {
            self.selected -= 1;
        }
        self.make_select_visible();
    }

    /// Find cross-reference at the given screen position
    /// Returns 1-based index (like Borland) or 0 if none found
    /// Matches Borland: THelpViewer::getNumRows() pattern for hit testing
    fn get_cross_ref_at(&self, screen_x: i16, screen_y: i16) -> usize {
        // Convert screen coordinates to view-relative coordinates
        let rel_x = screen_x - self.bounds.a.x;
        let rel_y = screen_y - self.bounds.a.y;

        // Check bounds
        if rel_x < 0 || rel_y < 0 || rel_x >= self.bounds.width() || rel_y >= self.bounds.height() {
            return 0;
        }

        // Calculate the line number (1-based, accounting for scroll)
        let line_num = (self.delta.y + rel_y + 1) as i16;

        // Search cross-refs for one that matches this position
        for (i, cross_ref) in self.cross_refs.iter().enumerate() {
            if cross_ref.line == line_num {
                let start = cross_ref.offset;
                let end = start + cross_ref.length as i16;
                if rel_x >= start && rel_x < end {
                    return i + 1; // Return 1-based index
                }
            }
        }

        0 // No cross-ref at this position
    }

    /// Get the current topic ID
    pub fn current_topic(&self) -> Option<&str> {
        self.current_topic.as_deref()
    }

    /// Clear the viewer
    pub fn clear(&mut self) {
        self.styled_lines.clear();
        self.cross_refs.clear();
        self.selected = 1;
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
            let sb_bounds = Rect::new(bounds.b.x - 1, bounds.a.y, bounds.b.x, bounds.b.y);
            if let Some(ref mut sb) = self.vscrollbar {
                sb.set_bounds(sb_bounds);
            }
        }

        // Calculate maximum line width for horizontal scrolling
        let max_line_width = self.styled_lines.iter()
            .map(|segments| segments.iter().map(|s| s.len()).sum::<usize>())
            .max()
            .unwrap_or(0) as i16;

        // Recalculate limits (vertical and horizontal)
        let display_width = if self.vscrollbar.is_some() {
            self.bounds.width() - 1
        } else {
            self.bounds.width()
        };
        let max_x = (max_line_width - display_width).max(0);
        let max_y = if self.styled_lines.len() > self.bounds.height_clamped() as usize {
            self.styled_lines.len() as i16 - self.bounds.height()
        } else {
            0
        };
        self.limit = Point::new(max_x, max_y);
        self.update_scrollbar();
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let start_line = self.delta.y as usize;
        let h_offset = self.delta.x as usize;  // Horizontal scroll offset

        // Determine display width (leave room for scrollbar if present)
        let display_width = if self.vscrollbar.is_some() {
            (self.bounds.width() - 1) as usize
        } else {
            self.bounds.width_clamped() as usize
        };

        // Get colors from palette for rich text rendering
        // Matches Borland: THelpViewer::draw() (help.cc:54-70)
        // Extended for bold, italic, code styling
        let normal = self.map_color(1);      // Normal text
        let keyword = self.map_color(2);     // Link text
        let sel_keyword = self.map_color(3); // Selected link
        let bold_color = self.map_color(4);  // Bold text
        let italic_color = self.map_color(5); // Italic text
        let code_color = self.map_color(6);  // Code text

        for row in 0..self.bounds.height() {
            let line_num = (start_line + row as usize + 1) as i16; // 1-based line number
            let line_idx = start_line + row as usize;

            let mut buf = DrawBuffer::new(display_width);
            buf.move_char(0, ' ', normal, display_width);

            if line_idx < self.styled_lines.len() {
                let segments = &self.styled_lines[line_idx];
                let mut abs_col = 0usize;  // Absolute column in the line

                for segment in segments {
                    let text = segment.text();
                    let seg_start = abs_col;
                    let seg_end = abs_col + text.len();

                    // Check if this segment is visible (at least partially)
                    if seg_end > h_offset && seg_start < h_offset + display_width {
                        // Determine color based on segment type
                        let color = match segment {
                            TextSegment::Normal(_) => normal,
                            TextSegment::Bold(_) => bold_color,
                            TextSegment::Italic(_) => italic_color,
                            TextSegment::Code(_) => code_color,
                            TextSegment::Link { .. } => {
                                // Find matching cross-ref to check if selected
                                let is_selected = self.cross_refs.iter().enumerate().any(|(i, r)| {
                                    r.line == line_num && r.offset == abs_col as i16 && i + 1 == self.selected
                                });
                                if is_selected {
                                    sel_keyword
                                } else {
                                    keyword
                                }
                            }
                        };

                        // Calculate visible portion of the segment
                        let visible_start = if seg_start >= h_offset { seg_start - h_offset } else { 0 };
                        let text_start = if seg_start >= h_offset { 0 } else { h_offset - seg_start };
                        let text_end = (seg_end - h_offset).min(display_width);
                        let visible_len = if text_end > visible_start { text_end - visible_start } else { 0 };

                        if visible_len > 0 && text_start < text.len() {
                            let text_slice_end = (text_start + visible_len).min(text.len());
                            buf.move_str(visible_start, &text[text_start..text_slice_end], color);
                        }
                    }

                    abs_col = seg_end;

                    // Early exit if we've gone past the visible area
                    if abs_col >= h_offset + display_width {
                        break;
                    }
                }
            }

            write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + row, &buf);
        }

        // Draw scrollbar if present
        if let Some(ref mut sb) = self.vscrollbar {
            sb.draw(terminal);
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        let page_size = self.bounds.height();

        match event.what {
            EventType::Keyboard => {
                match event.key_code {
                    KB_UP => {
                        self.scroll_by(0, -1);
                        event.clear();
                    }
                    KB_DOWN => {
                        self.scroll_by(0, 1);
                        event.clear();
                    }
                    KB_LEFT => {
                        // Horizontal scroll left
                        // Matches Borland: THelpViewer::handleEvent() horizontal scrolling
                        self.scroll_by(-1, 0);
                        event.clear();
                    }
                    KB_RIGHT => {
                        // Horizontal scroll right
                        self.scroll_by(1, 0);
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
                    KB_TAB => {
                        // Select next cross-reference
                        // Matches Borland: THelpViewer::handleEvent() kbTab case (help.cc:176-181)
                        self.select_next();
                        event.clear();
                    }
                    KB_SHIFT_TAB => {
                        // Select previous cross-reference
                        // Matches Borland: THelpViewer::handleEvent() kbShiftTab case (help.cc:182-188)
                        self.select_prev();
                        event.clear();
                    }
                    KB_ENTER => {
                        // Follow selected link - convert to command for HelpWindow to handle
                        // Matches Borland: THelpViewer::handleEvent() kbEnter case (help.cc:189-194)
                        if self.selected > 0 && self.selected <= self.cross_refs.len() {
                            // Don't clear the event - let HelpWindow intercept and navigate
                            // HelpWindow will call get_selected_target() to get the destination
                        }
                    }
                    _ => {}
                }
            }
            EventType::MouseDown => {
                // Handle mouse clicks on cross-references
                // Matches Borland: THelpViewer::handleEvent() evMouseDown case (help.cc:122-155)
                let mouse_pos = event.mouse.pos;

                if self.bounds.contains(mouse_pos) && event.mouse.buttons & MB_LEFT_BUTTON != 0 {
                    // Check if click is on a cross-reference link
                    let hit_ref = self.get_cross_ref_at(mouse_pos.x, mouse_pos.y);

                    if hit_ref > 0 {
                        // Select the clicked link
                        self.selected = hit_ref;

                        if event.mouse.double_click {
                            // Double-click follows the link
                            // Don't clear event - let HelpWindow handle navigation
                            // Convert to ENTER-like behavior
                        } else {
                            // Single click just selects
                            event.clear();
                        }
                    } else {
                        event.clear();
                    }
                }
            }
            EventType::MouseWheelUp => {
                // Scroll up on mouse wheel
                if self.bounds.contains(event.mouse.pos) {
                    self.scroll_by(0, -3);
                    event.clear();
                }
            }
            EventType::MouseWheelDown => {
                // Scroll down on mouse wheel
                if self.bounds.contains(event.mouse.pos) {
                    self.scroll_by(0, 3);
                    event.clear();
                }
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
        use crate::core::palette::{palettes, Palette};
        Some(Palette::from_slice(palettes::CP_HELP_VIEWER))
    }

    fn get_owner_type(&self) -> super::view::OwnerType {
        self.owner_type
    }

    fn set_owner_type(&mut self, owner_type: super::view::OwnerType) {
        self.owner_type = owner_type;
    }
}

/// Builder for creating help viewers with a fluent API.
pub struct HelpViewerBuilder {
    bounds: Option<Rect>,
    with_scrollbar: bool,
}

impl HelpViewerBuilder {
    pub fn new() -> Self {
        Self { bounds: None, with_scrollbar: false }
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

    pub fn build(self) -> HelpViewer {
        let bounds = self.bounds.expect("HelpViewer bounds must be set");
        let viewer = HelpViewer::new(bounds);
        if self.with_scrollbar {
            viewer.with_scrollbar()
        } else {
            viewer
        }
    }

    pub fn build_boxed(self) -> Box<HelpViewer> {
        Box::new(self.build())
    }
}

impl Default for HelpViewerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(!viewer.styled_lines.is_empty());
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
        assert!(viewer.styled_lines.is_empty());
    }
}
