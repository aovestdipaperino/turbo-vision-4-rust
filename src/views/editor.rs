// (C) 2025 - Enzo Lombardi

//! EditorWindow view - advanced multi-line text editor with syntax highlighting support.

use super::indicator::Indicator;
use super::scrollbar::ScrollBar;
use super::syntax::SyntaxHighlighter;
use super::view::{View, write_line_to_terminal};
use crate::core::clipboard;
use crate::core::draw::DrawBuffer;
use crate::core::event::{
    Event, EventType, KB_BACKSPACE, KB_DEL, KB_DOWN, KB_END, KB_ENTER, KB_HOME, KB_LEFT, KB_PGDN,
    KB_PGUP, KB_RIGHT, KB_TAB, KB_UP, MB_LEFT_BUTTON,
};
use crate::core::geometry::{Point, Rect};
use crate::core::state::StateFlags;
use crate::terminal::Terminal;
use std::cell::RefCell;
use std::cmp::min;
use std::rc::Rc;

// Control key codes
const KB_CTRL_A: u16 = 0x0001; // Ctrl+A - Select All
const KB_CTRL_C: u16 = 0x0003; // Ctrl+C - Copy
#[expect(dead_code, reason = "Reserved for future find/replace functionality")]
const KB_CTRL_F: u16 = 0x0006; // Ctrl+F - Find
#[expect(dead_code, reason = "Reserved for future find/replace functionality")]
const KB_CTRL_H: u16 = 0x0008; // Ctrl+H - Replace
const KB_CTRL_V: u16 = 0x0016; // Ctrl+V - Paste
const KB_CTRL_X: u16 = 0x0018; // Ctrl+X - Cut
const KB_CTRL_Y: u16 = 0x0019; // Ctrl+Y - Redo
const KB_CTRL_Z: u16 = 0x001A; // Ctrl+Z - Undo

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
    InsertChar {
        pos: Point,
        ch: char,
    },
    DeleteChar {
        pos: Point,
        ch: char,
    },
    InsertText {
        pos: Point,
        text: String,
    },
    DeleteText {
        pos: Point,
        text: String,
    },
    InsertLine {
        line: usize,
        text: String,
    },
    DeleteLine {
        line: usize,
        text: String,
    },
    /// A sequence of edits that must undo / redo as a single step. Used for
    /// composite operations like paste-over-selection (delete + insert) and
    /// find/replace, so one Ctrl+Z reverses the whole thing.
    Compound(Vec<EditAction>),
}

impl EditAction {
    /// Get the inverse action for undo/redo
    fn inverse(&self) -> Self {
        match self {
            EditAction::InsertChar { pos, ch } => EditAction::DeleteChar { pos: *pos, ch: *ch },
            EditAction::DeleteChar { pos, ch } => EditAction::InsertChar { pos: *pos, ch: *ch },
            EditAction::InsertText { pos, text } => EditAction::DeleteText {
                pos: *pos,
                text: text.clone(),
            },
            EditAction::DeleteText { pos, text } => EditAction::InsertText {
                pos: *pos,
                text: text.clone(),
            },
            EditAction::InsertLine { line, text } => EditAction::DeleteLine {
                line: *line,
                text: text.clone(),
            },
            EditAction::DeleteLine { line, text } => EditAction::InsertLine {
                line: *line,
                text: text.clone(),
            },
            EditAction::Compound(actions) => {
                // Reverse order + invert each: undoing a forward
                // [delete, insert] sequence means inserting the deleted
                // text first (last-out, first-in) then deleting the
                // inserted text.
                EditAction::Compound(actions.iter().rev().map(|a| a.inverse()).collect())
            }
        }
    }
}

/// EditorWindow - Advanced multi-line text editor with undo/redo and find/replace
///
/// Matches Borland: TEditor receives pointers to scrollbars/indicator created by parent window
/// How an active selection between `selection_start` and `cursor` is interpreted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionMode {
    /// Continuous character range (the classic Borland behavior).
    Stream,
    /// Rectangular column block: the column band `[min_x, max_x)` on every row
    /// in `[min_y, max_y]`. Activated with the Alt/Option modifier.
    Block,
}

pub struct EditorWindow {
    bounds: Rect,
    lines: Vec<String>,
    cursor: Point,
    delta: Point,
    selection_start: Option<Point>,
    /// Interpretation of the current selection. Fixed when a selection starts.
    selection_mode: SelectionMode,
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
    // Line-ending fidelity: remembered on load, reapplied on save
    trailing_newline: bool,
    crlf_line_endings: bool,
    // When set, saving renames the existing file to "<name>.bak" first
    // (Borland: efBackupFiles + backupExt)
    backup_files: bool,
    // Syntax highlighting
    highlighter: Option<Box<dyn SyntaxHighlighter>>,
    palette_chain: Option<crate::core::palette_chain::PaletteChainNode>,
    /// Grow flags: the editor fills its parent's interior, so its bottom-right
    /// edge follows a resize while the top-left stays pinned.
    grow_mode: crate::core::state::GrowFlags,
}

impl EditorWindow {
    /// Create a new editor control
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            lines: vec![String::new()],
            cursor: Point::zero(),
            delta: Point::zero(),
            selection_start: None,
            selection_mode: SelectionMode::Stream,
            grow_mode: crate::core::state::GF_GROW_HI_X | crate::core::state::GF_GROW_HI_Y,
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
            trailing_newline: true,
            crlf_line_endings: false,
            backup_files: false,
            highlighter: None,
            palette_chain: None,
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
        // .lines() strips \r\n and \n; line-ending style is tracked
        // separately (trailing_newline / crlf_line_endings) for saving
        self.lines = text.lines().map(|s| s.to_string()).collect();
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.cursor = Point::zero();
        self.delta = Point::zero();
        self.selection_start = None;
        self.selection_mode = SelectionMode::Stream;
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

    /// Get the current scroll offset (top-left visible position).
    pub fn get_delta(&self) -> Point {
        self.delta
    }

    /// Cursor position in document coordinates (0-based line, 0-based
    /// character column).
    pub fn cursor(&self) -> Point {
        self.cursor
    }

    /// Scroll the editor so that the given 0-based line is visible,
    /// moving the cursor to the beginning of that line.
    pub fn scroll_to_line(&mut self, line: usize) {
        self.cursor.y = line as i16;
        self.cursor.x = 0;
        self.clamp_cursor();
        self.ensure_cursor_visible();
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
        self.trailing_newline = content.is_empty() || content.ends_with('\n');
        self.crlf_line_endings = content.contains("\r\n");
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

        // Optional Borland-style backup: rename the old file to "<name>.bak"
        if self.backup_files && path_ref.exists() {
            let mut backup = path_ref.as_os_str().to_owned();
            backup.push(".bak");
            let _ = std::fs::rename(path_ref, std::path::Path::new(&backup));
        }

        // Reapply the line-ending style remembered at load time
        let newline = if self.crlf_line_endings { "\r\n" } else { "\n" };
        let mut content = self.lines.join(newline);
        if self.trailing_newline && !self.lines.is_empty() {
            content.push_str(newline);
        }
        std::fs::write(path_ref, content)?;
        self.filename = Some(path_ref.to_string_lossy().to_string());
        self.modified = false;
        self.update_indicator();
        Ok(())
    }

    /// Enable or disable Borland-style .bak backups on save.
    pub fn set_backup_files(&mut self, backup: bool) {
        self.backup_files = backup;
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

    /// True when the undo stack has at least one entry.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// True when the redo stack has at least one entry. The redo stack is
    /// cleared on every fresh edit (see [`Self::push_undo`]), so this only
    /// returns true between an undo and the next mutation.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
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

        // The cursor already sits just past the previous match, so searching
        // from it finds the next occurrence without skipping adjacent matches
        if self.selection_start.is_some() {
            self.selection_start = None;
        }

        self.find_from_cursor(&self.last_search.clone(), self.last_search_options)
    }

    /// Find text starting from current cursor position.
    ///
    /// Searches forward to the end of the document without wrapping, matching
    /// Borland's TEditor::search(). All column arithmetic is done in character
    /// space so multibyte lines can't misalign or panic. Case folding is done
    /// per character (first lowercase mapping) to keep columns 1:1.
    fn find_from_cursor(&mut self, text: &str, options: SearchOptions) -> Option<Point> {
        let case_sensitive = options.case_sensitive;
        let fold = move |ch: char| {
            if case_sensitive {
                ch
            } else {
                ch.to_lowercase().next().unwrap_or(ch)
            }
        };
        let needle: Vec<char> = text.chars().map(fold).collect();
        if needle.is_empty() {
            return None;
        }

        let is_word_char = |ch: char| ch.is_alphanumeric() || ch == '_';

        let start_line = self.cursor.y as usize;
        let start_col = self.cursor.x as usize;

        for (line_idx, line) in self.lines.iter().enumerate().skip(start_line) {
            let chars: Vec<char> = line.chars().collect();
            let from = if line_idx == start_line { start_col } else { 0 };

            let mut col = from;
            while col + needle.len() <= chars.len() {
                let matched = chars[col..col + needle.len()]
                    .iter()
                    .map(|&c| fold(c))
                    .eq(needle.iter().copied());

                if matched {
                    // Check whole-word constraint (Borland: efWholeWordsOnly)
                    let after = col + needle.len();
                    let whole_ok = !options.whole_words_only
                        || ((col == 0 || !is_word_char(chars[col - 1]))
                            && (after >= chars.len() || !is_word_char(chars[after])));

                    if whole_ok {
                        let pos = Point::new(col as i16, line_idx as i16);
                        // Set selection to highlight the found text
                        self.selection_start = Some(pos);
                        self.cursor = Point::new(after as i16, line_idx as i16);
                        self.make_cursor_visible();
                        return Some(pos);
                    }
                }
                col += 1;
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
    pub fn replace_next(
        &mut self,
        find_text: &str,
        replace_text: &str,
        options: SearchOptions,
    ) -> bool {
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
    pub fn replace_all(
        &mut self,
        find_text: &str,
        replace_text: &str,
        options: SearchOptions,
    ) -> usize {
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
    /// Fix the selection mode at the moment a selection starts. When extending
    /// and no selection is active yet, the global block-edit mode decides the
    /// shape (see `Application::set_block_edit_mode`). A non-extending move
    /// resets the mode so the next selection picks it up again.
    fn set_selection_mode_for(&mut self, extend: bool) {
        if !extend || self.selection_start.is_none() {
            self.selection_mode = Self::mode_from_global();
        }
    }

    /// The selection shape implied by the global block-edit mode.
    fn mode_from_global() -> SelectionMode {
        if crate::core::state::block_edit_mode() {
            SelectionMode::Block
        } else {
            SelectionMode::Stream
        }
    }

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
        let max_x = self.max_line_length();
        let max_y = self.lines.len() as i16;

        if let Some(ref h_bar) = self.h_scrollbar {
            h_bar
                .borrow_mut()
                .set_params(self.cursor.x as i32, 0, (max_x - 1).max(0) as i32, 1, 1);
            h_bar.borrow_mut().set_total(max_x as i32);
        }

        if let Some(ref v_bar) = self.v_scrollbar {
            v_bar
                .borrow_mut()
                .set_params(self.cursor.y as i32, 0, (max_y - 1).max(0) as i32, 1, 1);
            v_bar.borrow_mut().set_total(max_y as i32);
        }
    }

    /// Sync editor cursor from scrollbar values and ensure it's visible.
    /// Scrollbar value represents cursor position in the document.
    pub fn sync_from_scrollbars(&mut self) {
        if let Some(ref h_bar) = self.h_scrollbar {
            self.cursor.x = h_bar.borrow().get_value() as i16;
        }

        if let Some(ref v_bar) = self.v_scrollbar {
            self.cursor.y = v_bar.borrow().get_value() as i16;
        }

        self.clamp_cursor();
        self.ensure_cursor_visible();
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

        self.delta.x = self.delta.x.max(0);
        self.delta.y = self.delta.y.max(0);

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
                self.selection_start = Some(*pos);
                // Walk the text to find the end position; embedded newlines
                // move the end down and reset the column
                let mut end = *pos;
                for ch in text.chars() {
                    if ch == '\n' {
                        end.y += 1;
                        end.x = 0;
                    } else {
                        end.x += 1;
                    }
                }
                self.cursor = end;
                self.delete_selection_internal();
            }
            EditAction::Compound(actions) => {
                // Replay each step in order. `inverse()` already reversed
                // the order for undo, so this loop is correct in both
                // directions.
                for inner in actions {
                    self.apply_action(inner);
                }
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
            let action = EditAction::InsertChar {
                pos: self.cursor,
                ch,
            };
            let byte_idx = self.char_to_byte_idx(line_idx, col);
            self.lines[line_idx].insert(byte_idx, ch);
            self.cursor.x += 1;
            self.push_undo(action);
        } else {
            // Overwrite mode: record delete + insert as one undo step so a
            // single Ctrl+Z restores the overtyped character
            let mut steps = Vec::with_capacity(2);
            let line_char_len = self.lines[line_idx].chars().count();
            if col < line_char_len {
                let old_ch = self.lines[line_idx].chars().nth(col).unwrap();
                steps.push(EditAction::DeleteChar {
                    pos: self.cursor,
                    ch: old_ch,
                });
                let byte_idx = self.char_to_byte_idx(line_idx, col);
                self.lines[line_idx].remove(byte_idx);
            }
            steps.push(EditAction::InsertChar {
                pos: self.cursor,
                ch,
            });
            let byte_idx = self.char_to_byte_idx(line_idx, col);
            self.lines[line_idx].insert(byte_idx, ch);
            self.cursor.x += 1;
            self.push_undo(EditAction::Compound(steps));
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
            current_line
                .chars()
                .take_while(|&c| c == ' ' || c == '\t')
                .collect::<String>()
        } else {
            String::new()
        };

        let action = EditAction::InsertText {
            pos: self.cursor,
            text: format!("\n{indent}"),
        };

        self.lines[line_idx] = before;
        self.lines.insert(line_idx + 1, indent.clone() + &after);

        self.cursor.y += 1;
        self.cursor.x = indent.chars().count() as i16;
        self.push_undo(action);
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
            let action = EditAction::DeleteChar {
                pos: self.cursor,
                ch,
            };
            let byte_idx = self.char_to_byte_idx(line_idx, col);
            self.lines[line_idx].remove(byte_idx);
            self.push_undo(action);
        } else if line_idx + 1 < self.lines.len() {
            let next_line = self.lines.remove(line_idx + 1);
            self.lines[line_idx].push_str(&next_line);
            self.push_undo(EditAction::DeleteText {
                pos: self.cursor,
                text: "\n".to_string(),
            });
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
            let action = EditAction::DeleteChar {
                pos: self.cursor,
                ch,
            };
            let byte_idx = self.char_to_byte_idx(line_idx, col - 1);
            self.lines[line_idx].remove(byte_idx);
            self.push_undo(action);
        } else if line_idx > 0 {
            let current_line = self.lines.remove(line_idx);
            self.cursor.y -= 1;
            let prev_line_char_len = self.lines[line_idx - 1].chars().count();
            self.lines[line_idx - 1].push_str(&current_line);
            self.cursor.x = prev_line_char_len as i16;
            self.push_undo(EditAction::DeleteText {
                pos: self.cursor,
                text: "\n".to_string(),
            });
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

    /// Position of the previous word boundary (crossing line breaks).
    ///
    /// Matches Borland TEditor::prevWord: skip separators leftwards, then
    /// skip the word the cursor lands in.
    fn word_left_pos(&self) -> Point {
        let is_word = |c: char| c.is_alphanumeric() || c == '_';
        let (mut x, mut y) = (self.cursor.x as i32, self.cursor.y as i32);

        let char_at = |lines: &Vec<String>, y: i32, x: i32| -> Option<char> {
            lines[y as usize].chars().nth(x as usize)
        };

        // Step left once (wrapping to the previous line end)
        let step_left = |x: &mut i32, y: &mut i32| -> bool {
            if *x > 0 {
                *x -= 1;
                true
            } else if *y > 0 {
                *y -= 1;
                *x = self.lines[*y as usize].chars().count() as i32;
                true
            } else {
                false
            }
        };

        // Skip separators (including line breaks)
        loop {
            if !step_left(&mut x, &mut y) {
                return Point::new(x as i16, y as i16);
            }
            if char_at(&self.lines, y, x).is_some_and(is_word) {
                break;
            }
            if x == 0 && y == 0 {
                return Point::new(0, 0);
            }
        }
        // Skip the word itself
        while x > 0 {
            if char_at(&self.lines, y, x - 1).is_some_and(is_word) {
                x -= 1;
            } else {
                break;
            }
        }
        Point::new(x as i16, y as i16)
    }

    /// Position of the next word boundary (crossing line breaks).
    ///
    /// Matches Borland TEditor::nextWord: skip the current word, then skip
    /// separators to the start of the next word.
    fn word_right_pos(&self) -> Point {
        let is_word = |c: char| c.is_alphanumeric() || c == '_';
        let (mut x, mut y) = (self.cursor.x as i32, self.cursor.y as i32);
        let last_line = self.lines.len() as i32 - 1;

        let line_len = |lines: &Vec<String>, y: i32| lines[y as usize].chars().count() as i32;
        let char_at = |lines: &Vec<String>, y: i32, x: i32| -> Option<char> {
            lines[y as usize].chars().nth(x as usize)
        };

        // Skip the word under the cursor
        while x < line_len(&self.lines, y) && char_at(&self.lines, y, x).is_some_and(is_word) {
            x += 1;
        }
        // Skip separators (including line breaks) to the next word start
        loop {
            if x >= line_len(&self.lines, y) {
                if y >= last_line {
                    return Point::new(line_len(&self.lines, y) as i16, y as i16);
                }
                y += 1;
                x = 0;
            } else if char_at(&self.lines, y, x).is_some_and(is_word) {
                return Point::new(x as i16, y as i16);
            } else {
                x += 1;
            }
        }
    }

    /// Move the cursor one word left/right, optionally extending selection.
    fn move_word(&mut self, right: bool, extend_selection: bool) {
        if extend_selection {
            if self.selection_start.is_none() {
                self.selection_start = Some(self.cursor);
            }
        } else {
            self.selection_start = None;
        }
        self.cursor = if right {
            self.word_right_pos()
        } else {
            self.word_left_pos()
        };
        self.ensure_cursor_visible();
    }

    /// Extract the text between two positions (for word-delete undo records).
    fn text_between(&self, from: Point, to: Point) -> String {
        let mut out = String::new();
        let (mut x, mut y) = (from.x as usize, from.y as usize);
        while (y as i16, x as i16) < (to.y, to.x) {
            let line = &self.lines[y];
            if x >= line.chars().count() {
                out.push('\n');
                y += 1;
                x = 0;
            } else {
                out.push(line.chars().nth(x).expect("in range"));
                x += 1;
            }
        }
        out
    }

    /// Delete from the cursor to the previous/next word boundary as one
    /// undoable step (Borland: cmDelWord / Ctrl+Backspace).
    fn delete_word(&mut self, right: bool) {
        if self.read_only {
            return;
        }
        let (from, to) = if right {
            (self.cursor, self.word_right_pos())
        } else {
            (self.word_left_pos(), self.cursor)
        };
        if from == to {
            return;
        }
        let text = self.text_between(from, to);
        self.selection_start = Some(from);
        self.cursor = to;
        self.delete_selection_internal();
        self.cursor = from;
        self.selection_start = None;
        self.push_undo(EditAction::DeleteText { pos: from, text });
        self.ensure_cursor_visible();
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

    pub fn has_selection(&self) -> bool {
        self.selection_start.is_some()
    }

    /// Check if a position (line, column) is within the current selection
    fn is_position_selected(&self, line: i16, col: i16) -> bool {
        if let Some(start) = self.selection_start {
            let end = self.cursor;

            // Block mode: a rectangular column band, independent per axis.
            if self.selection_mode == SelectionMode::Block {
                let (min_y, max_y) = (start.y.min(end.y), start.y.max(end.y));
                let (min_x, max_x) = (start.x.min(end.x), start.x.max(end.x));
                return line >= min_y && line <= max_y && col >= min_x && col < max_x;
            }

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

        // Block mode: each row contributes its `[min_x, max_x)` column slice
        // (clamped to the line's length; short rows contribute an empty slice),
        // joined by newlines.
        if self.selection_mode == SelectionMode::Block {
            let (min_y, max_y) = (start.y.min(end.y), start.y.max(end.y));
            let (min_x, max_x) = (start.x.min(end.x), start.x.max(end.x));
            if min_x == max_x {
                return None;
            }
            let mut rows = Vec::new();
            for y in min_y..=max_y {
                if y < 0 || y >= self.lines.len() as i16 {
                    continue;
                }
                let line_idx = y as usize;
                let line_char_len = self.lines[line_idx].chars().count();
                let s_char = (min_x.max(0) as usize).min(line_char_len);
                let e_char = (max_x.max(0) as usize).min(line_char_len);
                if s_char < e_char {
                    let s_byte = self.char_to_byte_idx(line_idx, s_char);
                    let e_byte = self.char_to_byte_idx(line_idx, e_char);
                    rows.push(self.lines[line_idx][s_byte..e_byte].to_string());
                } else {
                    rows.push(String::new());
                }
            }
            return Some(rows.join("\n"));
        }

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

    pub fn select_all(&mut self) {
        self.selection_start = Some(Point::zero());
        self.selection_mode = SelectionMode::Stream;
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

    pub fn delete_selection(&mut self) {
        if !self.has_selection() || self.read_only {
            return;
        }

        if self.selection_mode == SelectionMode::Block {
            self.delete_block_selection();
            return;
        }

        if let Some(text) = self.get_selection() {
            let action = EditAction::DeleteText {
                pos: self.selection_start.unwrap(),
                text,
            };
            self.delete_selection_internal();
            self.push_undo(action);
        }
    }

    /// Delete a rectangular block selection, removing the column band
    /// `[min_x, max_x)` from every row in range. Recorded as a single
    /// `Compound` undo step (one per-row deletion each), so one Ctrl+Z restores
    /// the whole block. Each per-row slice is newline-free, so replaying it as a
    /// stream `DeleteText` reproduces the same single-row removal.
    fn delete_block_selection(&mut self) {
        let start = self.selection_start.unwrap();
        let end = self.cursor;
        let (min_y, max_y) = (start.y.min(end.y), start.y.max(end.y));
        let (min_x, max_x) = (start.x.min(end.x), start.x.max(end.x));
        if min_x == max_x {
            self.selection_start = None;
            self.selection_mode = SelectionMode::Stream;
            return;
        }

        let mut actions: Vec<EditAction> = Vec::new();
        for y in min_y..=max_y {
            if y < 0 || y >= self.lines.len() as i16 {
                continue;
            }
            let line_idx = y as usize;
            let line_char_len = self.lines[line_idx].chars().count();
            let s_char = (min_x.max(0) as usize).min(line_char_len);
            let e_char = (max_x.max(0) as usize).min(line_char_len);
            if s_char >= e_char {
                continue; // nothing on this row within the band
            }
            let s_byte = self.char_to_byte_idx(line_idx, s_char);
            let e_byte = self.char_to_byte_idx(line_idx, e_char);
            let removed = self.lines[line_idx][s_byte..e_byte].to_string();
            self.lines[line_idx].drain(s_byte..e_byte);
            actions.push(EditAction::DeleteText {
                pos: Point::new(min_x, y),
                text: removed,
            });
        }

        self.cursor = Point::new(min_x, min_y);
        self.selection_start = None;
        self.selection_mode = SelectionMode::Stream;
        self.modified = true;
        self.ensure_cursor_visible();

        if !actions.is_empty() {
            self.push_undo(EditAction::Compound(actions));
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
    ///
    /// Pushes exactly one undo entry — a `Compound` covering both the
    /// deletion of any active selection and the subsequent insert — so
    /// a single Ctrl+Z reverts the whole paste in one step.
    pub fn clip_paste(&mut self) -> bool {
        if self.read_only {
            return false;
        }

        let text = clipboard::get_clipboard();
        if text.is_empty() {
            return false;
        }

        let mut actions: Vec<EditAction> = Vec::new();

        // Capture and delete the active selection without going through
        // `delete_selection`, which would push its own undo entry and
        // break atomicity.
        if self.has_selection() {
            if let Some(selected) = self.get_selection() {
                let start = self.selection_start.unwrap();
                let end = self.cursor;
                let pos = if start.y < end.y || (start.y == end.y && start.x < end.x) {
                    start
                } else {
                    end
                };
                actions.push(EditAction::DeleteText {
                    pos,
                    text: selected,
                });
                self.delete_selection_internal();
            }
        }

        // After delete_selection_internal the cursor sits at the canonical
        // top-left of the now-removed selection — that's also the insert
        // anchor.
        let insert_pos = self.cursor;
        actions.push(EditAction::InsertText {
            pos: insert_pos,
            text: text.clone(),
        });
        self.insert_text_internal(&text);

        // Avoid wrapping a single insert in a Compound — keeps the
        // undo stack flat for the no-selection case.
        let entry = if actions.len() == 1 {
            actions.into_iter().next().unwrap()
        } else {
            EditAction::Compound(actions)
        };
        self.push_undo(entry);
        true
    }

    fn insert_text_internal(&mut self, text: &str) {
        if self.read_only {
            return;
        }

        // Split on '\n' (not .lines()) so a trailing or lone newline still
        // produces a line break — undo/redo relies on exact round-tripping
        let lines_to_insert: Vec<&str> = text.split('\n').collect();
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

        let action = EditAction::InsertText {
            pos: self.cursor,
            text: text.to_string(),
        };
        self.insert_text_internal(text);
        self.push_undo(action);
    }
}

impl View for EditorWindow {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
        // Note: Scrollbars and indicator are now children of the Window, not the EditorWindow
        // The Window's interior Group automatically handles their positioning
        // We only need to update our internal state
        self.clamp_cursor();
        self.ensure_cursor_visible();
        self.update_scrollbars();
    }

    fn grow_mode(&self) -> crate::core::state::GrowFlags {
        self.grow_mode
    }

    fn set_grow_mode(&mut self, grow_mode: crate::core::state::GrowFlags) {
        self.grow_mode = grow_mode;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        use crate::core::palette::{EDITOR_CURSOR, EDITOR_NORMAL, EDITOR_SELECTED};

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
                                    self.map_color(token.token_type.palette_index()),
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

            if cursor_screen_x >= content_area.a.x
                && cursor_screen_x < content_area.b.x
                && cursor_screen_y >= content_area.a.y
                && cursor_screen_y < content_area.b.y
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
        // They are separate child views, not owned by the EditorWindow
    }

    fn handle_event(&mut self, event: &mut Event) {
        // Select-all command (e.g. an Edit menu item or command dispatch).
        // The Ctrl+A keystroke is handled separately in the keyboard path.
        if event.what == EventType::Command && event.command == crate::core::command::CM_SELECT_ALL
        {
            if self.is_focused() {
                self.select_all();
                event.clear();
            }
            return;
        }

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

            // The global block-edit mode decides whether the drag is a
            // rectangular (block) selection. The mode is locked in at press
            // time for the whole drag (MouseMove events don't re-check).
            self.selection_mode = Self::mode_from_global();

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

        // Mouse wheel scrolling
        if event.what == EventType::MouseWheelUp {
            let content_area = self.get_content_area();
            if content_area.contains(event.mouse.pos) {
                self.delta.y = (self.delta.y - 3).max(0);
                self.update_scrollbars();
                event.clear();
                return;
            }
        }
        if event.what == EventType::MouseWheelDown {
            let content_area = self.get_content_area();
            if content_area.contains(event.mouse.pos) {
                let max_y = (self.lines.len() as i16).saturating_sub(content_area.height());
                self.delta.y = (self.delta.y + 3).min(max_y).max(0);
                self.update_scrollbars();
                event.clear();
                return;
            }
        }

        if event.what == EventType::Keyboard {
            // Only handle keyboard events if focused
            if !self.is_focused() {
                return;
            }

            // Shift extends the selection; its shape (stream or block) comes
            // from the global block-edit mode, not from a modifier, because
            // terminals disagree on delivering Alt/Option (see
            // set_selection_mode_for).
            use crossterm::event::KeyModifiers;
            let shift_pressed = event.key_modifiers.contains(KeyModifiers::SHIFT);
            let extend = shift_pressed;

            match event.key_code {
                KB_UP => {
                    self.set_selection_mode_for(extend);
                    self.move_cursor(0, -1, extend);
                    event.clear();
                }
                KB_DOWN => {
                    self.set_selection_mode_for(extend);
                    self.move_cursor(0, 1, extend);
                    event.clear();
                }
                KB_LEFT => {
                    self.set_selection_mode_for(extend);
                    if event.key_modifiers.contains(KeyModifiers::CONTROL) {
                        // Ctrl+Left: previous word (Borland cmWordLeft)
                        self.move_word(false, extend);
                    } else {
                        // Move left (previous character), wrapping to previous line if at start
                        self.move_cursor_left(extend);
                    }
                    event.clear();
                }
                KB_RIGHT => {
                    self.set_selection_mode_for(extend);
                    if event.key_modifiers.contains(KeyModifiers::CONTROL) {
                        // Ctrl+Right: next word (Borland cmWordRight)
                        self.move_word(true, extend);
                    } else {
                        // Move right (following character), wrapping to following line if at end
                        self.move_cursor_right(extend);
                    }
                    event.clear();
                }
                crate::core::event::KB_INS => {
                    // Toggle insert/overwrite mode (Borland cmInsMode)
                    self.toggle_insert_mode();
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
                    if event.key_modifiers.contains(KeyModifiers::CONTROL) {
                        // Ctrl+Backspace: delete previous word
                        self.delete_word(false);
                    } else if self.has_selection() {
                        self.delete_selection();
                    } else {
                        self.backspace();
                    }
                    event.clear();
                }
                KB_DEL => {
                    if event.key_modifiers.contains(KeyModifiers::CONTROL) {
                        // Ctrl+Del: delete next word (Borland cmDelWord)
                        self.delete_word(true);
                    } else if self.has_selection() {
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
                    // Accept printable characters (Unicode BMP, excludes control chars).
                    // Key codes above 0xFF with a zero low byte are special keys
                    // (Alt combos, function keys, arrow keys) that must NOT be
                    // inserted as text — they need to propagate to the menu bar
                    // and application for shortcut handling.
                    let is_special = key_code > 0xFF && (key_code & 0xFF) == 0;
                    if !is_special {
                        if let Some(ch) = char::from_u32(key_code as u32) {
                            if !ch.is_control() {
                                self.insert_char(ch);
                                event.clear();
                            }
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

    fn set_palette_chain(&mut self, node: Option<crate::core::palette_chain::PaletteChainNode>) {
        self.palette_chain = node;
    }

    fn get_palette_chain(&self) -> Option<&crate::core::palette_chain::PaletteChainNode> {
        self.palette_chain.as_ref()
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        // EditorWindow uses cpEditor palette for proper color remapping through window hierarchy
        // Matches Borland: cpEditor = [6, 7] for normal and selected text
        Some(Palette::from_slice(palettes::CP_EDITOR))
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
        let mut editor = EditorWindow::new(bounds);

        editor.load_file(file.path().to_str().unwrap()).unwrap();

        assert_eq!(editor.line_count(), 3);
        assert_eq!(editor.get_text(), "Line 1\nLine 2\nLine 3");
        assert_eq!(editor.get_filename(), file.path().to_str());
        assert!(!editor.is_modified());
    }

    #[test]
    fn test_editor_save_as() {
        let bounds = Rect::new(0, 0, 80, 25);
        let mut editor = EditorWindow::new(bounds);

        editor.set_text("Hello\nWorld");

        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap();

        editor.save_as(path).unwrap();

        let content = std::fs::read_to_string(path).unwrap();
        // Saves append a trailing newline by default (POSIX text files)
        assert_eq!(content, "Hello\nWorld\n");
        assert_eq!(editor.get_filename(), Some(path));
        assert!(!editor.is_modified());
    }

    #[test]
    fn test_editor_save_file() {
        let bounds = Rect::new(0, 0, 80, 25);
        let mut editor = EditorWindow::new(bounds);

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
        assert_eq!(content, "Modified content\n");
        assert!(!editor.is_modified());
    }

    #[test]
    fn test_editor_modified_flag() {
        let bounds = Rect::new(0, 0, 80, 25);
        let mut editor = EditorWindow::new(bounds);

        assert!(!editor.is_modified());

        editor.set_text("Some text");
        assert!(!editor.is_modified()); // set_text clears modified flag

        // Simulate typing (would set modified via push_undo)
        let file = NamedTempFile::new().unwrap();
        editor.save_as(file.path().to_str().unwrap()).unwrap();
        assert!(!editor.is_modified());
    }

    #[test]
    fn paste_over_selection_undoes_in_one_step() {
        // Regression: clip_paste used to push two undo entries when a
        // selection was active (delete_selection + insert_text), so a
        // single Ctrl+Z only un-inserted but left the original
        // selection still gone.
        let bounds = Rect::new(0, 0, 80, 25);
        let mut editor = EditorWindow::new(bounds);
        editor.set_text("hello world");

        // Select "world" and put "RUST" on the clipboard.
        editor.cursor = Point::new(6, 0);
        editor.selection_start = Some(Point::new(6, 0));
        editor.cursor.x = 11;
        clipboard::set_clipboard("RUST");

        assert!(editor.clip_paste());
        assert_eq!(editor.get_text(), "hello RUST");

        // One Ctrl+Z must restore the original buffer entirely.
        editor.undo();
        assert_eq!(editor.get_text(), "hello world");

        // And one redo must reapply the whole paste.
        editor.redo();
        assert_eq!(editor.get_text(), "hello RUST");
    }

    #[test]
    fn paste_without_selection_is_single_undo_entry() {
        let bounds = Rect::new(0, 0, 80, 25);
        let mut editor = EditorWindow::new(bounds);
        editor.set_text("abc");
        editor.cursor = Point::new(3, 0);
        clipboard::set_clipboard("XYZ");

        assert!(editor.clip_paste());
        assert_eq!(editor.get_text(), "abcXYZ");
        editor.undo();
        assert_eq!(editor.get_text(), "abc");
    }

    #[test]
    fn two_stacks_preserve_undo_history_through_redo_cycle() {
        // Verifies the stack-position semantics the IDE menu enablement
        // relies on: after N edits you can undo N times, after which
        // can_undo flips off and can_redo stays on; redoing brings the
        // actions back; a fresh edit then clears the redo branch.
        let bounds = Rect::new(0, 0, 80, 25);
        let mut editor = EditorWindow::new(bounds);
        editor.set_text("");
        assert!(!editor.can_undo());
        assert!(!editor.can_redo());

        editor.cursor = Point::new(0, 0);
        editor.insert_char('A');
        editor.insert_char('B');
        assert_eq!(editor.get_text(), "AB");
        assert!(editor.can_undo());
        assert!(!editor.can_redo());

        editor.undo();
        assert_eq!(editor.get_text(), "A");
        assert!(editor.can_undo());
        assert!(editor.can_redo());

        editor.undo();
        assert_eq!(editor.get_text(), "");
        // Undo stack is now empty — but the actions are NOT lost; they
        // live on the redo stack.
        assert!(!editor.can_undo());
        assert!(editor.can_redo());

        editor.redo();
        assert_eq!(editor.get_text(), "A");
        assert!(editor.can_undo());
        assert!(editor.can_redo());

        editor.redo();
        assert_eq!(editor.get_text(), "AB");
        assert!(editor.can_undo());
        assert!(!editor.can_redo());

        // Undo once, then make a fresh edit: the redo branch must be
        // discarded (push_undo clears redo_stack).
        editor.undo();
        assert!(editor.can_redo());
        editor.insert_char('C');
        assert_eq!(editor.get_text(), "AC");
        assert!(editor.can_undo());
        assert!(!editor.can_redo());
    }

    #[test]
    fn cut_undoes_and_redoes_atomically() {
        let bounds = Rect::new(0, 0, 80, 25);
        let mut editor = EditorWindow::new(bounds);
        editor.set_text("foo bar");
        editor.cursor = Point::new(4, 0);
        editor.selection_start = Some(Point::new(4, 0));
        editor.cursor.x = 7;

        assert!(editor.clip_cut());
        assert_eq!(editor.get_text(), "foo ");
        editor.undo();
        assert_eq!(editor.get_text(), "foo bar");
        editor.redo();
        assert_eq!(editor.get_text(), "foo ");
    }

    #[test]
    fn test_editor_load_empty_file() {
        let file = NamedTempFile::new().unwrap();
        // Don't write anything - file is empty

        let bounds = Rect::new(0, 0, 80, 25);
        let mut editor = EditorWindow::new(bounds);

        editor.load_file(file.path().to_str().unwrap()).unwrap();

        assert_eq!(editor.line_count(), 1); // EditorWindow always has at least one line
        assert_eq!(editor.get_text(), "");
        assert!(!editor.is_modified());
    }

    #[test]
    fn newline_and_line_joins_are_undoable() {
        let mut editor = EditorWindow::new(Rect::new(0, 0, 80, 25));
        editor.set_text("abc");
        editor.cursor = Point::new(3, 0);

        // Type Enter then "def"
        editor.insert_newline();
        editor.insert_char('d');
        editor.insert_char('e');
        editor.insert_char('f');
        assert_eq!(editor.get_text(), "abc\ndef");

        // Undo everything, including the Enter
        editor.undo();
        editor.undo();
        editor.undo();
        assert_eq!(editor.get_text(), "abc\n");
        editor.undo();
        assert_eq!(editor.get_text(), "abc");

        // Redo restores the same shape
        for _ in 0..4 {
            editor.redo();
        }
        assert_eq!(editor.get_text(), "abc\ndef");

        // Backspace line-join is undoable
        editor.cursor = Point::new(0, 1);
        editor.backspace();
        assert_eq!(editor.get_text(), "abcdef");
        editor.undo();
        assert_eq!(editor.get_text(), "abc\ndef");

        // Delete-at-EOL line-join is undoable
        editor.cursor = Point::new(3, 0);
        editor.delete_char();
        assert_eq!(editor.get_text(), "abcdef");
        editor.undo();
        assert_eq!(editor.get_text(), "abc\ndef");
    }

    #[test]
    fn overwrite_mode_is_single_undo_step() {
        let mut editor = EditorWindow::new(Rect::new(0, 0, 80, 25));
        editor.set_text("xyz");
        editor.cursor = Point::new(0, 0);
        editor.toggle_insert_mode();

        editor.insert_char('A');
        assert_eq!(editor.get_text(), "Ayz");
        editor.undo();
        assert_eq!(editor.get_text(), "xyz");
    }

    #[test]
    fn replace_all_terminates_when_replacement_contains_pattern() {
        let mut editor = EditorWindow::new(Rect::new(0, 0, 80, 25));
        editor.set_text("a b a");

        let count = editor.replace_all("a", "aa", SearchOptions::default());

        assert_eq!(count, 2);
        assert_eq!(editor.get_text(), "aa b aa");
    }

    #[test]
    fn search_handles_multibyte_lines() {
        let mut editor = EditorWindow::new(Rect::new(0, 0, 80, 25));
        editor.set_text("héllo wörld\nsécond ligne");

        // Match after a multibyte char: column must be in characters
        let pos = editor.find("wörld", SearchOptions::default()).unwrap();
        assert_eq!((pos.x, pos.y), (6, 0));

        // Case-insensitive search across multibyte text
        let mut editor = EditorWindow::new(Rect::new(0, 0, 80, 25));
        editor.set_text("héllo wörld\nsécond ligne");
        let pos = editor.find(
            "SÉCOND",
            SearchOptions {
                case_sensitive: false,
                ..SearchOptions::default()
            },
        );
        assert_eq!(pos.map(|p| (p.x, p.y)), Some((0, 1)));
    }

    #[test]
    fn find_next_finds_adjacent_matches() {
        let mut editor = EditorWindow::new(Rect::new(0, 0, 80, 25));
        editor.set_text("abab");

        let first = editor.find("ab", SearchOptions::default()).unwrap();
        assert_eq!((first.x, first.y), (0, 0));
        let second = editor.find_next().unwrap();
        assert_eq!((second.x, second.y), (2, 0));
        // No wrap-around: Borland's search stops at end of document
        assert!(editor.find_next().is_none());
    }

    #[test]
    fn word_movement_crosses_lines() {
        use crossterm::event::KeyModifiers;

        let mut editor = EditorWindow::new(Rect::new(0, 0, 80, 25));
        editor.set_text("foo bar\nbaz");
        editor.set_focus(true);
        editor.cursor = Point::new(0, 0);

        // Ctrl+Right: foo| -> bar| -> next line baz
        let mut ev = Event::keyboard(KB_RIGHT);
        ev.key_modifiers = KeyModifiers::CONTROL;
        editor.handle_event(&mut ev);
        assert_eq!((editor.cursor.x, editor.cursor.y), (4, 0)); // start of "bar"

        let mut ev = Event::keyboard(KB_RIGHT);
        ev.key_modifiers = KeyModifiers::CONTROL;
        editor.handle_event(&mut ev);
        assert_eq!((editor.cursor.x, editor.cursor.y), (0, 1)); // start of "baz"

        // Ctrl+Left goes back to "bar"
        let mut ev = Event::keyboard(KB_LEFT);
        ev.key_modifiers = KeyModifiers::CONTROL;
        editor.handle_event(&mut ev);
        assert_eq!((editor.cursor.x, editor.cursor.y), (4, 0));
    }

    #[test]
    fn delete_word_is_single_undo_step() {
        let mut editor = EditorWindow::new(Rect::new(0, 0, 80, 25));
        editor.set_text("foo bar baz");
        editor.cursor = Point::new(4, 0); // start of "bar"

        editor.delete_word(true); // deletes "bar " up to "baz"
        assert_eq!(editor.get_text(), "foo baz");
        editor.undo();
        assert_eq!(editor.get_text(), "foo bar baz");

        // Backwards: cursor after "bar ", delete previous word
        editor.cursor = Point::new(8, 0); // start of "baz"
        editor.delete_word(false);
        assert_eq!(editor.get_text(), "foo baz");
        editor.undo();
        assert_eq!(editor.get_text(), "foo bar baz");
    }

    #[test]
    fn ins_key_toggles_overwrite() {
        let mut editor = EditorWindow::new(Rect::new(0, 0, 80, 25));
        editor.set_text("abc");
        editor.set_focus(true);
        editor.cursor = Point::new(0, 0);

        let mut ev = Event::keyboard(crate::core::event::KB_INS);
        editor.handle_event(&mut ev);
        let mut ev = Event::keyboard('X' as u16);
        editor.handle_event(&mut ev);
        assert_eq!(editor.get_text(), "Xbc");
    }

    #[test]
    fn save_preserves_line_ending_style() {
        use std::io::Write;
        let mut file = NamedTempFile::new().unwrap();
        // CRLF file without trailing newline
        write!(file, "one\r\ntwo").unwrap();
        file.flush().unwrap();

        let mut editor = EditorWindow::new(Rect::new(0, 0, 80, 25));
        editor.load_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(editor.get_text(), "one\ntwo");

        let out = NamedTempFile::new().unwrap();
        editor.save_as(out.path().to_str().unwrap()).unwrap();
        let round = std::fs::read_to_string(out.path()).unwrap();
        // CRLF endings and missing trailing newline both preserved
        assert_eq!(round, "one\r\ntwo");
    }

    #[test]
    fn backup_files_creates_bak() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data.txt");
        std::fs::write(&path, "old contents\n").unwrap();

        let mut editor = EditorWindow::new(Rect::new(0, 0, 80, 25));
        editor.load_file(path.to_str().unwrap()).unwrap();
        editor.set_backup_files(true);
        editor.insert_char('x');
        editor.save_file().unwrap();

        let bak = dir.path().join("data.txt.bak");
        assert_eq!(std::fs::read_to_string(&bak).unwrap(), "old contents\n");
        assert!(std::fs::read_to_string(&path).unwrap().starts_with('x'));
    }

    // ---- Rectangular (block) selection ----

    fn block_editor() -> EditorWindow {
        let mut editor = EditorWindow::new(Rect::new(0, 0, 80, 25));
        editor.set_text("abcde\nfghij\nklmno");
        // Column band [1, 3) over rows 0..=2.
        editor.selection_start = Some(Point::new(1, 0));
        editor.cursor = Point::new(3, 2);
        editor.selection_mode = SelectionMode::Block;
        editor
    }

    #[test]
    fn block_is_position_selected_is_a_column_band() {
        let editor = block_editor();
        // In-band columns (1,2) across every row are selected...
        assert!(editor.is_position_selected(0, 1));
        assert!(editor.is_position_selected(0, 2));
        assert!(editor.is_position_selected(1, 1)); // middle row, band only
        assert!(editor.is_position_selected(2, 2));
        // ...out-of-band columns are not — unlike stream, the middle row is NOT
        // fully selected.
        assert!(!editor.is_position_selected(0, 0));
        assert!(!editor.is_position_selected(0, 3));
        assert!(!editor.is_position_selected(1, 0));
        assert!(!editor.is_position_selected(1, 3));
    }

    #[test]
    fn block_get_selection_copies_each_rows_column_slice() {
        let editor = block_editor();
        assert_eq!(editor.get_selection().as_deref(), Some("bc\ngh\nlm"));
    }

    #[test]
    fn block_delete_removes_band_and_undoes_in_one_step() {
        let mut editor = block_editor();
        editor.delete_selection();
        assert_eq!(editor.get_text(), "ade\nfij\nkno");
        assert_eq!((editor.cursor.x, editor.cursor.y), (1, 0));

        editor.undo();
        assert_eq!(editor.get_text(), "abcde\nfghij\nklmno");
    }

    #[test]
    fn block_get_selection_clamps_short_rows() {
        let mut editor = EditorWindow::new(Rect::new(0, 0, 80, 25));
        editor.set_text("abcdef\ngh\nklmnop");
        // Band [2, 5) over rows 0..=2; the middle row "gh" has nothing there.
        editor.selection_start = Some(Point::new(2, 0));
        editor.cursor = Point::new(5, 2);
        editor.selection_mode = SelectionMode::Block;
        // Rows: "abcdef"[2..5]="cde", "gh" has nothing at cols 2..5 (empty),
        // "klmnop"[2..5]="mno".
        assert_eq!(editor.get_selection().as_deref(), Some("cde\n\nmno"));
    }

    /// Serializes the tests that flip the process-wide block-edit mode, and
    /// clears it again when the guard drops.
    struct BlockModeGuard(std::sync::MutexGuard<'static, ()>);

    impl BlockModeGuard {
        fn on() -> Self {
            static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
            let guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
            crate::core::state::set_block_edit_mode(true);
            Self(guard)
        }
    }

    impl Drop for BlockModeGuard {
        fn drop(&mut self) {
            crate::core::state::set_block_edit_mode(false);
        }
    }

    #[test]
    fn arrow_starts_block_selection_in_block_edit_mode() {
        use crossterm::event::KeyModifiers;

        let _mode = BlockModeGuard::on();

        let mut editor = EditorWindow::new(Rect::new(0, 0, 80, 25));
        editor.set_text("abcde\nfghij\nklmno");
        editor.set_focus(true);
        editor.cursor = Point::new(1, 0);

        // Shift+Right then Shift+Down builds a 2x2-ish block.
        let mut ev = Event::keyboard(KB_RIGHT);
        ev.key_modifiers = KeyModifiers::SHIFT;
        editor.handle_event(&mut ev);
        let mut ev = Event::keyboard(KB_DOWN);
        ev.key_modifiers = KeyModifiers::SHIFT;
        editor.handle_event(&mut ev);

        assert_eq!(editor.selection_mode, SelectionMode::Block);
        assert!(editor.has_selection());
        assert_eq!(editor.selection_start, Some(Point::new(1, 0)));
        assert_eq!((editor.cursor.x, editor.cursor.y), (2, 1));
    }

    #[test]
    fn mouse_drag_starts_block_selection_in_block_edit_mode() {
        let _mode = BlockModeGuard::on();

        let mut editor = EditorWindow::new(Rect::new(0, 0, 80, 25));
        editor.set_text("abcde\nfghij\nklmno");
        editor.set_focus(true);

        // MouseDown at (1,0)...
        let mut down = Event::mouse(
            EventType::MouseDown,
            Point::new(1, 0),
            MB_LEFT_BUTTON,
            false,
        );
        editor.handle_event(&mut down);
        assert_eq!(editor.selection_mode, SelectionMode::Block);

        // ...drag to (3,2) with the button still held.
        let mut mv = Event::mouse(
            EventType::MouseMove,
            Point::new(3, 2),
            MB_LEFT_BUTTON,
            false,
        );
        editor.handle_event(&mut mv);

        assert_eq!(editor.selection_mode, SelectionMode::Block);
        assert!(editor.has_selection());
        assert_eq!(editor.get_selection().as_deref(), Some("bc\ngh\nlm"));
    }

    #[test]
    fn cm_select_all_command_selects_whole_buffer_when_focused() {
        let mut editor = EditorWindow::new(Rect::new(0, 0, 80, 25));
        editor.set_text("abc\ndef");
        editor.set_focus(true);

        let mut ev = Event::command(crate::core::command::CM_SELECT_ALL);
        editor.handle_event(&mut ev);

        assert!(editor.has_selection());
        assert_eq!(editor.get_selection().as_deref(), Some("abc\ndef"));
        assert_eq!(ev.what, EventType::Nothing); // command was consumed
    }

    #[test]
    fn cm_select_all_command_ignored_when_unfocused() {
        let mut editor = EditorWindow::new(Rect::new(0, 0, 80, 25));
        editor.set_text("abc\ndef");
        // not focused

        let mut ev = Event::command(crate::core::command::CM_SELECT_ALL);
        editor.handle_event(&mut ev);

        assert!(!editor.has_selection());
        assert_eq!(ev.what, EventType::Command); // not consumed
    }

    #[test]
    fn click_resets_to_stream_mode_when_block_edit_mode_is_off() {
        let mut editor = block_editor();
        assert_eq!(editor.selection_mode, SelectionMode::Block);

        // With block-edit mode off, a click drops back to stream selection.
        let mut down = Event::mouse(
            EventType::MouseDown,
            Point::new(0, 0),
            MB_LEFT_BUTTON,
            false,
        );
        editor.set_focus(true);
        editor.handle_event(&mut down);
        assert_eq!(editor.selection_mode, SelectionMode::Stream);
    }
}
