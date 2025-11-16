// (C) 2025 - Enzo Lombardi

//! InputLine view - single-line text input with editing and history support.

use super::validator::ValidatorRef;
use super::view::{write_line_to_terminal, View};
use crate::core::clipboard;
use crate::core::draw::DrawBuffer;
use crate::core::event::{
    Event, EventType, KB_BACKSPACE, KB_DEL, KB_END, KB_ENTER, KB_HOME, KB_LEFT, KB_RIGHT,
};
use crate::core::geometry::Rect;
use crate::core::palette::{INPUT_ARROWS, INPUT_FOCUSED, INPUT_NORMAL, INPUT_SELECTED};
use crate::core::state::StateFlags;
use crate::terminal::Terminal;
use std::cell::RefCell;
use std::rc::Rc;

// Control key codes
const KB_CTRL_A: u16 = 0x0001; // Ctrl+A - Select All
const KB_CTRL_C: u16 = 0x0003; // Ctrl+C - Copy
const KB_CTRL_V: u16 = 0x0016; // Ctrl+V - Paste
const KB_CTRL_X: u16 = 0x0018; // Ctrl+X - Cut

pub struct InputLine {
    bounds: Rect,
    data: Rc<RefCell<String>>,
    cursor_pos: usize,
    max_length: usize,
    sel_start: usize,                // Selection start position
    sel_end: usize,                  // Selection end position
    first_pos: usize,                // First visible character position for horizontal scrolling
    validator: Option<ValidatorRef>, // Optional validator for input validation
    state: StateFlags,               // View state flags (including SF_FOCUSED)
    owner: Option<*const dyn View>,
    owner_type: super::view::OwnerType,
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
            owner_type: super::view::OwnerType::Dialog, // InputLine defaults to Dialog context
        }
    }

    /// Create an InputLine with a validator
    /// Matches Borland's TInputLine with validator attachment pattern
    pub fn with_validator(
        bounds: Rect,
        max_length: usize,
        data: Rc<RefCell<String>>,
        validator: ValidatorRef,
    ) -> Self {
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
        let width = self.bounds.width_clamped() as usize;

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
        let width = self.bounds.width_clamped() as usize;

        // Don't render input lines that are too small
        // Minimum width: 1 (at least 1 char visible)
        if width < 1 {
            return;
        }

        let mut buf = DrawBuffer::new(width);

        // InputLine palette indices:
        // 1: Normal, 2: Focused, 3: Selected, 4: Arrows
        let attr = if self.is_focused() {
            self.map_color(INPUT_FOCUSED) // Focused
        } else {
            self.map_color(INPUT_NORMAL) // Normal
        };

        let sel_attr = self.map_color(INPUT_SELECTED); // Selected text
        let arrow_attr = self.map_color(INPUT_ARROWS); // Arrow indicators

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
        } else {
            // Explicitly hide cursor when not focused to prevent it from lingering
            // after dialogs close. This ensures clean cursor state management.
            let _ = terminal.hide_cursor();
        }
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

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{palettes, Palette};
        Some(Palette::from_slice(palettes::CP_INPUT_LINE))
    }
}

/// Builder for creating input lines with a fluent API.
///
/// # Examples
///
/// ```ignore
/// use turbo_vision::views::input_line::InputLineBuilder;
/// use turbo_vision::core::geometry::Rect;
/// use std::rc::Rc;
/// use std::cell::RefCell;
///
/// // Create a basic input line
/// let data = Rc::new(RefCell::new(String::new()));
/// let input = InputLineBuilder::new()
///     .bounds(Rect::new(10, 5, 50, 6))
///     .data(data.clone())
///     .max_length(30)
///     .build();
///
/// // Create an input line with validator
/// let data = Rc::new(RefCell::new(String::new()));
/// let input = InputLineBuilder::new()
///     .bounds(Rect::new(10, 5, 50, 6))
///     .data(data.clone())
///     .max_length(10)
///     .validator(some_validator)
///     .build();
/// ```
pub struct InputLineBuilder {
    bounds: Option<Rect>,
    data: Option<Rc<RefCell<String>>>,
    max_length: usize,
    validator: Option<ValidatorRef>,
}

impl InputLineBuilder {
    /// Creates a new InputLineBuilder with default values.
    pub fn new() -> Self {
        Self {
            bounds: None,
            data: None,
            max_length: 255,
            validator: None,
        }
    }

    /// Sets the input line bounds (required).
    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Sets the shared data reference (required).
    #[must_use]
    pub fn data(mut self, data: Rc<RefCell<String>>) -> Self {
        self.data = Some(data);
        self
    }

    /// Sets the maximum length (default: 255).
    #[must_use]
    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = max_length;
        self
    }

    /// Sets the validator for input validation (optional).
    #[must_use]
    pub fn validator(mut self, validator: ValidatorRef) -> Self {
        self.validator = Some(validator);
        self
    }

    /// Builds the InputLine.
    ///
    /// # Panics
    ///
    /// Panics if required fields (bounds, data) are not set.
    pub fn build(self) -> InputLine {
        let bounds = self.bounds.expect("InputLine bounds must be set");
        let data = self.data.expect("InputLine data must be set");

        let mut input_line = InputLine::new(bounds, self.max_length, data);
        if let Some(validator) = self.validator {
            input_line.validator = Some(validator);
        }
        input_line
    }

    /// Builds the InputLine as a Box.
    pub fn build_boxed(self) -> Box<InputLine> {
        Box::new(self.build())
    }
}

impl Default for InputLineBuilder {
    fn default() -> Self {
        Self::new()
    }
}
