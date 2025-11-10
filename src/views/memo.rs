// (C) 2025 - Enzo Lombardi

//! Memo view - multi-line text input with scrolling and editing support.

use crate::core::geometry::{Point, Rect};
use crate::core::event::{Event, EventType, KB_UP, KB_DOWN, KB_LEFT, KB_RIGHT, KB_PGUP, KB_PGDN, KB_HOME, KB_END, KB_ENTER, KB_BACKSPACE, KB_DEL, KB_TAB};
use crate::core::draw::DrawBuffer;
use crate::core::clipboard;
use crate::core::state::StateFlags;
use crate::terminal::Terminal;
use super::view::{View, write_line_to_terminal};
use super::scrollbar::ScrollBar;
use std::cmp::min;

// Control key codes
const KB_CTRL_A: u16 = 0x0001;  // Ctrl+A - Select All
const KB_CTRL_C: u16 = 0x0003;  // Ctrl+C - Copy
const KB_CTRL_V: u16 = 0x0016;  // Ctrl+V - Paste
const KB_CTRL_X: u16 = 0x0018;  // Ctrl+X - Cut
#[expect(dead_code, reason = "Reserved for future undo functionality in Memo widget")]
const KB_CTRL_Z: u16 = 0x001A;  // Ctrl+Z - Undo

/// Memo - Multi-line text editor control
/// Supports basic text editing operations including insert, delete, navigation, and selection
pub struct Memo {
    bounds: Rect,
    lines: Vec<String>,
    cursor: Point,           // Current cursor position (x=col, y=line)
    delta: Point,            // Scroll offset
    selection_start: Option<Point>, // Selection anchor point
    state: StateFlags,
    v_scrollbar: Option<Box<ScrollBar>>,
    h_scrollbar: Option<Box<ScrollBar>>,
    max_length: Option<usize>, // Maximum length per line (None = unlimited)
    read_only: bool,
    modified: bool,
    tab_size: usize,
    owner: Option<*const dyn View>,
}

impl Memo {
    /// Create a new memo control
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            lines: vec![String::new()],
            cursor: Point::zero(),
            delta: Point::zero(),
            selection_start: None,
            state: 0,
            v_scrollbar: None,
            h_scrollbar: None,
            max_length: None,
            read_only: false,
            modified: false,
            tab_size: 4,
            owner: None,
        }
    }

    /// Create with scrollbars
    pub fn with_scrollbars(mut self, add_scrollbars: bool) -> Self {
        if add_scrollbars {
            // Vertical scrollbar on right edge
            let v_bounds = Rect::new(
                self.bounds.b.x - 1,
                self.bounds.a.y,
                self.bounds.b.x,
                self.bounds.b.y - 1,
            );
            self.v_scrollbar = Some(Box::new(ScrollBar::new_vertical(v_bounds)));

            // Horizontal scrollbar on bottom edge
            let h_bounds = Rect::new(
                self.bounds.a.x,
                self.bounds.b.y - 1,
                self.bounds.b.x - 1,
                self.bounds.b.y,
            );
            self.h_scrollbar = Some(Box::new(ScrollBar::new_horizontal(h_bounds)));
        }
        self
    }

    /// Set read-only mode
    pub fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
    }

    /// Set maximum line length
    pub fn set_max_length(&mut self, max_length: Option<usize>) {
        self.max_length = max_length;
    }

    /// Set tab size
    pub fn set_tab_size(&mut self, tab_size: usize) {
        self.tab_size = tab_size.max(1);
    }

    /// Get the text content
    pub fn get_text(&self) -> String {
        self.lines.join("\n")
    }

    /// Set the text content
    pub fn set_text(&mut self, text: &str) {
        self.lines = text.lines().map(|s| s.to_string()).collect();
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.cursor = Point::zero();
        self.delta = Point::zero();
        self.selection_start = None;
        self.modified = false;
        self.update_scrollbars();
    }

    /// Check if text has been modified
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Clear the modified flag
    pub fn clear_modified(&mut self) {
        self.modified = false;
    }

    /// Get current line count
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Get the visible content area
    fn get_content_area(&self) -> Rect {
        let mut area = self.bounds;
        if self.v_scrollbar.is_some() {
            area.b.x -= 1;
        }
        if self.h_scrollbar.is_some() {
            area.b.y -= 1;
        }
        area
    }

    /// Get maximum line length
    fn max_line_length(&self) -> i16 {
        self.lines
            .iter()
            .map(|line| line.len() as i16)
            .max()
            .unwrap_or(0)
    }

    /// Update scrollbars based on content and cursor
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

    /// Ensure cursor is visible by adjusting scroll offset
    fn ensure_cursor_visible(&mut self) {
        let content_area = self.get_content_area();
        let width = content_area.width();
        let height = content_area.height();

        // Vertical scrolling
        if self.cursor.y < self.delta.y {
            self.delta.y = self.cursor.y;
        } else if self.cursor.y >= self.delta.y + height {
            self.delta.y = self.cursor.y - height + 1;
        }

        // Horizontal scrolling
        if self.cursor.x < self.delta.x {
            self.delta.x = self.cursor.x;
        } else if self.cursor.x >= self.delta.x + width {
            self.delta.x = self.cursor.x - width + 1;
        }

        self.update_scrollbars();
    }

    /// Clamp cursor to valid position
    fn clamp_cursor(&mut self) {
        if self.cursor.y < 0 {
            self.cursor.y = 0;
        }
        if self.cursor.y >= self.lines.len() as i16 {
            self.cursor.y = (self.lines.len() - 1) as i16;
        }

        let line_len = self.lines[self.cursor.y as usize].chars().count() as i16;
        if self.cursor.x > line_len {
            self.cursor.x = line_len;
        }
        if self.cursor.x < 0 {
            self.cursor.x = 0;
        }
    }

    /// Convert character position to byte index for a line
    fn char_to_byte_idx(&self, line_idx: usize, char_pos: usize) -> usize {
        let line = &self.lines[line_idx];
        line.char_indices()
            .nth(char_pos)
            .map(|(idx, _)| idx)
            .unwrap_or(line.len())
    }

    /// Insert a character at cursor position
    fn insert_char(&mut self, ch: char) {
        if self.read_only {
            return;
        }

        let line_idx = self.cursor.y as usize;
        let col = self.cursor.x as usize;

        // Check max length (in characters)
        if let Some(max_len) = self.max_length {
            if self.lines[line_idx].chars().count() >= max_len {
                return;
            }
        }

        let byte_idx = self.char_to_byte_idx(line_idx, col);
        self.lines[line_idx].insert(byte_idx, ch);
        self.cursor.x += 1;
        self.modified = true;
        self.selection_start = None;
        self.ensure_cursor_visible();
    }

    /// Insert a newline at cursor position
    fn insert_newline(&mut self) {
        if self.read_only {
            return;
        }

        let line_idx = self.cursor.y as usize;
        let col = self.cursor.x as usize;

        let current_line = &self.lines[line_idx];
        let byte_idx = self.char_to_byte_idx(line_idx, col);
        let before = current_line[..byte_idx].to_string();
        let after = current_line[byte_idx..].to_string();

        self.lines[line_idx] = before;
        self.lines.insert(line_idx + 1, after);

        self.cursor.y += 1;
        self.cursor.x = 0;
        self.modified = true;
        self.selection_start = None;
        self.ensure_cursor_visible();
    }

    /// Delete character at cursor (Delete key)
    fn delete_char(&mut self) {
        if self.read_only {
            return;
        }

        let line_idx = self.cursor.y as usize;
        let col = self.cursor.x as usize;
        let line_char_count = self.lines[line_idx].chars().count();

        if col < line_char_count {
            // Delete character in current line
            let byte_idx = self.char_to_byte_idx(line_idx, col);
            let ch = self.lines[line_idx][byte_idx..].chars().next().unwrap();
            let ch_len = ch.len_utf8();
            self.lines[line_idx].drain(byte_idx..byte_idx + ch_len);
            self.modified = true;
        } else if line_idx + 1 < self.lines.len() {
            // Join with next line
            let next_line = self.lines.remove(line_idx + 1);
            self.lines[line_idx].push_str(&next_line);
            self.modified = true;
        }

        self.selection_start = None;
        self.ensure_cursor_visible();
    }

    /// Delete character before cursor (Backspace)
    fn backspace(&mut self) {
        if self.read_only {
            return;
        }

        let line_idx = self.cursor.y as usize;
        let col = self.cursor.x as usize;

        if col > 0 {
            // Delete character in current line
            let byte_idx = self.char_to_byte_idx(line_idx, col - 1);
            let ch = self.lines[line_idx][byte_idx..].chars().next().unwrap();
            let ch_len = ch.len_utf8();
            self.lines[line_idx].drain(byte_idx..byte_idx + ch_len);
            self.cursor.x -= 1;
            self.modified = true;
        } else if line_idx > 0 {
            // Join with previous line
            let current_line = self.lines.remove(line_idx);
            self.cursor.y -= 1;
            let prev_line_len = self.lines[line_idx - 1].chars().count();
            self.lines[line_idx - 1].push_str(&current_line);
            self.cursor.x = prev_line_len as i16;
            self.modified = true;
        }

        self.selection_start = None;
        self.ensure_cursor_visible();
    }

    /// Insert tab (as spaces)
    fn insert_tab(&mut self) {
        if self.read_only {
            return;
        }

        for _ in 0..self.tab_size {
            self.insert_char(' ');
        }
    }

    /// Move cursor
    fn move_cursor(&mut self, dx: i16, dy: i16, extend_selection: bool) {
        if !extend_selection {
            self.selection_start = None;
        } else if self.selection_start.is_none() {
            self.selection_start = Some(self.cursor);
        }

        self.cursor.x += dx;
        self.cursor.y += dy;
        self.clamp_cursor();
        self.ensure_cursor_visible();
    }

    /// Check if there's an active selection
    pub fn has_selection(&self) -> bool {
        self.selection_start.is_some()
    }

    /// Get selected text
    pub fn get_selection(&self) -> Option<String> {
        let start = self.selection_start?;
        let end = self.cursor;

        let (start, end) = if start.y < end.y || (start.y == end.y && start.x < end.x) {
            (start, end)
        } else {
            (end, start)
        };

        if start == end {
            return None;
        }

        let mut result = String::new();
        for y in start.y..=end.y {
            if y < 0 || y >= self.lines.len() as i16 {
                continue;
            }

            let line = &self.lines[y as usize];
            if y == start.y && y == end.y {
                // Single line selection
                let s = start.x.max(0) as usize;
                let e = end.x.min(line.len() as i16) as usize;
                if s < e {
                    result.push_str(&line[s..e]);
                }
            } else if y == start.y {
                // First line
                let s = start.x.max(0) as usize;
                result.push_str(&line[s..]);
                result.push('\n');
            } else if y == end.y {
                // Last line
                let e = end.x.min(line.len() as i16) as usize;
                result.push_str(&line[..e]);
            } else {
                // Middle lines
                result.push_str(line);
                result.push('\n');
            }
        }

        Some(result)
    }

    /// Select all text
    pub fn select_all(&mut self) {
        self.selection_start = Some(Point::zero());
        self.cursor = Point::new(
            self.lines.last().map(|l| l.chars().count()).unwrap_or(0) as i16,
            (self.lines.len() - 1) as i16,
        );
        self.ensure_cursor_visible();
    }

    /// Delete the current selection
    fn delete_selection(&mut self) {
        if !self.has_selection() || self.read_only {
            return;
        }

        let start = self.selection_start.unwrap();
        let end = self.cursor;

        let (start, end) = if start.y < end.y || (start.y == end.y && start.x < end.x) {
            (start, end)
        } else {
            (end, start)
        };

        let start_line = start.y.max(0) as usize;
        let end_line = end.y.min((self.lines.len() - 1) as i16) as usize;

        if start_line == end_line {
            // Single line deletion
            let start_col = start.x.max(0) as usize;
            let end_col = end.x.min(self.lines[start_line].chars().count() as i16) as usize;
            if start_col < end_col {
                let start_byte = self.char_to_byte_idx(start_line, start_col);
                let end_byte = self.char_to_byte_idx(start_line, end_col);
                self.lines[start_line].drain(start_byte..end_byte);
            }
        } else {
            // Multi-line deletion
            let start_col = start.x.max(0) as usize;
            let end_col = end.x.min(self.lines[end_line].chars().count() as i16) as usize;

            // Keep part before selection on first line and part after selection on last line
            let start_byte = self.char_to_byte_idx(start_line, start_col);
            let end_byte = self.char_to_byte_idx(end_line, end_col);

            let before = self.lines[start_line][..start_byte].to_string();
            let after = self.lines[end_line][end_byte..].to_string();

            // Remove lines in between
            self.lines.drain(start_line..=end_line);

            // Insert the combined line
            self.lines.insert(start_line, before + &after);
        }

        self.cursor = start;
        self.selection_start = None;
        self.modified = true;
        self.ensure_cursor_visible();
    }

    /// Insert text at cursor position
    fn insert_text(&mut self, text: &str) {
        if self.read_only {
            return;
        }

        // Delete selection if any
        if self.has_selection() {
            self.delete_selection();
        }

        let lines_to_insert: Vec<&str> = text.lines().collect();
        if lines_to_insert.is_empty() {
            return;
        }

        let line_idx = self.cursor.y as usize;
        let col = self.cursor.x as usize;

        if lines_to_insert.len() == 1 {
            // Single line insertion
            let byte_idx = self.char_to_byte_idx(line_idx, col);
            self.lines[line_idx].insert_str(byte_idx, lines_to_insert[0]);
            self.cursor.x += lines_to_insert[0].chars().count() as i16;
        } else {
            // Multi-line insertion
            let current_line = &self.lines[line_idx];
            let byte_idx = self.char_to_byte_idx(line_idx, col);
            let before = current_line[..byte_idx].to_string();
            let after = current_line[byte_idx..].to_string();

            // First line
            self.lines[line_idx] = before + lines_to_insert[0];

            // Middle lines
            for (i, line) in lines_to_insert.iter().enumerate().skip(1) {
                self.lines.insert(line_idx + i, line.to_string());
            }

            // Last line gets the "after" part
            let last_line_idx = line_idx + lines_to_insert.len() - 1;
            let last_inserted = lines_to_insert.last().unwrap();
            self.lines[last_line_idx].push_str(&after);

            self.cursor.y = last_line_idx as i16;
            self.cursor.x = last_inserted.chars().count() as i16;
        }

        self.modified = true;
        self.selection_start = None;
        self.ensure_cursor_visible();
    }
}

impl View for Memo {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;

        // Update scrollbar positions
        if self.v_scrollbar.is_some() {
            let v_bounds = Rect::new(
                bounds.b.x - 1,
                bounds.a.y,
                bounds.b.x,
                bounds.b.y - 1,
            );
            self.v_scrollbar.as_mut().unwrap().set_bounds(v_bounds);
        }

        if self.h_scrollbar.is_some() {
            let h_bounds = Rect::new(
                bounds.a.x,
                bounds.b.y - 1,
                bounds.b.x - 1,
                bounds.b.y,
            );
            self.h_scrollbar.as_mut().unwrap().set_bounds(h_bounds);
        }

        self.update_scrollbars();
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let content_area = self.get_content_area();
        let width = content_area.width() as usize;
        let height = content_area.height() as usize;

        // Use palette indices from CP_MEMO
        // 1 = Normal text, 2 = Selected/cursor text
        let color = self.map_color(1);
        let cursor_color = self.map_color(2);

        // Draw text content
        for y in 0..height {
            let line_idx = (self.delta.y + y as i16) as usize;
            let mut buf = DrawBuffer::new(width);

            buf.move_char(0, ' ', color, width);

            if line_idx < self.lines.len() {
                let line = &self.lines[line_idx];
                let start_col = self.delta.x as usize;
                let line_char_count = line.chars().count();

                if start_col < line_char_count {
                    // Calculate visible portion in CHARACTER positions
                    let end_col_char = min(start_col + width, line_char_count);

                    // Convert to string slice using character-based iteration
                    let visible_text: String = line
                        .chars()
                        .skip(start_col)
                        .take(end_col_char - start_col)
                        .collect();

                    buf.move_str(0, &visible_text, color);
                }
            }

            write_line_to_terminal(
                terminal,
                content_area.a.x,
                content_area.a.y + y as i16,
                &buf,
            );
        }

        // Draw cursor if focused
        if self.is_focused() {
            let cursor_screen_x = content_area.a.x + (self.cursor.x - self.delta.x);
            let cursor_screen_y = content_area.a.y + (self.cursor.y - self.delta.y);

            if cursor_screen_x >= content_area.a.x && cursor_screen_x < content_area.b.x
                && cursor_screen_y >= content_area.a.y && cursor_screen_y < content_area.b.y
            {
                // Draw cursor as inverted character
                let line_idx = self.cursor.y as usize;
                let col = self.cursor.x as usize;
                let ch = if line_idx < self.lines.len() {
                    self.lines[line_idx].chars().nth(col).unwrap_or(' ')
                } else {
                    ' '
                };

                let cursor_attr = cursor_color;
                terminal.write_cell(
                    cursor_screen_x as u16,
                    cursor_screen_y as u16,
                    crate::core::draw::Cell::new(ch, cursor_attr),
                );
            }
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
                // Only handle keyboard events if focused
                if !self.is_focused() {
                    return;
                }

                let shift_pressed = false; // TODO: Track shift state

                match event.key_code {
                    KB_UP => {
                        self.move_cursor(0, -1, shift_pressed);
                        event.clear();
                    }
                    KB_DOWN => {
                        self.move_cursor(0, 1, shift_pressed);
                        event.clear();
                    }
                    KB_LEFT => {
                        self.move_cursor(-1, 0, shift_pressed);
                        event.clear();
                    }
                    KB_RIGHT => {
                        self.move_cursor(1, 0, shift_pressed);
                        event.clear();
                    }
                    KB_HOME => {
                        self.cursor.x = 0;
                        self.selection_start = None;
                        self.ensure_cursor_visible();
                        event.clear();
                    }
                    KB_END => {
                        let line_len = self.lines[self.cursor.y as usize].chars().count() as i16;
                        self.cursor.x = line_len;
                        self.selection_start = None;
                        self.ensure_cursor_visible();
                        event.clear();
                    }
                    KB_PGUP => {
                        let height = self.get_content_area().height();
                        self.move_cursor(0, -height, shift_pressed);
                        event.clear();
                    }
                    KB_PGDN => {
                        let height = self.get_content_area().height();
                        self.move_cursor(0, height, shift_pressed);
                        event.clear();
                    }
                    KB_ENTER => {
                        self.insert_newline();
                        event.clear();
                    }
                    KB_BACKSPACE => {
                        if self.has_selection() {
                            self.delete_selection();
                        } else {
                            self.backspace();
                        }
                        event.clear();
                    }
                    KB_DEL => {
                        if self.has_selection() {
                            self.delete_selection();
                        } else {
                            self.delete_char();
                        }
                        event.clear();
                    }
                    KB_TAB => {
                        self.insert_tab();
                        event.clear();
                    }
                    KB_CTRL_A => {
                        self.select_all();
                        event.clear();
                    }
                    KB_CTRL_C => {
                        // Copy to clipboard
                        if let Some(selection) = self.get_selection() {
                            clipboard::set_clipboard(&selection);
                        }
                        event.clear();
                    }
                    KB_CTRL_X => {
                        // Cut to clipboard
                        if let Some(selection) = self.get_selection() {
                            clipboard::set_clipboard(&selection);
                            self.delete_selection();
                        }
                        event.clear();
                    }
                    KB_CTRL_V => {
                        // Paste from clipboard
                        let clipboard_text = clipboard::get_clipboard();
                        if !clipboard_text.is_empty() {
                            self.insert_text(&clipboard_text);
                        }
                        event.clear();
                    }
                    key_code => {
                        // Regular character input
                        if (32..127).contains(&key_code) {
                            let ch = key_code as u8 as char;
                            self.insert_char(ch);
                            event.clear();
                        }
                    }
                }
            }
            EventType::MouseWheelUp => {
                let mouse_pos = event.mouse.pos;
                let content_area = self.get_content_area();
                // Check if mouse is within the memo content area
                if mouse_pos.x >= content_area.a.x && mouse_pos.x < content_area.b.x &&
                   mouse_pos.y >= content_area.a.y && mouse_pos.y < content_area.b.y {
                    // Scroll up by moving cursor up (which automatically adjusts delta)
                    let shift_pressed = false;
                    self.move_cursor(0, -1, shift_pressed);
                    event.clear();
                }
            }
            EventType::MouseWheelDown => {
                let mouse_pos = event.mouse.pos;
                let content_area = self.get_content_area();
                // Check if mouse is within the memo content area
                if mouse_pos.x >= content_area.a.x && mouse_pos.x < content_area.b.x &&
                   mouse_pos.y >= content_area.a.y && mouse_pos.y < content_area.b.y {
                    // Scroll down by moving cursor down (which automatically adjusts delta)
                    let shift_pressed = false;
                    self.move_cursor(0, 1, shift_pressed);
                    event.clear();
                }
            }
            _ => {}
        }
    }

    fn can_focus(&self) -> bool {
        true
    }

    // set_focus() now uses default implementation from View trait
    // which sets/clears SF_FOCUSED flag

    fn state(&self) -> StateFlags {
        self.state
    }

    fn set_state(&mut self, state: StateFlags) {
        self.state = state;
    }

    fn update_cursor(&self, terminal: &mut Terminal) {
        if self.is_focused() {
            // Calculate cursor position on screen
            let cursor_x = self.bounds.a.x + (self.cursor.x - self.delta.x) as i16;
            let cursor_y = self.bounds.a.y + (self.cursor.y - self.delta.y) as i16;

            // Show cursor at the position
            let _ = terminal.show_cursor(cursor_x as u16, cursor_y as u16);
        }
    }

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{palettes, Palette};
        Some(Palette::from_slice(palettes::CP_MEMO))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memo_creation() {
        let memo = Memo::new(Rect::new(0, 0, 40, 10));
        assert_eq!(memo.get_text(), "");
        assert_eq!(memo.line_count(), 1);
        assert!(!memo.is_modified());
    }

    #[test]
    fn test_memo_set_text() {
        let mut memo = Memo::new(Rect::new(0, 0, 40, 10));
        memo.set_text("Line 1\nLine 2\nLine 3");
        assert_eq!(memo.line_count(), 3);
        assert_eq!(memo.get_text(), "Line 1\nLine 2\nLine 3");
        assert!(!memo.is_modified());
    }

    #[test]
    fn test_memo_insert_char() {
        let mut memo = Memo::new(Rect::new(0, 0, 40, 10));
        memo.set_text("Hello");

        memo.cursor = Point::new(5, 0);
        memo.insert_char('!');

        assert_eq!(memo.get_text(), "Hello!");
        assert!(memo.is_modified());
        assert_eq!(memo.cursor.x, 6);
    }

    #[test]
    fn test_memo_backspace() {
        let mut memo = Memo::new(Rect::new(0, 0, 40, 10));
        memo.set_text("Hello!");

        memo.cursor = Point::new(6, 0);
        memo.backspace();

        assert_eq!(memo.get_text(), "Hello");
        assert!(memo.is_modified());
        assert_eq!(memo.cursor.x, 5);
    }

    #[test]
    fn test_memo_delete_char() {
        let mut memo = Memo::new(Rect::new(0, 0, 40, 10));
        memo.set_text("Hello!");

        memo.cursor = Point::new(5, 0);
        memo.delete_char();

        assert_eq!(memo.get_text(), "Hello");
        assert!(memo.is_modified());
    }

    #[test]
    fn test_memo_insert_newline() {
        let mut memo = Memo::new(Rect::new(0, 0, 40, 10));
        memo.set_text("HelloWorld");

        memo.cursor = Point::new(5, 0);
        memo.insert_newline();

        assert_eq!(memo.get_text(), "Hello\nWorld");
        assert_eq!(memo.line_count(), 2);
        assert_eq!(memo.cursor, Point::new(0, 1));
    }

    #[test]
    fn test_memo_join_lines_backspace() {
        let mut memo = Memo::new(Rect::new(0, 0, 40, 10));
        memo.set_text("Hello\nWorld");

        memo.cursor = Point::new(0, 1);
        memo.backspace();

        assert_eq!(memo.get_text(), "HelloWorld");
        assert_eq!(memo.line_count(), 1);
        assert_eq!(memo.cursor.x, 5);
    }

    #[test]
    fn test_memo_read_only() {
        let mut memo = Memo::new(Rect::new(0, 0, 40, 10));
        memo.set_text("Hello");
        memo.set_read_only(true);

        memo.cursor = Point::new(5, 0);
        memo.insert_char('!');

        assert_eq!(memo.get_text(), "Hello");
        assert!(!memo.is_modified());
    }

    #[test]
    fn test_memo_max_length() {
        let mut memo = Memo::new(Rect::new(0, 0, 40, 10));
        memo.set_max_length(Some(5));
        memo.set_text("Hello");

        memo.cursor = Point::new(5, 0);
        memo.insert_char('!');

        assert_eq!(memo.get_text(), "Hello");
    }
}

/// Builder for creating memos with a fluent API.
pub struct MemoBuilder {
    bounds: Option<Rect>,
    with_scrollbars: bool,
    max_length: Option<usize>,
    read_only: bool,
    tab_size: usize,
}

impl MemoBuilder {
    pub fn new() -> Self {
        Self {
            bounds: None,
            with_scrollbars: false,
            max_length: None,
            read_only: false,
            tab_size: 4,
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
    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    #[must_use]
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    #[must_use]
    pub fn tab_size(mut self, tab_size: usize) -> Self {
        self.tab_size = tab_size;
        self
    }

    pub fn build(self) -> Memo {
        let bounds = self.bounds.expect("Memo bounds must be set");
        let mut memo = Memo::new(bounds).with_scrollbars(self.with_scrollbars);
        memo.set_max_length(self.max_length);
        memo.set_read_only(self.read_only);
        memo.set_tab_size(self.tab_size);
        memo
    }

    pub fn build_boxed(self) -> Box<Memo> {
        Box::new(self.build())
    }
}

impl Default for MemoBuilder {
    fn default() -> Self {
        Self::new()
    }
}
