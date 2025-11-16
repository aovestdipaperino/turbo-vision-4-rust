// (C) 2025 - Enzo Lombardi

//! Editor view - advanced multi-line text editor with syntax highlighting support.

use crate::core::geometry::{Point, Rect};
use crate::core::event::{Event, EventType, KB_UP, KB_DOWN, KB_LEFT, KB_RIGHT, KB_PGUP, KB_PGDN, KB_HOME, KB_END, KB_ENTER, KB_BACKSPACE, KB_DEL, KB_TAB, MB_LEFT_BUTTON};
use crate::core::draw::DrawBuffer;
use crate::core::clipboard;
use crate::core::state::StateFlags;
use crate::terminal::Terminal;
use super::view::{View, write_line_to_terminal};
use super::scrollbar::ScrollBar;
use super::indicator::Indicator;
use super::syntax::SyntaxHighlighter;
use std::cmp::min;
use std::rc::Rc;
use std::cell::RefCell;

// Control key codes
const KB_CTRL_A: u16 = 0x0001;  // Ctrl+A - Select All
const KB_CTRL_C: u16 = 0x0003;  // Ctrl+C - Copy
#[expect(dead_code, reason = "Reserved for future find/replace functionality")]
const KB_CTRL_F: u16 = 0x0006;  // Ctrl+F - Find
#[expect(dead_code, reason = "Reserved for future find/replace functionality")]
const KB_CTRL_H: u16 = 0x0008;  // Ctrl+H - Replace
const KB_CTRL_V: u16 = 0x0016;  // Ctrl+V - Paste
const KB_CTRL_X: u16 = 0x0018;  // Ctrl+X - Cut
const KB_CTRL_Y: u16 = 0x0019;  // Ctrl+Y - Redo
const KB_CTRL_Z: u16 = 0x001A;  // Ctrl+Z - Undo

/// Maximum undo history size
const MAX_UNDO_HISTORY: usize = 100;

/// Search options flags (matching Borland's efXXX constants)
#[derive(Clone, Copy, Debug)]
pub struct SearchOptions {
    pub case_sensitive: bool,
    pub whole_words_only: bool,
    pub backwards: bool,
}

impl SearchOptions {
    pub fn new() -> Self {
        Self {
            case_sensitive: false,
            whole_words_only: false,
            backwards: false,
        }
    }
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Edit action for undo/redo
#[derive(Clone, Debug)]
enum EditAction {
    InsertChar { pos: Point, ch: char },
    DeleteChar { pos: Point, ch: char },
    InsertText { pos: Point, text: String },
    DeleteText { pos: Point, text: String },
    InsertLine { line: usize, text: String },
    DeleteLine { line: usize, text: String },
}

impl EditAction {
    /// Get the inverse action for undo/redo
    fn inverse(&self) -> Self {
        match self {
            EditAction::InsertChar { pos, ch } => EditAction::DeleteChar { pos: *pos, ch: *ch },
            EditAction::DeleteChar { pos, ch } => EditAction::InsertChar { pos: *pos, ch: *ch },
            EditAction::InsertText { pos, text } => EditAction::DeleteText { pos: *pos, text: text.clone() },
            EditAction::DeleteText { pos, text } => EditAction::InsertText { pos: *pos, text: text.clone() },
            EditAction::InsertLine { line, text } => EditAction::DeleteLine { line: *line, text: text.clone() },
            EditAction::DeleteLine { line, text } => EditAction::InsertLine { line: *line, text: text.clone() },
        }
    }
}

/// Editor - Advanced multi-line text editor with undo/redo and find/replace
///
/// Matches Borland: TEditor receives pointers to scrollbars/indicator created by parent window
pub struct Editor {
    bounds: Rect,
    lines: Vec<String>,
    cursor: Point,
    delta: Point,
    selection_start: Option<Point>,
    state: StateFlags,
    v_scrollbar: Option<Rc<RefCell<ScrollBar>>>,
    h_scrollbar: Option<Rc<RefCell<ScrollBar>>>,
    indicator: Option<Rc<RefCell<Indicator>>>,
    read_only: bool,
    modified: bool,
    tab_size: usize,
    undo_stack: Vec<EditAction>,
    redo_stack: Vec<EditAction>,
    insert_mode: bool, // true = insert, false = overwrite
    auto_indent: bool,
    // Search state (matching Borland's TEditor static members)
    last_search: String,
    last_search_options: SearchOptions,
    // File state (matching Borland's TFileEditor)
    filename: Option<String>,
    // Syntax highlighting
    highlighter: Option<Box<dyn SyntaxHighlighter>>,
    owner: Option<*const dyn View>,
    owner_type: super::view::OwnerType,
}

impl Editor {
    /// Create a new editor control
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
            indicator: None,
            read_only: false,
            modified: false,
            tab_size: 4,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            insert_mode: true,
            auto_indent: false,
            last_search: String::new(),
            last_search_options: SearchOptions::new(),
            filename: None,
            highlighter: None,
            owner: None,
            owner_type: super::view::OwnerType::None,
        }
    }

    /// Create with scrollbars and indicator (Borland style)
    /// Matches Borland: TEditor receives pointers to scrollbars/indicator created by parent
    pub fn with_scrollbars(
        bounds: Rect,
        h_scrollbar: Option<Rc<RefCell<ScrollBar>>>,
        v_scrollbar: Option<Rc<RefCell<ScrollBar>>>,
        indicator: Option<Rc<RefCell<Indicator>>>,
    ) -> Self {
        let mut editor = Self::new(bounds);
        editor.h_scrollbar = h_scrollbar;
        editor.v_scrollbar = v_scrollbar;
        editor.indicator = indicator;
        editor.update_scrollbars();
        editor.update_indicator();
        editor
    }

    /// Set read-only mode
    pub fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
    }

    /// Set tab size
    pub fn set_tab_size(&mut self, tab_size: usize) {
        self.tab_size = tab_size.max(1);
    }

    /// Set auto-indent mode
    pub fn set_auto_indent(&mut self, auto_indent: bool) {
        self.auto_indent = auto_indent;
    }

    /// Set syntax highlighter
    pub fn set_highlighter(&mut self, highlighter: Box<dyn SyntaxHighlighter>) {
        self.highlighter = Some(highlighter);
    }

    /// Clear syntax highlighter (use plain text)
    pub fn clear_highlighter(&mut self) {
        self.highlighter = None;
    }

    /// Check if syntax highlighting is enabled
    pub fn has_highlighter(&self) -> bool {
        self.highlighter.is_some()
    }

    /// Toggle insert/overwrite mode
    pub fn toggle_insert_mode(&mut self) {
        self.insert_mode = !self.insert_mode;
        self.update_indicator();
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
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.update_scrollbars();
        self.update_indicator();
    }

    /// Check if text has been modified
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Clear the modified flag
    pub fn clear_modified(&mut self) {
        self.modified = false;
        self.update_indicator();
    }

    /// Get current line count
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Get the maximum line width (length of the longest line)
    pub fn max_line_width(&self) -> usize {
        self.lines.iter().map(|line| line.len()).max().unwrap_or(0)
    }

    /// Check if vertical scrollbar is needed
    pub fn needs_vertical_scrollbar(&self) -> bool {
        let visible_height = self.bounds.height_clamped() as usize;
        self.line_count() > visible_height
    }

    /// Check if horizontal scrollbar is needed
    pub fn needs_horizontal_scrollbar(&self) -> bool {
        let visible_width = self.bounds.width_clamped() as usize;
        self.max_line_width() > visible_width
    }

    /// Load file contents into the editor
    /// Matches Borland's TFileEditor::load()
    pub fn load_file(&mut self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        let path_ref = path.as_ref();
        let content = std::fs::read_to_string(path_ref)?;
        self.set_text(&content);
        self.filename = Some(path_ref.to_string_lossy().to_string());
        self.modified = false;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.update_indicator();
        Ok(())
    }

    /// Save editor contents to the associated filename
    /// Matches Borland's TFileEditor::save()
    pub fn save_file(&mut self) -> std::io::Result<()> {
        if let Some(path) = self.filename.clone() {
            self.save_as(&path)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No filename set - use save_as() first",
            ))
        }
    }

    /// Save editor contents to a specific filename
    /// Matches Borland's TFileEditor::saveAs()
    pub fn save_as(&mut self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        let path_ref = path.as_ref();
        let content = self.get_text();
        std::fs::write(path_ref, content)?;
        self.filename = Some(path_ref.to_string_lossy().to_string());
        self.modified = false;
        self.update_indicator();
        Ok(())
    }

    /// Get the current filename, if any
    pub fn get_filename(&self) -> Option<&str> {
        self.filename.as_deref()
    }

    /// Undo the last action
    pub fn undo(&mut self) {
        if let Some(action) = self.undo_stack.pop() {
            self.apply_action_inverse(&action);
            self.redo_stack.push(action);
        }
    }

    /// Redo the last undone action
    pub fn redo(&mut self) {
        if let Some(action) = self.redo_stack.pop() {
            self.apply_action(&action);
            self.undo_stack.push(action);
        }
    }

    /// Find text in the editor with options
    /// Matches Borland's TEditor::search() (teditor.cc:917-949)
    pub fn find(&mut self, text: &str, options: SearchOptions) -> Option<Point> {
        if text.is_empty() {
            return None;
        }

        // Save search parameters for find-next
        self.last_search = text.to_string();
        self.last_search_options = options;

        self.find_from_cursor(text, options)
    }

    /// Find next occurrence of last search
    /// Matches Borland's cmSearchAgain command
    pub fn find_next(&mut self) -> Option<Point> {
        if self.last_search.is_empty() {
            return None;
        }

        // Move cursor forward to find next occurrence
        if self.selection_start.is_some() {
            // If there's a selection, start after it
            self.cursor.x += 1;
            self.selection_start = None;
        }

        self.find_from_cursor(&self.last_search.clone(), self.last_search_options)
    }

    /// Find text starting from current cursor position
    fn find_from_cursor(&mut self, text: &str, options: SearchOptions) -> Option<Point> {
        let search_text = if options.case_sensitive {
            text.to_string()
        } else {
            text.to_lowercase()
        };

        // Helper to check if a character is a word character
        let is_word_char = |ch: char| ch.is_alphanumeric() || ch == '_';

        // Start searching from current cursor position
        let start_line = self.cursor.y as usize;
        let start_col = self.cursor.x as usize;

        // Search from cursor to end of document
        for (line_idx, line) in self.lines.iter().enumerate().skip(start_line) {
            let search_line = if options.case_sensitive {
                line.clone()
            } else {
                line.to_lowercase()
            };

            let col_start = if line_idx == start_line {
                start_col
            } else {
                0
            };

            if col_start < line.len() {
                if let Some(col) = search_line[col_start..].find(&search_text) {
                    let found_col = col_start + col;

                    // Check whole-word constraint (Borland: efWholeWordsOnly)
                    if options.whole_words_only {
                        let before_ok = found_col == 0 || !is_word_char(line.chars().nth(found_col - 1).unwrap_or(' '));
                        let after_idx = found_col + text.len();
                        let after_ok = after_idx >= line.len() || !is_word_char(line.chars().nth(after_idx).unwrap_or(' '));

                        if !before_ok || !after_ok {
                            continue; // Not a whole word match, keep searching
                        }
                    }

                    let pos = Point::new(found_col as i16, line_idx as i16);
                    // Set selection to highlight the found text
                    self.selection_start = Some(pos);
                    self.cursor = Point::new((found_col + text.chars().count()) as i16, line_idx as i16);
                    self.make_cursor_visible();
                    return Some(pos);
                }
            }
        }

        // Wrap around: search from beginning to cursor (Borland wraps by default)
        for (line_idx, line) in self.lines.iter().enumerate().take(start_line + 1) {
            let search_line = if options.case_sensitive {
                line.clone()
            } else {
                line.to_lowercase()
            };

            let col_end = if line_idx == start_line {
                start_col
            } else {
                line.len()
            };

            if let Some(col) = search_line[..col_end].find(&search_text) {
                // Check whole-word constraint
                if options.whole_words_only {
                    let before_ok = col == 0 || !is_word_char(line.chars().nth(col - 1).unwrap_or(' '));
                    let after_idx = col + text.len();
                    let after_ok = after_idx >= line.len() || !is_word_char(line.chars().nth(after_idx).unwrap_or(' '));

                    if !before_ok || !after_ok {
                        continue;
                    }
                }

                let pos = Point::new(col as i16, line_idx as i16);
                self.selection_start = Some(pos);
                self.cursor = Point::new((col + text.chars().count()) as i16, line_idx as i16);
                self.make_cursor_visible();
                return Some(pos);
            }
        }

        None
    }

    /// Replace current selection with new text
    /// Returns true if replacement was made
    pub fn replace_selection(&mut self, replace_text: &str) -> bool {
        if self.selection_start.is_some() {
            self.delete_selection();
            self.insert_text(replace_text);
            true
        } else {
            false
        }
    }

    /// Replace next occurrence of find_text with replace_text
    /// Matches Borland's TEditor::doSearchReplace() with efDoReplace
    pub fn replace_next(&mut self, find_text: &str, replace_text: &str, options: SearchOptions) -> bool {
        if let Some(_pos) = self.find(find_text, options) {
            // find() already set selection, now replace it
            self.delete_selection();
            self.insert_text(replace_text);
            true
        } else {
            false
        }
    }

    /// Replace all occurrences of find_text with replace_text
    /// Matches Borland's TEditor::doSearchReplace() with efReplaceAll
    pub fn replace_all(&mut self, find_text: &str, replace_text: &str, options: SearchOptions) -> usize {
        let mut count = 0;

        // Start from beginning of document
        self.cursor = Point::zero();
        self.selection_start = None;

        // Save search parameters
        self.last_search = find_text.to_string();
        self.last_search_options = options;

        // Keep replacing until no more matches
        loop {
            if let Some(_pos) = self.find_from_cursor(find_text, options) {
                self.delete_selection();
                self.insert_text(replace_text);
                count += 1;

                // Move cursor forward to continue searching
                // (insert_text already moved cursor, but we need to position for next search)
            } else {
                break;
            }
        }

        count
    }

    // Private helper methods

    fn get_content_area(&self) -> Rect {
        // In the Borland-style architecture, scrollbars are siblings (not children)
        // So the editor's bounds already exclude scrollbar space - just return full bounds
        self.bounds
    }

    /// Convert mouse position to cursor position (line, column)
    /// Matches Borland: TEditor::getMousePtr() (teditor.cc:426-433)
    fn mouse_pos_to_cursor(&self, mouse_pos: Point) -> Point {
        let content_area = self.get_content_area();

        // Convert absolute mouse position to relative position within editor
        let mut relative_x = mouse_pos.x - content_area.a.x;
        let mut relative_y = mouse_pos.y - content_area.a.y;

        // Clamp to content area (matching Borland's max(0, min(mouse.x, size.x - 1)))
        relative_x = relative_x.max(0).min(content_area.width() - 1);
        relative_y = relative_y.max(0).min(content_area.height() - 1);

        // Add scroll offset to get document position
        let doc_y = (relative_y + self.delta.y) as usize;
        let doc_x = (relative_x + self.delta.x) as usize;

        // Clamp Y to valid line range
        let line_idx = doc_y.min(self.lines.len().saturating_sub(1));

        // Clamp X to line length (allow position at end of line for cursor placement)
        let line_char_len = self.lines[line_idx].chars().count();
        let col = doc_x.min(line_char_len);

        Point::new(col as i16, line_idx as i16)
    }

    /// Set cursor position and handle selection based on mode
    /// Matches Borland: TEditor::setCurPtr() (teditor.cc:986-1014)
    fn set_cursor_with_selection(&mut self, pos: Point, extend_selection: bool) {
        if !extend_selection {
            // Simple click - clear selection and move cursor
            self.selection_start = None;
            self.cursor = pos;
        } else {
            // Drag or shift-click - extend selection
            if self.selection_start.is_none() {
                // Start new selection from current cursor
                self.selection_start = Some(self.cursor);
            }
            // Move cursor to new position (selection_start stays anchored)
            self.cursor = pos;
        }

        self.clamp_cursor();
        self.ensure_cursor_visible();
    }

    fn max_line_length(&self) -> i16 {
        self.lines
            .iter()
            .map(|line| line.chars().count() as i16)
            .max()
            .unwrap_or(0)
    }

    fn update_scrollbars(&mut self) {
        let content_area = self.get_content_area();
        let max_x = self.max_line_length();
        let max_y = self.lines.len() as i16;

        if let Some(ref h_bar) = self.h_scrollbar {
            h_bar.borrow_mut().set_params(
                self.delta.x as i32,
                0,
                max_x.saturating_sub(content_area.width()) as i32,
                content_area.width() as i32,
                1,
            );
        }

        if let Some(ref v_bar) = self.v_scrollbar {
            v_bar.borrow_mut().set_params(
                self.delta.y as i32,
                0,
                max_y.saturating_sub(content_area.height()) as i32,
                content_area.height() as i32,
                1,
            );
        }
    }

    /// Sync editor's delta (scroll position) from scrollbar values
    /// Called after scrollbar events to update editor view
    pub fn sync_from_scrollbars(&mut self) {
        if let Some(ref h_bar) = self.h_scrollbar {
            self.delta.x = h_bar.borrow().get_value() as i16;
        }

        if let Some(ref v_bar) = self.v_scrollbar {
            self.delta.y = v_bar.borrow().get_value() as i16;
        }
    }

    fn update_indicator(&mut self) {
        if let Some(ref indicator) = self.indicator {
            indicator.borrow_mut().set_value(
                Point::new(self.cursor.x + 1, self.cursor.y + 1),
                self.modified,
            );
        }
    }

    fn make_cursor_visible(&mut self) {
        self.ensure_cursor_visible();
    }

    fn ensure_cursor_visible(&mut self) {
        let content_area = self.get_content_area();
        let width = content_area.width();
        let height = content_area.height();

        if self.cursor.y < self.delta.y {
            self.delta.y = self.cursor.y;
        } else if self.cursor.y >= self.delta.y + height {
            self.delta.y = self.cursor.y - height + 1;
        }

        if self.cursor.x < self.delta.x {
            self.delta.x = self.cursor.x;
        } else if self.cursor.x >= self.delta.x + width {
            self.delta.x = self.cursor.x - width + 1;
        }

        self.update_scrollbars();
        self.update_indicator();
    }

    fn clamp_cursor(&mut self) {
        if self.cursor.y < 0 {
            self.cursor.y = 0;
        }
        if self.cursor.y >= self.lines.len() as i16 {
            self.cursor.y = (self.lines.len() - 1) as i16;
        }

        let line_char_len = self.lines[self.cursor.y as usize].chars().count() as i16;
        if self.cursor.x > line_char_len {
            self.cursor.x = line_char_len;
        }
        if self.cursor.x < 0 {
            self.cursor.x = 0;
        }
    }

    /// Convert character index to byte index for a given line
    /// This is necessary because Rust strings are UTF-8 and String::remove/insert expect byte indices
    fn char_to_byte_idx(&self, line_idx: usize, char_idx: usize) -> usize {
        self.lines[line_idx]
            .char_indices()
            .nth(char_idx)
            .map(|(byte_idx, _)| byte_idx)
            .unwrap_or_else(|| self.lines[line_idx].len())
    }

    fn push_undo(&mut self, action: EditAction) {
        self.undo_stack.push(action);
        if self.undo_stack.len() > MAX_UNDO_HISTORY {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
        self.modified = true;
        self.update_indicator();
    }

    fn apply_action(&mut self, action: &EditAction) {
        match action {
            EditAction::InsertChar { pos, ch } => {
                self.cursor = *pos;
                let line_idx = pos.y as usize;
                let col = pos.x as usize;
                let byte_idx = self.char_to_byte_idx(line_idx, col);
                self.lines[line_idx].insert(byte_idx, *ch);
                self.cursor.x += 1;
            }
            EditAction::DeleteChar { pos, .. } => {
                self.cursor = *pos;
                let line_idx = pos.y as usize;
                let col = pos.x as usize;
                let line_char_len = self.lines[line_idx].chars().count();
                if col < line_char_len {
                    let byte_idx = self.char_to_byte_idx(line_idx, col);
                    self.lines[line_idx].remove(byte_idx);
                }
            }
            EditAction::InsertText { pos, text } => {
                self.cursor = *pos;
                self.insert_text_internal(text);
            }
            EditAction::DeleteText { pos, text } => {
                self.cursor = *pos;
                self.selection_start = Some(*pos);
                self.cursor.x += text.chars().count() as i16;
                self.delete_selection_internal();
            }
            _ => {}
        }
        self.ensure_cursor_visible();
    }

    fn apply_action_inverse(&mut self, action: &EditAction) {
        let inverse = action.inverse();
        self.apply_action(&inverse);
    }

    fn insert_char(&mut self, ch: char) {
        if self.read_only {
            return;
        }

        let line_idx = self.cursor.y as usize;
        let col = self.cursor.x as usize;

        if self.insert_mode {
            let action = EditAction::InsertChar { pos: self.cursor, ch };
            let byte_idx = self.char_to_byte_idx(line_idx, col);
            self.lines[line_idx].insert(byte_idx, ch);
            self.cursor.x += 1;
            self.push_undo(action);
        } else {
            // Overwrite mode
            let line_char_len = self.lines[line_idx].chars().count();
            if col < line_char_len {
                let old_ch = self.lines[line_idx].chars().nth(col).unwrap();
                let action = EditAction::DeleteChar { pos: self.cursor, ch: old_ch };
                self.push_undo(action);
                let byte_idx = self.char_to_byte_idx(line_idx, col);
                self.lines[line_idx].remove(byte_idx);
            }
            let action = EditAction::InsertChar { pos: self.cursor, ch };
            let byte_idx = self.char_to_byte_idx(line_idx, col);
            self.lines[line_idx].insert(byte_idx, ch);
            self.cursor.x += 1;
            self.push_undo(action);
        }

        self.selection_start = None;
        self.ensure_cursor_visible();
    }

    fn insert_newline(&mut self) {
        if self.read_only {
            return;
        }

        let line_idx = self.cursor.y as usize;
        let col_char = self.cursor.x as usize;
        let col_byte = self.char_to_byte_idx(line_idx, col_char);

        let current_line = &self.lines[line_idx];
        let before = current_line[..col_byte].to_string();
        let after = current_line[col_byte..].to_string();

        // Auto-indent: calculate leading whitespace
        let indent = if self.auto_indent {
            current_line.chars().take_while(|&c| c == ' ' || c == '\t').collect::<String>()
        } else {
            String::new()
        };

        self.lines[line_idx] = before;
        self.lines.insert(line_idx + 1, indent.clone() + &after);

        self.cursor.y += 1;
        self.cursor.x = indent.chars().count() as i16;
        self.modified = true;
        self.selection_start = None;
        self.ensure_cursor_visible();
        self.update_indicator();
    }

    fn delete_char(&mut self) {
        if self.read_only {
            return;
        }

        let line_idx = self.cursor.y as usize;
        if line_idx >= self.lines.len() {
            return; // Safety check
        }

        let col = self.cursor.x as usize;
        let line_char_len = self.lines[line_idx].chars().count();

        if col < line_char_len {
            let ch = self.lines[line_idx].chars().nth(col).unwrap();
            let action = EditAction::DeleteChar { pos: self.cursor, ch };
            let byte_idx = self.char_to_byte_idx(line_idx, col);
            self.lines[line_idx].remove(byte_idx);
            self.push_undo(action);
        } else if line_idx + 1 < self.lines.len() {
            let next_line = self.lines.remove(line_idx + 1);
            self.lines[line_idx].push_str(&next_line);
            self.modified = true;
        }

        self.selection_start = None;
        self.ensure_cursor_visible();
    }

    fn backspace(&mut self) {
        if self.read_only {
            return;
        }

        let line_idx = self.cursor.y as usize;
        if line_idx >= self.lines.len() {
            return; // Safety check
        }

        let col = self.cursor.x as usize;

        if col > 0 {
            let ch = self.lines[line_idx].chars().nth(col - 1).unwrap();
            self.cursor.x -= 1;
            let action = EditAction::DeleteChar { pos: self.cursor, ch };
            let byte_idx = self.char_to_byte_idx(line_idx, col - 1);
            self.lines[line_idx].remove(byte_idx);
            self.push_undo(action);
        } else if line_idx > 0 {
            let current_line = self.lines.remove(line_idx);
            self.cursor.y -= 1;
            let prev_line_char_len = self.lines[line_idx - 1].chars().count();
            self.lines[line_idx - 1].push_str(&current_line);
            self.cursor.x = prev_line_char_len as i16;
            self.modified = true;
        }

        self.selection_start = None;
        self.ensure_cursor_visible();
    }

    fn insert_tab(&mut self) {
        if self.read_only {
            return;
        }

        for _ in 0..self.tab_size {
            self.insert_char(' ');
        }
    }

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

    /// Move cursor left (previous character), wrapping to previous line if at start
    fn move_cursor_left(&mut self, extend_selection: bool) {
        if !extend_selection {
            self.selection_start = None;
        } else if self.selection_start.is_none() {
            self.selection_start = Some(self.cursor);
        }

        if self.cursor.x > 0 {
            // Not at start of line - move left within current line
            self.cursor.x -= 1;
        } else if self.cursor.y > 0 {
            // At start of line - wrap to end of previous line
            self.cursor.y -= 1;
            let line_char_len = self.lines[self.cursor.y as usize].chars().count() as i16;
            self.cursor.x = line_char_len;
        }
        // else: at position (0,0) - can't move further left

        self.ensure_cursor_visible();
    }

    /// Move cursor right (following character), wrapping to next line if at end
    fn move_cursor_right(&mut self, extend_selection: bool) {
        if !extend_selection {
            self.selection_start = None;
        } else if self.selection_start.is_none() {
            self.selection_start = Some(self.cursor);
        }

        let line_char_len = self.lines[self.cursor.y as usize].chars().count() as i16;

        if self.cursor.x < line_char_len {
            // Not at end of line - move right within current line
            self.cursor.x += 1;
        } else if self.cursor.y < (self.lines.len() - 1) as i16 {
            // At end of line - wrap to start of following line
            self.cursor.y += 1;
            self.cursor.x = 0;
        }
        // else: at end of last line - can't move further right

        self.ensure_cursor_visible();
    }

    fn has_selection(&self) -> bool {
        self.selection_start.is_some()
    }

    /// Check if a position (line, column) is within the current selection
    fn is_position_selected(&self, line: i16, col: i16) -> bool {
        if let Some(start) = self.selection_start {
            let end = self.cursor;

            // Normalize selection bounds (start should be before end)
            let (start, end) = if start.y < end.y || (start.y == end.y && start.x < end.x) {
                (start, end)
            } else {
                (end, start)
            };

            // Check if position is within selection
            if line < start.y || line > end.y {
                return false;
            }

            if line == start.y && line == end.y {
                // Single line selection
                return col >= start.x && col < end.x;
            } else if line == start.y {
                // First line of multi-line selection
                return col >= start.x;
            } else if line == end.y {
                // Last line of multi-line selection
                return col < end.x;
            } else {
                // Middle line of multi-line selection
                return true;
            }
        }
        false
    }

    fn get_selection(&self) -> Option<String> {
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

            let line_idx = y as usize;
            let line = &self.lines[line_idx];
            let line_char_len = line.chars().count();

            if y == start.y && y == end.y {
                let s_char = start.x.max(0) as usize;
                let e_char = (end.x as usize).min(line_char_len);
                if s_char < e_char {
                    let s_byte = self.char_to_byte_idx(line_idx, s_char);
                    let e_byte = self.char_to_byte_idx(line_idx, e_char);
                    result.push_str(&line[s_byte..e_byte]);
                }
            } else if y == start.y {
                let s_char = start.x.max(0) as usize;
                let s_byte = self.char_to_byte_idx(line_idx, s_char);
                result.push_str(&line[s_byte..]);
                result.push('\n');
            } else if y == end.y {
                let e_char = (end.x as usize).min(line_char_len);
                let e_byte = self.char_to_byte_idx(line_idx, e_char);
                result.push_str(&line[..e_byte]);
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }

        Some(result)
    }

    fn select_all(&mut self) {
        self.selection_start = Some(Point::zero());
        self.cursor = Point::new(
            self.lines.last().map(|l| l.chars().count()).unwrap_or(0) as i16,
            (self.lines.len() - 1) as i16,
        );
        self.ensure_cursor_visible();
    }

    fn delete_selection_internal(&mut self) {
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
            let start_col_char = start.x.max(0) as usize;
            let end_col_char = (end.x as usize).min(self.lines[start_line].chars().count());
            if start_col_char < end_col_char {
                let start_col_byte = self.char_to_byte_idx(start_line, start_col_char);
                let end_col_byte = self.char_to_byte_idx(start_line, end_col_char);
                self.lines[start_line].drain(start_col_byte..end_col_byte);
            }
        } else {
            let start_col_char = start.x.max(0) as usize;
            let end_col_char = (end.x as usize).min(self.lines[end_line].chars().count());

            let start_col_byte = self.char_to_byte_idx(start_line, start_col_char);
            let end_col_byte = self.char_to_byte_idx(end_line, end_col_char);

            let before = self.lines[start_line][..start_col_byte].to_string();
            let after = self.lines[end_line][end_col_byte..].to_string();

            self.lines.drain(start_line..=end_line);
            self.lines.insert(start_line, before + &after);
        }

        self.cursor = start;
        self.selection_start = None;
        self.modified = true;
        self.ensure_cursor_visible();
    }

    fn delete_selection(&mut self) {
        if !self.has_selection() {
            return;
        }

        if let Some(text) = self.get_selection() {
            let action = EditAction::DeleteText { pos: self.selection_start.unwrap(), text };
            self.delete_selection_internal();
            self.push_undo(action);
        }
    }

    /// Copy selection to clipboard
    /// Matches Borland: TEditor::clipCopy()
    pub fn clip_copy(&mut self) -> bool {
        if let Some(text) = self.get_selection() {
            clipboard::set_clipboard(&text);
            true
        } else {
            false
        }
    }

    /// Cut selection to clipboard (copy + delete)
    /// Matches Borland: TEditor::clipCut()
    pub fn clip_cut(&mut self) -> bool {
        if self.read_only || !self.has_selection() {
            return false;
        }

        if let Some(text) = self.get_selection() {
            clipboard::set_clipboard(&text);
            self.delete_selection();
            true
        } else {
            false
        }
    }

    /// Paste from clipboard
    /// Matches Borland: TEditor::clipPaste()
    pub fn clip_paste(&mut self) -> bool {
        if self.read_only {
            return false;
        }

        let text = clipboard::get_clipboard();
        if !text.is_empty() {
            // Delete selection first if there is one
            if self.has_selection() {
                self.delete_selection();
            }
            self.insert_text(&text);
            true
        } else {
            false
        }
    }

    fn insert_text_internal(&mut self, text: &str) {
        if self.read_only {
            return;
        }

        let lines_to_insert: Vec<&str> = text.lines().collect();
        if lines_to_insert.is_empty() {
            return;
        }

        let line_idx = self.cursor.y as usize;
        let col_char = self.cursor.x as usize;
        let col_byte = self.char_to_byte_idx(line_idx, col_char);

        if lines_to_insert.len() == 1 {
            self.lines[line_idx].insert_str(col_byte, lines_to_insert[0]);
            self.cursor.x += lines_to_insert[0].chars().count() as i16;
        } else {
            let current_line = &self.lines[line_idx];
            let before = current_line[..col_byte].to_string();
            let after = current_line[col_byte..].to_string();

            self.lines[line_idx] = before + lines_to_insert[0];

            for (i, line) in lines_to_insert.iter().enumerate().skip(1) {
                self.lines.insert(line_idx + i, line.to_string());
            }

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

    fn insert_text(&mut self, text: &str) {
        if self.has_selection() {
            self.delete_selection();
        }

        let action = EditAction::InsertText { pos: self.cursor, text: text.to_string() };
        self.insert_text_internal(text);
        self.push_undo(action);
    }
}

impl View for Editor {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
        // Note: Scrollbars and indicator are now children of the Window, not the Editor
        // The Window's interior Group automatically handles their positioning
        // We only need to update our internal state
        self.update_scrollbars();
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        use crate::core::palette::{EDITOR_NORMAL, EDITOR_SELECTED, EDITOR_CURSOR};

        let content_area = self.get_content_area();
        let width = content_area.width_clamped() as usize;
        let height = content_area.height_clamped() as usize;

        let default_color = self.map_color(EDITOR_NORMAL);
        let selected_color = self.map_color(EDITOR_SELECTED);
        let cursor_color = self.map_color(EDITOR_CURSOR);

        for y in 0..height {
            let line_idx = (self.delta.y + y as i16) as usize;
            let mut buf = DrawBuffer::new(width);

            buf.move_char(0, ' ', default_color, width);

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

                    // Apply syntax highlighting if available
                    if let Some(ref highlighter) = self.highlighter {
                        let tokens = highlighter.highlight_line(line, line_idx);

                        // Draw each token with its color
                        let mut current_col = 0;
                        for token in tokens {
                            // Skip tokens before visible area
                            if token.end <= start_col {
                                continue;
                            }
                            // Stop at tokens past visible area
                            if token.start >= end_col_char {
                                break;
                            }

                            // Calculate visible portion of this token
                            let token_start = token.start.max(start_col) - start_col;
                            let token_end = token.end.min(end_col_char) - start_col;

                            // Fill gap before this token with default color
                            if current_col < token_start {
                                // Already filled with spaces above
                                // (no action needed, spaces already written)
                            }

                            // Get text for this token
                            let token_text: String = line
                                .chars()
                                .skip(start_col + token_start)
                                .take(token_end - token_start)
                                .collect();

                            // Draw token with its color
                            if !token_text.is_empty() {
                                buf.move_str(
                                    token_start,
                                    &token_text,
                                    token.token_type.default_color(),
                                );
                            }

                            current_col = token_end;
                        }
                    } else {
                        // No highlighting - use default color
                        buf.move_str(0, &visible_text, default_color);
                    }
                }
            }

            // Apply selection highlighting
            // Check each character position in this line to see if it's selected
            if self.has_selection() {
                let line_y = (self.delta.y + y as i16) as i16;
                let start_col = self.delta.x;

                for x in 0..width {
                    let col = (start_col + x as i16) as i16;
                    if self.is_position_selected(line_y, col) {
                        // Highlight this character as selected
                        if x < buf.data.len() {
                            buf.data[x].attr = selected_color;
                        }
                    }
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

        // Note: Scrollbars and indicator are now drawn by the Window's interior Group
        // They are separate child views, not owned by the Editor
    }

    fn handle_event(&mut self, event: &mut Event) {
        // Handle mouse events (matching Borland TEditor::handleEvent - teditor.cc:454-493)
        if event.what == EventType::MouseDown {
            // Only handle mouse events if focused
            if !self.is_focused() {
                return;
            }

            let mouse_pos = event.mouse.pos;
            let content_area = self.get_content_area();

            // Check if click is within editor bounds
            if !content_area.contains(mouse_pos) {
                return;
            }

            // Convert mouse position to cursor position
            let cursor_pos = self.mouse_pos_to_cursor(mouse_pos);

            // Check if this is the start of a drag operation
            // Matches Borland: do { ... } while( mouseEvent(event, evMouseMove + evMouseAuto) )
            let extend_selection = false;

            // First click sets cursor position
            self.set_cursor_with_selection(cursor_pos, extend_selection);

            // Now track mouse movement for drag selection
            // We need to consume the MouseDown event and wait for MouseMove/MouseUp events
            event.clear();

            // Note: In the Borland implementation, there's a mouseEvent() helper that
            // waits for the next mouse event in a loop. In our architecture, we handle
            // this differently - we'll set extend_selection flag after the first click
            // and subsequent MouseMove events will extend the selection.
            //
            // However, to truly match Borland's behavior, we would need to implement
            // a tracking loop here that actively polls for mouse events. This requires
            // access to the application's event queue, which we don't have in handle_event.
            //
            // For now, we implement a simplified version where:
            // 1. MouseDown sets cursor position
            // 2. Subsequent MouseMove events (if button still held) extend selection
            //
            // This is a limitation of our current event architecture compared to Borland's.

            return;
        }

        // Handle mouse move for drag selection
        if event.what == EventType::MouseMove {
            // Only track drags if focused and left button is held
            if !self.is_focused() || (event.mouse.buttons & MB_LEFT_BUTTON == 0) {
                return;
            }

            let mouse_pos = event.mouse.pos;
            let content_area = self.get_content_area();

            // Auto-scroll if mouse is outside editor bounds
            // Matches Borland: teditor.cc:475-487
            let mut scroll_delta = self.delta;
            let mut needs_scroll = false;

            if mouse_pos.x < content_area.a.x {
                scroll_delta.x = scroll_delta.x.saturating_sub(1);
                needs_scroll = true;
            } else if mouse_pos.x >= content_area.b.x {
                scroll_delta.x += 1;
                needs_scroll = true;
            }

            if mouse_pos.y < content_area.a.y {
                scroll_delta.y = scroll_delta.y.saturating_sub(1);
                needs_scroll = true;
            } else if mouse_pos.y >= content_area.b.y {
                scroll_delta.y += 1;
                needs_scroll = true;
            }

            if needs_scroll {
                // Clamp scroll position
                let max_x = self.max_line_length().saturating_sub(content_area.width());
                let max_y = (self.lines.len() as i16).saturating_sub(content_area.height());
                scroll_delta.x = scroll_delta.x.max(0).min(max_x);
                scroll_delta.y = scroll_delta.y.max(0).min(max_y);

                self.delta = scroll_delta;
                self.update_scrollbars();
            }

            // Convert mouse position to cursor position and extend selection
            let cursor_pos = self.mouse_pos_to_cursor(mouse_pos);
            self.set_cursor_with_selection(cursor_pos, true);

            event.clear();
            return;
        }

        if event.what == EventType::Keyboard {
            // Only handle keyboard events if focused
            if !self.is_focused() {
                return;
            }

            // Check if Shift key is pressed for text selection
            use crossterm::event::KeyModifiers;
            let shift_pressed = event.key_modifiers.contains(KeyModifiers::SHIFT);

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
                    // Move left (previous character), wrapping to previous line if at start
                    self.move_cursor_left(shift_pressed);
                    event.clear();
                }
                KB_RIGHT => {
                    // Move right (following character), wrapping to following line if at end
                    self.move_cursor_right(shift_pressed);
                    event.clear();
                }
                KB_HOME => {
                    // Save old position if starting selection
                    if shift_pressed && self.selection_start.is_none() {
                        self.selection_start = Some(self.cursor);
                    } else if !shift_pressed {
                        self.selection_start = None;
                    }

                    self.cursor.x = 0;
                    self.ensure_cursor_visible();
                    event.clear();
                }
                KB_END => {
                    // Save old position if starting selection
                    if shift_pressed && self.selection_start.is_none() {
                        self.selection_start = Some(self.cursor);
                    } else if !shift_pressed {
                        self.selection_start = None;
                    }

                    let line_idx = self.cursor.y as usize;
                    if line_idx < self.lines.len() {
                        let line_char_len = self.lines[line_idx].chars().count() as i16;
                        self.cursor.x = line_char_len;
                    }
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
                    self.clip_copy();
                    event.clear();
                }
                KB_CTRL_X => {
                    self.clip_cut();
                    event.clear();
                }
                KB_CTRL_V => {
                    self.clip_paste();
                    event.clear();
                }
                KB_CTRL_Z => {
                    self.undo();
                    event.clear();
                }
                KB_CTRL_Y => {
                    self.redo();
                    event.clear();
                }
                key_code => {
                    // Accept all printable characters including Unicode (è, à, etc.)
                    // Key codes represent Unicode codepoints, so convert directly to char
                    if let Some(ch) = char::from_u32(key_code as u32) {
                        // Only insert if it's a printable character (not control characters)
                        if !ch.is_control() {
                            self.insert_char(ch);
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
            // Calculate cursor position on screen using content area (not bounds)
            // to account for indicator and scrollbars
            let content_area = self.get_content_area();
            let cursor_x = content_area.a.x + (self.cursor.x - self.delta.x);
            let cursor_y = content_area.a.y + (self.cursor.y - self.delta.y);

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
        // Editor uses cpEditor palette for proper color remapping through window hierarchy
        // Matches Borland: cpEditor = [6, 7] for normal and selected text
        Some(Palette::from_slice(palettes::CP_EDITOR))
    }

    fn get_owner_type(&self) -> super::view::OwnerType {
        self.owner_type
    }

    fn set_owner_type(&mut self, owner_type: super::view::OwnerType) {
        self.owner_type = owner_type;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_editor_load_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Line 1").unwrap();
        writeln!(file, "Line 2").unwrap();
        writeln!(file, "Line 3").unwrap();
        file.flush().unwrap();

        let bounds = Rect::new(0, 0, 80, 25);
        let mut editor = Editor::new(bounds);

        editor.load_file(file.path().to_str().unwrap()).unwrap();

        assert_eq!(editor.line_count(), 3);
        assert_eq!(editor.get_text(), "Line 1\nLine 2\nLine 3");
        assert_eq!(editor.get_filename(), file.path().to_str());
        assert!(!editor.is_modified());
    }

    #[test]
    fn test_editor_save_as() {
        let bounds = Rect::new(0, 0, 80, 25);
        let mut editor = Editor::new(bounds);

        editor.set_text("Hello\nWorld");

        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap();

        editor.save_as(path).unwrap();

        let content = std::fs::read_to_string(path).unwrap();
        assert_eq!(content, "Hello\nWorld");
        assert_eq!(editor.get_filename(), Some(path));
        assert!(!editor.is_modified());
    }

    #[test]
    fn test_editor_save_file() {
        let bounds = Rect::new(0, 0, 80, 25);
        let mut editor = Editor::new(bounds);

        // Should fail without filename
        assert!(editor.save_file().is_err());

        // Set filename via save_as
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap();
        editor.set_text("Test content");
        editor.save_as(path).unwrap();
        assert!(!editor.is_modified());

        // Modify by setting new text
        editor.set_text("Modified content");
        // Note: set_text() clears modified flag, so we need to save and verify content changed

        editor.save_file().unwrap();

        let content = std::fs::read_to_string(path).unwrap();
        assert_eq!(content, "Modified content");
        assert!(!editor.is_modified());
    }

    #[test]
    fn test_editor_modified_flag() {
        let bounds = Rect::new(0, 0, 80, 25);
        let mut editor = Editor::new(bounds);

        assert!(!editor.is_modified());

        editor.set_text("Some text");
        assert!(!editor.is_modified()); // set_text clears modified flag

        // Simulate typing (would set modified via push_undo)
        let file = NamedTempFile::new().unwrap();
        editor.save_as(file.path().to_str().unwrap()).unwrap();
        assert!(!editor.is_modified());
    }

    #[test]
    fn test_editor_load_empty_file() {
        let file = NamedTempFile::new().unwrap();
        // Don't write anything - file is empty

        let bounds = Rect::new(0, 0, 80, 25);
        let mut editor = Editor::new(bounds);

        editor.load_file(file.path().to_str().unwrap()).unwrap();

        assert_eq!(editor.line_count(), 1); // Editor always has at least one line
        assert_eq!(editor.get_text(), "");
        assert!(!editor.is_modified());
    }
}
