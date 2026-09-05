// (C) 2025 - Enzo Lombardi

//! InputLine view - single-line text input with editing and history support.

use super::validator::ValidatorRef;
use super::view::{View, ViewCore, write_line_to_terminal};
use crate::core::clipboard;
use crate::core::draw::DrawBuffer;
use crate::core::event::{
    Event, EventType, KB_BACKSPACE, KB_DEL, KB_END, KB_ENTER, KB_HOME, KB_LEFT, KB_RIGHT,
};
use crate::core::geometry::Rect;
use crate::core::palette::{INPUT_ARROWS, INPUT_FOCUSED, INPUT_NORMAL, INPUT_SELECTED};
use crate::terminal::Terminal;
use std::cell::RefCell;
use std::rc::Rc;

// Control key codes
const KB_CTRL_A: u16 = 0x0001; // Ctrl+A - Select All
const KB_CTRL_C: u16 = 0x0003; // Ctrl+C - Copy
const KB_CTRL_V: u16 = 0x0016; // Ctrl+V - Paste
const KB_CTRL_X: u16 = 0x0018; // Ctrl+X - Cut

/// Converts a char index into a byte offset, clamping to the end of `text`.
fn byte_offset(text: &str, char_idx: usize) -> usize {
    text.char_indices()
        .nth(char_idx)
        .map_or(text.len(), |(i, _)| i)
}

/// Counts the characters in `text`.
fn char_len(text: &str) -> usize {
    text.chars().count()
}

pub struct InputLine {
    core: ViewCore,
    data: Rc<RefCell<String>>,
    cursor_pos: usize,               // Cursor position in characters (not bytes)
    max_length: usize,               // Maximum length in characters
    sel_start: usize,                // Selection start position (characters)
    sel_end: usize,                  // Selection end position (characters)
    first_pos: usize,                // First visible character position for horizontal scrolling
    validator: Option<ValidatorRef>, // Optional validator for input validation
    insert_mode: bool,               // Ins toggles; overwrite replaces at cursor
}

impl InputLine {
    pub fn new(bounds: Rect, max_length: usize, data: Rc<RefCell<String>>) -> Self {
        let cursor_pos = char_len(&data.borrow());
        Self {
            core: ViewCore {
                bounds,
                state: 0,
                palette_chain: None,
                ..ViewCore::default()
            },
            data,
            cursor_pos,
            max_length,
            sel_start: 0,
            sel_end: 0,
            first_pos: 0,
            validator: None,
            insert_mode: true,
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
        self.cursor_pos = char_len(&self.data.borrow());
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
        let len = char_len(&self.data.borrow());
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
        let start = byte_offset(&text, self.sel_start.min(self.sel_end));
        let end = byte_offset(&text, self.sel_start.max(self.sel_end));
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
        let byte_start = byte_offset(&text, start);
        let byte_end = byte_offset(&text, end);
        text.replace_range(byte_start..byte_end, "");
        drop(text);

        self.cursor_pos = start;
        self.sel_start = 0;
        self.sel_end = 0;
    }

    /// Ensure cursor is visible by adjusting first_pos
    fn make_cursor_visible(&mut self) {
        let width = self.core.bounds.width_clamped() as usize;

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
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.core.bounds.width_clamped() as usize;

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

        // Get text and calculate visible portion (all positions in characters)
        let text = self.data.borrow();
        let text_len = char_len(&text);

        // Calculate visible range
        let visible_start = self.first_pos;
        let visible_end = (visible_start + width).min(text_len);

        // Draw text
        if visible_start < text_len {
            let byte_start = byte_offset(&text, visible_start);
            let byte_end = byte_offset(&text, visible_end);
            let visible_text = &text[byte_start..byte_end];

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

        write_line_to_terminal(terminal, self.core.bounds.a.x, self.core.bounds.a.y, &buf);
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
                    self.cursor_pos = char_len(&self.data.borrow());
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
                            let at = byte_offset(&text, self.cursor_pos - 1);
                            text.remove(at);
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
                    } else if self.cursor_pos < char_len(&self.data.borrow()) {
                        let mut text = self.data.borrow_mut();
                        let at = byte_offset(&text, self.cursor_pos);
                        text.remove(at);
                        event.clear();
                    }
                }
                KB_LEFT => {
                    let shift = event
                        .key_modifiers
                        .contains(crossterm::event::KeyModifiers::SHIFT);
                    if self.cursor_pos > 0 {
                        if shift {
                            // Anchor the selection at the old cursor position
                            if !self.has_selection() {
                                self.sel_start = self.cursor_pos;
                            }
                            self.cursor_pos -= 1;
                            self.sel_end = self.cursor_pos;
                        } else {
                            self.cursor_pos -= 1;
                            self.sel_start = 0;
                            self.sel_end = 0;
                        }
                        self.make_cursor_visible();
                        event.clear();
                    } else if !shift && self.has_selection() {
                        self.sel_start = 0;
                        self.sel_end = 0;
                        event.clear();
                    }
                }
                KB_RIGHT => {
                    let shift = event
                        .key_modifiers
                        .contains(crossterm::event::KeyModifiers::SHIFT);
                    if self.cursor_pos < char_len(&self.data.borrow()) {
                        if shift {
                            if !self.has_selection() {
                                self.sel_start = self.cursor_pos;
                            }
                            self.cursor_pos += 1;
                            self.sel_end = self.cursor_pos;
                        } else {
                            self.cursor_pos += 1;
                            self.sel_start = 0;
                            self.sel_end = 0;
                        }
                        self.make_cursor_visible();
                        event.clear();
                    } else if !shift && self.has_selection() {
                        self.sel_start = 0;
                        self.sel_end = 0;
                        event.clear();
                    }
                }
                crate::core::event::KB_INS => {
                    // Toggle insert/overwrite (Borland: sfCursorIns)
                    self.insert_mode = !self.insert_mode;
                    event.clear();
                }
                KB_HOME => {
                    self.cursor_pos = 0;
                    self.sel_start = 0;
                    self.sel_end = 0;
                    self.make_cursor_visible();
                    event.clear();
                }
                KB_END => {
                    self.cursor_pos = char_len(&self.data.borrow());
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

                        // Insert clipboard text at cursor position (truncated to fit,
                        // counting characters so multibyte input can't split)
                        {
                            let mut text = self.data.borrow_mut();
                            let remaining_space = self.max_length.saturating_sub(char_len(&text));
                            let cut = byte_offset(&clipboard_text, remaining_space);
                            let insert_text = &clipboard_text[..cut];

                            let at = byte_offset(&text, self.cursor_pos);
                            text.insert_str(at, insert_text);
                            self.cursor_pos += char_len(insert_text);
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

                        let text_len = char_len(&self.data.borrow());
                        if text_len < self.max_length {
                            let ch = key_code as u8 as char;

                            // Check validator before inserting
                            // Matches Borland's TValidator::IsValidInput() pattern
                            if let Some(ref validator) = self.validator {
                                // Create test string with new character
                                let mut test_text = self.data.borrow().clone();
                                let at = byte_offset(&test_text, self.cursor_pos);
                                test_text.insert(at, ch);

                                // Check if valid input during typing
                                if !validator.borrow().is_valid_input(&test_text, true) {
                                    // Invalid character - reject it
                                    event.clear();
                                    return;
                                }
                            }

                            // Character is valid, insert it (overwrite mode
                            // replaces the character under the cursor)
                            {
                                let mut text = self.data.borrow_mut();
                                let at = byte_offset(&text, self.cursor_pos);
                                if !self.insert_mode && self.cursor_pos < char_len(&text) {
                                    text.remove(at);
                                }
                                text.insert(at, ch);
                            }
                            self.cursor_pos += 1;

                            // Let the validator auto-fill/transform the text.
                            // Matches Borland: TInputLine::handleEvent() calls
                            // checkValid(False) after inserting a character,
                            // which lets picture masks insert literals
                            // ("12" + mask "##/##" -> "12/") and force
                            // uppercase for `&`/`!` positions.
                            if let Some(ref validator) = self.validator {
                                let filled = validator.borrow().complete(&self.data.borrow());
                                if let Some(filled) = filled {
                                    let filled_len = char_len(&filled);
                                    let old_len = char_len(&self.data.borrow());
                                    *self.data.borrow_mut() = filled;
                                    if filled_len > old_len {
                                        // Literals were appended: move the
                                        // char-based cursor past the fill.
                                        self.cursor_pos = filled_len.min(self.max_length);
                                    } else {
                                        self.cursor_pos = self.cursor_pos.min(filled_len);
                                    }
                                }
                            }

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

    /// Select all text when the field gains focus (Borland:
    /// TInputLine::setState(sfSelected) selects the content so typing
    /// replaces it); losing focus collapses the selection.
    fn set_focus(&mut self, focused: bool) {
        let was_focused = self.is_focused();
        self.set_state_flag(crate::core::state::SF_FOCUSED, focused);
        if focused && !was_focused {
            self.select_all();
        } else if !focused {
            self.sel_start = 0;
            self.sel_end = 0;
        }
    }

    /// Validate on modal close (Borland: TInputLine::valid via checkValid).
    ///
    /// Cancel always passes; any other end state consults the attached
    /// validator, letting a failing validator veto the dialog close.
    fn valid(&mut self, command: crate::core::command::CommandId) -> bool {
        use crate::core::command::CM_CANCEL;
        if command == CM_CANCEL {
            return true;
        }
        self.validate()
    }

    // set_focus() now uses default implementation from View trait
    // which sets/clears SF_FOCUSED flag

    fn update_cursor(&self, terminal: &mut Terminal) {
        if self.is_focused() {
            // Calculate cursor position on screen
            let cursor_x = self.core.bounds.a.x as usize + (self.cursor_pos - self.first_pos);
            let cursor_y = self.core.bounds.a.y;

            // Show cursor at the position
            let _ = terminal.show_cursor(cursor_x as u16, cursor_y as u16);
        } else {
            // Explicitly hide cursor when not focused to prevent it from lingering
            // after dialogs close. This ensures clean cursor state management.
            let _ = terminal.hide_cursor();
        }
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        Some(Palette::from_slice(palettes::CP_INPUT_LINE))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::{KB_BACKSPACE, KB_DEL, KB_LEFT};

    fn make(text: &str) -> (InputLine, Rc<RefCell<String>>) {
        let data = Rc::new(RefCell::new(text.to_string()));
        let mut input = InputLine::new(Rect::new(0, 0, 20, 1), 10, data.clone());
        input.set_focus(true);
        (input, data)
    }

    #[test]
    fn multibyte_backspace_and_delete_do_not_panic() {
        // Regression: cursor positions were byte offsets, panicking on "é"
        let (mut input, data) = make("héllo");
        let mut ev = Event::keyboard(KB_LEFT);
        input.handle_event(&mut ev);
        let mut ev = Event::keyboard(KB_BACKSPACE);
        input.handle_event(&mut ev);
        assert_eq!(*data.borrow(), "hélo");

        let mut ev = Event::keyboard(KB_LEFT);
        input.handle_event(&mut ev);
        let mut ev = Event::keyboard(KB_DEL);
        input.handle_event(&mut ev);
        assert_eq!(*data.borrow(), "héo");
    }

    #[test]
    fn multibyte_selection_and_typing() {
        let (mut input, data) = make("héé");
        input.select_all();
        assert_eq!(input.get_selection().as_deref(), Some("héé"));

        // Typing over the selection replaces it, at char boundaries
        let mut ev = Event::keyboard('x' as u16);
        input.handle_event(&mut ev);
        assert_eq!(*data.borrow(), "x");
    }

    #[test]
    fn picture_validator_auto_fills_literals_while_typing() {
        use crate::views::picture_validator::picture_validator;

        let data = Rc::new(RefCell::new(String::new()));
        let mut input = InputLine::with_validator(
            Rect::new(0, 0, 20, 1),
            10,
            data.clone(),
            picture_validator("##/##"),
        );
        input.set_focus(true);

        for ch in ['1', '2'] {
            let mut ev = Event::keyboard(ch as u16);
            input.handle_event(&mut ev);
        }
        // The '/' literal is auto-inserted and the cursor moves past it
        assert_eq!(*data.borrow(), "12/");
        assert_eq!(input.cursor_pos, 3);

        for ch in ['3', '4'] {
            let mut ev = Event::keyboard(ch as u16);
            input.handle_event(&mut ev);
        }
        assert_eq!(*data.borrow(), "12/34");
    }

    #[test]
    fn picture_validator_forces_uppercase_while_typing() {
        use crate::views::picture_validator::picture_validator;

        // `!` = any char uppercased, `&` = letter uppercased
        let data = Rc::new(RefCell::new(String::new()));
        let mut input = InputLine::with_validator(
            Rect::new(0, 0, 20, 1),
            10,
            data.clone(),
            picture_validator("!&"),
        );
        input.set_focus(true);

        for ch in ['a', 'b'] {
            let mut ev = Event::keyboard(ch as u16);
            input.handle_event(&mut ev);
        }
        assert_eq!(*data.borrow(), "AB");
        assert_eq!(input.cursor_pos, 2);
    }

    #[test]
    fn picture_validator_rejects_invalid_chars() {
        use crate::views::picture_validator::picture_validator;

        let data = Rc::new(RefCell::new(String::new()));
        let mut input = InputLine::with_validator(
            Rect::new(0, 0, 20, 1),
            10,
            data.clone(),
            picture_validator("###"),
        );
        input.set_focus(true);

        let mut ev = Event::keyboard('x' as u16);
        input.handle_event(&mut ev);
        assert_eq!(*data.borrow(), "");
    }

    #[test]
    fn paste_truncates_by_characters() {
        // max_length 10; pasting 12 chars of multibyte text must cut at a
        // char boundary, not mid-code-point
        let (mut input, data) = make("");
        crate::core::clipboard::set_clipboard("éééééééééééé");
        let mut ev = Event::keyboard(0x0016); // Ctrl+V
        input.handle_event(&mut ev);
        assert_eq!(data.borrow().chars().count(), 10);
        assert!(data.borrow().chars().all(|c| c == 'é'));
    }

    #[test]
    fn failing_validator_vetoes_dialog_close() {
        use crate::views::validator::Validator;
        use std::cell::RefCell as RC;

        struct RejectAll;
        impl Validator for RejectAll {
            fn is_valid(&self, _input: &str) -> bool {
                false
            }
            fn is_valid_input(&self, _input: &str, _append: bool) -> bool {
                true
            }
            fn error(&self) {}
        }

        let data = Rc::new(RefCell::new("anything".to_string()));
        let mut input = InputLine::with_validator(
            Rect::new(0, 0, 10, 1),
            10,
            data,
            Rc::new(RC::new(RejectAll)),
        );
        use crate::core::command::{CM_CANCEL, CM_OK};
        assert!(!View::valid(&mut input, CM_OK));
        assert!(View::valid(&mut input, CM_CANCEL));
    }

    #[test]
    fn focus_selects_all_and_typing_replaces() {
        let data = Rc::new(RefCell::new("hello".to_string()));
        let mut input = InputLine::new(Rect::new(0, 0, 20, 1), 10, data.clone());
        input.set_focus(true);
        assert_eq!(input.get_selection().as_deref(), Some("hello"));

        let mut ev = Event::keyboard('x' as u16);
        input.handle_event(&mut ev);
        assert_eq!(*data.borrow(), "x");

        // Losing focus collapses any selection
        input.select_all();
        input.set_focus(false);
        assert!(!input.has_selection());
    }

    #[test]
    fn shift_arrows_extend_selection() {
        use crossterm::event::KeyModifiers;

        let (mut input, _data) = make("abcd");
        // Collapse the focus-selection and put the cursor at the end
        let mut ev = Event::keyboard(KB_RIGHT);
        input.handle_event(&mut ev);

        let mut ev = Event::keyboard(KB_LEFT);
        ev.key_modifiers = KeyModifiers::SHIFT;
        input.handle_event(&mut ev);
        let mut ev = Event::keyboard(KB_LEFT);
        ev.key_modifiers = KeyModifiers::SHIFT;
        input.handle_event(&mut ev);
        assert_eq!(input.get_selection().as_deref(), Some("cd"));

        // Plain arrow collapses the selection
        let mut ev = Event::keyboard(KB_RIGHT);
        input.handle_event(&mut ev);
        assert!(!input.has_selection());
    }

    #[test]
    fn ins_toggles_overwrite_mode() {
        let (mut input, data) = make("abc");
        // Move cursor to start (collapse focus selection first)
        let mut ev = Event::keyboard(crate::core::event::KB_HOME);
        input.handle_event(&mut ev);

        let mut ev = Event::keyboard(crate::core::event::KB_INS);
        input.handle_event(&mut ev);
        let mut ev = Event::keyboard('X' as u16);
        input.handle_event(&mut ev);
        assert_eq!(*data.borrow(), "Xbc");

        // Back to insert mode
        let mut ev = Event::keyboard(crate::core::event::KB_INS);
        input.handle_event(&mut ev);
        let mut ev = Event::keyboard('Y' as u16);
        input.handle_event(&mut ev);
        assert_eq!(*data.borrow(), "XYbc");
    }
}
