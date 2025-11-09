// (C) 2025 - Enzo Lombardi

//! InputLine view - single-line text input with editing and history support.

use crate::core::geometry::Rect;
use crate::core::event::{Event, EventType, KB_ENTER, KB_BACKSPACE, KB_LEFT, KB_RIGHT, KB_HOME, KB_END, KB_DEL};
use crate::core::draw::DrawBuffer;
use crate::core::clipboard;
use crate::core::state::StateFlags;
use crate::terminal::Terminal;
use super::view::{View, write_line_to_terminal};
use super::validator::ValidatorRef;
use std::rc::Rc;
use std::cell::RefCell;

// Control key codes
const KB_CTRL_A: u16 = 0x0001;  // Ctrl+A - Select All
const KB_CTRL_C: u16 = 0x0003;  // Ctrl+C - Copy
const KB_CTRL_V: u16 = 0x0016;  // Ctrl+V - Paste
const KB_CTRL_X: u16 = 0x0018;  // Ctrl+X - Cut

pub struct InputLine {
    bounds: Rect,
    data: Rc<RefCell<String>>,
    cursor_pos: usize,
    max_length: usize,
    sel_start: usize,      // Selection start position
    sel_end: usize,        // Selection end position
    first_pos: usize,      // First visible character position for horizontal scrolling
    validator: Option<ValidatorRef>,  // Optional validator for input validation
    state: StateFlags,     // View state flags (including SF_FOCUSED)
    owner: Option<*const dyn View>,
}

impl InputLine {
    pub fn new(bounds: Rect, max_length: usize, data: Rc<RefCell<String>>) -> Self {
        let cursor_pos = data.borrow().len();
        Self {
            bounds,
            data,
            cursor_pos,
            max_length,
            sel_start: 0,
            sel_end: 0,
            first_pos: 0,
            validator: None,
            state: 0,
            owner: None,
        }
    }

    /// Create an InputLine with a validator
    /// Matches Borland's TInputLine with validator attachment pattern
    pub fn with_validator(bounds: Rect, max_length: usize, data: Rc<RefCell<String>>, validator: ValidatorRef) -> Self {
        let mut input_line = Self::new(bounds, max_length, data);
        input_line.validator = Some(validator);
        input_line
    }

    /// Set the validator for this InputLine
    pub fn set_validator(&mut self, validator: ValidatorRef) {
        self.validator = Some(validator);
    }

    /// Validate the current input
    /// Returns true if valid or no validator is set
    pub fn validate(&self) -> bool {
        if let Some(ref validator) = self.validator {
            validator.borrow().valid(&self.data.borrow())
        } else {
            true
        }
    }

    pub fn set_text(&mut self, text: String) {
        *self.data.borrow_mut() = text;
        self.cursor_pos = self.data.borrow().len();
        self.sel_start = 0;
        self.sel_end = 0;
        self.first_pos = 0;
    }

    pub fn get_text(&self) -> String {
        self.data.borrow().clone()
    }

    // set_focused() removed - use set_focus() from View trait instead

    /// Select all text
    pub fn select_all(&mut self) {
        let len = self.data.borrow().len();
        self.sel_start = 0;
        self.sel_end = len;
        self.cursor_pos = len;
    }

    /// Check if there's an active selection
    pub fn has_selection(&self) -> bool {
        self.sel_start != self.sel_end
    }

    /// Get the selected text
    pub fn get_selection(&self) -> Option<String> {
        if !self.has_selection() {
            return None;
        }
        let text = self.data.borrow();
        let start = self.sel_start.min(self.sel_end);
        let end = self.sel_start.max(self.sel_end);
        Some(text[start..end].to_string())
    }

    /// Delete the current selection
    fn delete_selection(&mut self) {
        if !self.has_selection() {
            return;
        }
        let start = self.sel_start.min(self.sel_end);
        let end = self.sel_start.max(self.sel_end);

        let mut text = self.data.borrow_mut();
        text.replace_range(start..end, "");
        drop(text);

        self.cursor_pos = start;
        self.sel_start = 0;
        self.sel_end = 0;
    }

    /// Ensure cursor is visible by adjusting first_pos
    fn make_cursor_visible(&mut self) {
        let width = self.bounds.width() as usize;

        // If cursor is before the visible area
        if self.cursor_pos < self.first_pos {
            self.first_pos = self.cursor_pos;
        }
        // If cursor is after the visible area
        else if self.cursor_pos >= self.first_pos + width {
            self.first_pos = self.cursor_pos - width + 1;
        }
    }
}

impl View for InputLine {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width() as usize;
        let mut buf = DrawBuffer::new(width);

        // InputLine palette indices:
        // 1: Normal, 2: Focused, 3: Selected, 4: Arrows
        let attr = if self.is_focused() {
            self.map_color(2)  // Focused
        } else {
            self.map_color(1)  // Normal
        };

        let sel_attr = self.map_color(3);  // Selected text
        let arrow_attr = self.map_color(4);  // Arrow indicators

        buf.move_char(0, ' ', attr, width);

        // Get text and calculate visible portion
        let text = self.data.borrow();
        let text_len = text.len();

        // Calculate visible range
        let visible_start = self.first_pos;
        let visible_end = (visible_start + width).min(text_len);

        // Draw text
        if visible_start < text_len {
            let visible_text = &text[visible_start..visible_end];

            // If there's a selection, draw it with selection color
            if self.has_selection() {
                let sel_start = self.sel_start.min(self.sel_end);
                let sel_end = self.sel_start.max(self.sel_end);

                // Draw characters one by one to handle selection highlighting
                for (i, ch) in visible_text.chars().enumerate() {
                    let pos = visible_start + i;
                    let char_attr = if pos >= sel_start && pos < sel_end {
                        sel_attr
                    } else {
                        attr
                    };
                    buf.move_char(i, ch, char_attr, 1);
                }
            } else {
                buf.move_str(0, visible_text, attr);
            }

            // Show left arrow if text is scrolled
            if self.first_pos > 0 {
                buf.move_char(0, '<', arrow_attr, 1);
            }

            // Show right arrow if there's more text beyond the visible area
            if visible_end < text_len {
                buf.move_char(width - 1, '>', arrow_attr, 1);
            }
        }

        write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y, &buf);
    }

    fn handle_event(&mut self, event: &mut Event) {
        // Handle broadcasts even when not focused
        if event.what == EventType::Broadcast {
            use crate::core::command::CM_FILE_FOCUSED;

            // Handle cmFileFocused broadcast from FileDialog
            // Matches Borland: TFileInputLine::handleEvent() (tfileinp.cc:35-45)
            if event.command == CM_FILE_FOCUSED {
                // Only update display if user isn't currently typing
                // Matches Borland: if( !(state & sfSelected) )
                if !self.is_focused() {
                    // The data has already been updated by FileDialog
                    // Just need to update our cursor position and clear selection
                    self.cursor_pos = self.data.borrow().len();
                    self.sel_start = 0;
                    self.sel_end = 0;
                    self.first_pos = 0;
                    // Note: Event is NOT cleared - other views may need it
                }
            }
            return;
        }

        if !self.is_focused() {
            return;
        }

        if event.what == EventType::Keyboard {
            match event.key_code {
                KB_BACKSPACE => {
                    if self.has_selection() {
                        self.delete_selection();
                        self.make_cursor_visible();
                        event.clear();
                    } else if self.cursor_pos > 0 {
                        {
                            let mut text = self.data.borrow_mut();
                            text.remove(self.cursor_pos - 1);
                        }
                        self.cursor_pos -= 1;
                        self.make_cursor_visible();
                        event.clear();
                    }
                }
                KB_DEL => {
                    if self.has_selection() {
                        self.delete_selection();
                        self.make_cursor_visible();
                        event.clear();
                    } else if self.cursor_pos < self.data.borrow().len() {
                        let mut text = self.data.borrow_mut();
                        text.remove(self.cursor_pos);
                        event.clear();
                    }
                }
                KB_LEFT => {
                    if self.cursor_pos > 0 {
                        self.cursor_pos -= 1;
                        self.sel_start = 0;
                        self.sel_end = 0;
                        self.make_cursor_visible();
                        event.clear();
                    }
                }
                KB_RIGHT => {
                    if self.cursor_pos < self.data.borrow().len() {
                        self.cursor_pos += 1;
                        self.sel_start = 0;
                        self.sel_end = 0;
                        self.make_cursor_visible();
                        event.clear();
                    }
                }
                KB_HOME => {
                    self.cursor_pos = 0;
                    self.sel_start = 0;
                    self.sel_end = 0;
                    self.make_cursor_visible();
                    event.clear();
                }
                KB_END => {
                    self.cursor_pos = self.data.borrow().len();
                    self.sel_start = 0;
                    self.sel_end = 0;
                    self.make_cursor_visible();
                    event.clear();
                }
                KB_ENTER => {
                    // Don't handle Enter - let dialog handle it for default button
                    // Just pass through without clearing
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
                        self.make_cursor_visible();
                    }
                    event.clear();
                }
                KB_CTRL_V => {
                    // Paste from clipboard
                    let clipboard_text = clipboard::get_clipboard();
                    if !clipboard_text.is_empty() {
                        // Delete selection if any
                        if self.has_selection() {
                            self.delete_selection();
                        }

                        // Insert clipboard text at cursor position
                        {
                            let mut text = self.data.borrow_mut();
                            let remaining_space = self.max_length.saturating_sub(text.len());
                            let insert_text = if clipboard_text.len() <= remaining_space {
                                clipboard_text.as_str()
                            } else {
                                &clipboard_text[..remaining_space]
                            };

                            text.insert_str(self.cursor_pos, insert_text);
                            self.cursor_pos += insert_text.len();
                        }
                        self.make_cursor_visible();
                    }
                    event.clear();
                }
                // Regular character input
                key_code => {
                    if (32..127).contains(&key_code) {
                        // Delete selection if any
                        if self.has_selection() {
                            self.delete_selection();
                        }

                        let text_len = self.data.borrow().len();
                        if text_len < self.max_length {
                            let ch = key_code as u8 as char;

                            // Check validator before inserting
                            // Matches Borland's TValidator::IsValidInput() pattern
                            if let Some(ref validator) = self.validator {
                                // Create test string with new character
                                let mut test_text = self.data.borrow().clone();
                                test_text.insert(self.cursor_pos, ch);

                                // Check if valid input during typing
                                if !validator.borrow().is_valid_input(&test_text, true) {
                                    // Invalid character - reject it
                                    event.clear();
                                    return;
                                }
                            }

                            // Character is valid, insert it
                            {
                                let mut text = self.data.borrow_mut();
                                text.insert(self.cursor_pos, ch);
                            }
                            self.cursor_pos += 1;
                            self.make_cursor_visible();
                            event.clear();
                        }
                    }
                }
            }
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
            let cursor_x = self.bounds.a.x as usize + (self.cursor_pos - self.first_pos);
            let cursor_y = self.bounds.a.y;

            // Show cursor at the position
            let _ = terminal.show_cursor(cursor_x as u16, cursor_y as u16);
        }
        // Note: cursor is already hidden by Group if not focused
    }

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        Some(Palette::from_slice(palettes::CP_INPUT_LINE))
    }
}
