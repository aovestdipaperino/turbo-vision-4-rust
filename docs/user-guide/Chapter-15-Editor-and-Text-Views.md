# Chapter 15 — Editor and Text Views (Rust Edition)

**Version:** 0.2.11
**Updated:** 2025-11-04

This chapter explores text editing capabilities in Turbo Vision. You'll learn about Terminal views for debugging output, the buffer gap editor architecture, undo/redo mechanisms, clipboard operations, and building full-featured text editors.

**Prerequisites:** Chapters 8-12 (Views, Events, Application, Windows, Controls)

---

## Table of Contents

1. [Understanding Text Views](#understanding-text-views)
2. [Terminal View](#terminal-view)
3. [Buffer Gap Architecture](#buffer-gap-architecture)
4. [Editor Implementation](#editor-implementation)
5. [Undo and Redo](#undo-and-redo)
6. [Clipboard Operations](#clipboard-operations)
7. [Search and Replace](#search-and-replace)
8. [Memo Field](#memo-field)
9. [File Editor](#file-editor)
10. [Editor Window](#editor-window)
11. [Complete Examples](#complete-examples)

---

## Understanding Text Views

### Types of Text Views

Turbo Vision provides three types of text views:

**1. Terminal View** - Write-only scrolling text:
```rust
pub struct Terminal {
    bounds: Rect,
    buffer: VecDeque<String>,
    max_lines: usize,
    state: StateFlags,
}

// Used for: Debug output, build logs, console
```

**2. Editor** - Full text editing with undo:
```rust
pub struct Editor {
    bounds: Rect,
    buffer: EditorBuffer,  // Buffer gap architecture
    undo_stack: Vec<UndoAction>,
    clipboard: Option<String>,
    state: StateFlags,
}

// Used for: Text editing, code editing, notes
```

**3. Memo** - Multi-line input control:
```rust
pub struct Memo {
    bounds: Rect,
    data: Rc<RefCell<String>>,
    cursor_pos: usize,
    scroll_pos: usize,
    state: StateFlags,
}

// Used for: Forms, multi-line input fields
```

### Comparison

| Feature | Terminal | Editor | Memo |
|---------|----------|--------|------|
| **Input** | Write-only | Full editing | User input |
| **Undo** | No | Yes | No |
| **Clipboard** | No | Yes | Limited |
| **Files** | No | Yes | No |
| **Use Case** | Logging | Documents | Forms |

---

## Terminal View

### What is a Terminal View?

A **Terminal** is a write-only scrolling text view for debugging and logging:

```rust
pub struct Terminal {
    bounds: Rect,
    buffer: VecDeque<String>,
    max_lines: usize,
    top_line: usize,  // Scroll position
    scrollbar: Option<ScrollBar>,
    state: StateFlags,
}
```

### Creating a Terminal

```rust
impl Terminal {
    pub fn new(bounds: Rect, max_lines: usize) -> Self {
        Self {
            bounds,
            buffer: VecDeque::new(),
            max_lines,
            top_line: 0,
            scrollbar: None,
            state: SF_VISIBLE,
        }
    }

    pub fn with_scrollbar(mut self, scrollbar: ScrollBar) -> Self {
        self.scrollbar = Some(scrollbar);
        self
    }
}
```

### Writing to Terminal

```rust
impl Terminal {
    /// Write a line to the terminal
    pub fn writeln(&mut self, text: &str) {
        // Add line to buffer
        self.buffer.push_back(text.to_string());

        // Limit buffer size
        while self.buffer.len() > self.max_lines {
            self.buffer.pop_front();
        }

        // Auto-scroll to bottom
        self.scroll_to_bottom();
        self.update_scrollbar();
    }

    /// Write without newline
    pub fn write(&mut self, text: &str) {
        if let Some(last) = self.buffer.back_mut() {
            last.push_str(text);
        } else {
            self.buffer.push_back(text.to_string());
        }
    }

    /// Clear the terminal
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.top_line = 0;
        self.update_scrollbar();
    }

    fn scroll_to_bottom(&mut self) {
        let visible_lines = self.bounds.height() as usize;
        if self.buffer.len() > visible_lines {
            self.top_line = self.buffer.len() - visible_lines;
        } else {
            self.top_line = 0;
        }
    }

    fn update_scrollbar(&mut self) {
        if let Some(ref mut scrollbar) = self.scrollbar {
            scrollbar.set_range(0, self.buffer.len() as i32);
            scrollbar.set_value(self.top_line as i32);
        }
    }
}
```

### Drawing Terminal

```rust
impl View for Terminal {
    fn draw(&mut self, terminal: &mut crate::Terminal) {
        let width = self.bounds.width() as usize;
        let height = self.bounds.height() as usize;

        let color = self.get_color(1);

        for row in 0..height {
            let line_idx = self.top_line + row;
            let mut buf = DrawBuffer::new(width);

            if line_idx < self.buffer.len() {
                let line = &self.buffer[line_idx];
                buf.move_str(0, line, color);

                // Pad rest of line
                let len = line.len().min(width);
                buf.move_char(len, ' ', color, width - len);
            } else {
                // Empty line
                buf.move_char(0, ' ', color, width);
            }

            write_line_to_terminal(
                terminal,
                self.bounds.a.x,
                self.bounds.a.y + row as i16,
                &buf,
            );
        }
    }
}
```

### Terminal Event Handling

```rust
impl View for Terminal {
    fn handle_event(&mut self, event: &mut Event) {
        match event.what {
            EventType::Keyboard => {
                match event.key_code {
                    KB_UP => {
                        if self.top_line > 0 {
                            self.top_line -= 1;
                            self.update_scrollbar();
                        }
                        event.clear();
                    }
                    KB_DOWN => {
                        let max_scroll = self.buffer.len().saturating_sub(
                            self.bounds.height() as usize
                        );
                        if self.top_line < max_scroll {
                            self.top_line += 1;
                            self.update_scrollbar();
                        }
                        event.clear();
                    }
                    KB_PGUP => {
                        let visible = self.bounds.height() as usize;
                        self.top_line = self.top_line.saturating_sub(visible);
                        self.update_scrollbar();
                        event.clear();
                    }
                    KB_PGDN => {
                        let visible = self.bounds.height() as usize;
                        let max_scroll = self.buffer.len().saturating_sub(visible);
                        self.top_line = (self.top_line + visible).min(max_scroll);
                        self.update_scrollbar();
                        event.clear();
                    }
                    _ => {}
                }
            }
            EventType::Broadcast => {
                if event.command == CM_SCROLLBAR_CHANGED {
                    if let Some(ref scrollbar) = self.scrollbar {
                        self.top_line = scrollbar.value() as usize;
                        event.clear();
                    }
                }
            }
            _ => {}
        }
    }
}
```

### Terminal Usage Example

```rust
// Create terminal window
let mut window = Window::new(
    Rect::new(5, 3, 75, 20),
    "Build Output"
);

// Create terminal
let terminal = Terminal::new(
    window.interior_bounds(),
    1000  // Max 1000 lines
).with_scrollbar(ScrollBar::new(
    Rect::new(68, 0, 69, 16)
));

window.add(Box::new(terminal));

// Write to terminal
terminal.writeln("Building project...");
terminal.writeln("Compiling main.rs");
terminal.writeln("   Finished release [optimized] target(s)");
terminal.writeln("");
terminal.writeln("Build complete!");
```

---

## Buffer Gap Architecture

### What is Buffer Gap?

The **buffer gap** is a classic text editor data structure that makes insertion and deletion at the cursor position O(1):

```
┌─────────────────────────────────────────┐
│ Text before cursor │ GAP │ Text after  │
└─────────────────────────────────────────┘
                      ↑
                   Cursor
```

**Benefits:**
- Constant-time insertion/deletion at cursor
- No memory allocation for small edits
- Cache-friendly (cursor edits in contiguous memory)

### Buffer Gap Implementation

```rust
pub struct EditorBuffer {
    buffer: Vec<u8>,        // Entire buffer
    gap_start: usize,       // Start of gap
    gap_end: usize,         // End of gap (exclusive)
}

impl EditorBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0; capacity],
            gap_start: 0,
            gap_end: capacity,
        }
    }

    pub fn with_text(text: &str) -> Self {
        let capacity = (text.len() + 1024).next_power_of_two();
        let mut buf = Self::new(capacity);
        buf.insert_text(0, text);
        buf
    }

    /// Gap size
    pub fn gap_size(&self) -> usize {
        self.gap_end - self.gap_start
    }

    /// Text size (excluding gap)
    pub fn len(&self) -> usize {
        self.buffer.len() - self.gap_size()
    }

    /// Text before cursor
    fn text_before_gap(&self) -> &[u8] {
        &self.buffer[0..self.gap_start]
    }

    /// Text after cursor
    fn text_after_gap(&self) -> &[u8] {
        &self.buffer[self.gap_end..self.buffer.len()]
    }

    /// Get full text
    pub fn text(&self) -> String {
        let mut result = Vec::new();
        result.extend_from_slice(self.text_before_gap());
        result.extend_from_slice(self.text_after_gap());
        String::from_utf8_lossy(&result).to_string()
    }

    /// Move gap to position
    fn move_gap_to(&mut self, pos: usize) {
        if pos < self.gap_start {
            // Move gap left
            let count = self.gap_start - pos;
            let new_gap_end = self.gap_end - count;
            self.buffer.copy_within(pos..self.gap_start, new_gap_end);
            self.gap_start = pos;
            self.gap_end = new_gap_end;
        } else if pos > self.gap_start {
            // Move gap right
            let count = pos - self.gap_start;
            self.buffer.copy_within(self.gap_end..self.gap_end + count, self.gap_start);
            self.gap_start += count;
            self.gap_end += count;
        }
    }

    /// Ensure gap has at least `size` bytes
    fn ensure_gap(&mut self, size: usize) {
        if self.gap_size() < size {
            // Grow buffer
            let new_capacity = (self.buffer.len() + size).next_power_of_two();
            let additional = new_capacity - self.buffer.len();

            // Move text after gap to end of new buffer
            let after_gap = self.text_after_gap().to_vec();
            self.buffer.resize(new_capacity, 0);
            let new_gap_end = new_capacity - after_gap.len();
            self.buffer[new_gap_end..].copy_from_slice(&after_gap);
            self.gap_end = new_gap_end;
        }
    }

    /// Insert text at position
    pub fn insert_text(&mut self, pos: usize, text: &str) {
        let bytes = text.as_bytes();
        self.move_gap_to(pos);
        self.ensure_gap(bytes.len());

        // Insert into gap
        self.buffer[self.gap_start..self.gap_start + bytes.len()]
            .copy_from_slice(bytes);
        self.gap_start += bytes.len();
    }

    /// Insert character at position
    pub fn insert_char(&mut self, pos: usize, ch: char) {
        let mut buf = [0u8; 4];
        let s = ch.encode_utf8(&mut buf);
        self.insert_text(pos, s);
    }

    /// Delete text range
    pub fn delete_range(&mut self, start: usize, end: usize) {
        if start >= end {
            return;
        }

        // Move gap to start
        self.move_gap_to(start);

        // Extend gap to cover deleted text
        let delete_count = end - start;
        self.gap_end = (self.gap_end + delete_count).min(self.buffer.len());
    }

    /// Delete character at position
    pub fn delete_char(&mut self, pos: usize) {
        if pos < self.len() {
            self.delete_range(pos, pos + 1);
        }
    }
}
```

### Buffer Gap Example

```rust
let mut buffer = EditorBuffer::new(64);

// Insert "Hello"
buffer.insert_text(0, "Hello");
// Buffer: "Hello│                                    │"
//                ↑ gap

// Insert " World"
buffer.insert_text(5, " World");
// Buffer: "Hello World│                            │"
//                     ↑ gap

// Move cursor to position 6 (after space)
// Gap moves to cursor position
buffer.move_gap_to(6);
// Buffer: "Hello │                            │World"
//                ↑ gap

// Delete "World" (backspace 5 times)
buffer.delete_range(6, 11);
// Buffer: "Hello │                                 │"
//                ↑ gap enlarged
```

---

## Editor Implementation

### Editor Structure

```rust
pub struct Editor {
    bounds: Rect,
    buffer: EditorBuffer,
    cursor_pos: usize,      // Logical position in text
    cursor_x: i16,          // Visual column
    cursor_y: i16,          // Visual row
    top_line: usize,        // First visible line
    left_col: usize,        // First visible column
    selection: Option<Selection>,
    undo_stack: Vec<UndoAction>,
    redo_stack: Vec<UndoAction>,
    modified: bool,
    state: StateFlags,
}

pub struct Selection {
    pub start: usize,
    pub end: usize,
}

impl Selection {
    pub fn range(&self) -> (usize, usize) {
        if self.start < self.end {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }
}
```

### Creating an Editor

```rust
impl Editor {
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            buffer: EditorBuffer::new(4096),
            cursor_pos: 0,
            cursor_x: 0,
            cursor_y: 0,
            top_line: 0,
            left_col: 0,
            selection: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            modified: false,
            state: SF_VISIBLE | SF_SELECTABLE,
        }
    }

    pub fn with_text(bounds: Rect, text: &str) -> Self {
        let mut editor = Self::new(bounds);
        editor.buffer = EditorBuffer::with_text(text);
        editor
    }

    pub fn text(&self) -> String {
        self.buffer.text()
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }
}
```

### Cursor Movement

```rust
impl Editor {
    /// Move cursor left
    pub fn move_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.update_cursor_position();
            self.ensure_cursor_visible();
        }
    }

    /// Move cursor right
    pub fn move_right(&mut self) {
        if self.cursor_pos < self.buffer.len() {
            self.cursor_pos += 1;
            self.update_cursor_position();
            self.ensure_cursor_visible();
        }
    }

    /// Move cursor up
    pub fn move_up(&mut self) {
        let line = self.cursor_line();
        if line > 0 {
            self.cursor_pos = self.line_start(line - 1) + self.cursor_x as usize;
            self.cursor_pos = self.cursor_pos.min(self.line_end(line - 1));
            self.update_cursor_position();
            self.ensure_cursor_visible();
        }
    }

    /// Move cursor down
    pub fn move_down(&mut self) {
        let line = self.cursor_line();
        let line_count = self.line_count();
        if line < line_count - 1 {
            self.cursor_pos = self.line_start(line + 1) + self.cursor_x as usize;
            self.cursor_pos = self.cursor_pos.min(self.line_end(line + 1));
            self.update_cursor_position();
            self.ensure_cursor_visible();
        }
    }

    /// Move to start of line
    pub fn move_home(&mut self) {
        let line = self.cursor_line();
        self.cursor_pos = self.line_start(line);
        self.update_cursor_position();
        self.ensure_cursor_visible();
    }

    /// Move to end of line
    pub fn move_end(&mut self) {
        let line = self.cursor_line();
        self.cursor_pos = self.line_end(line);
        self.update_cursor_position();
        self.ensure_cursor_visible();
    }

    /// Get current line number
    fn cursor_line(&self) -> usize {
        let text = self.buffer.text();
        text[..self.cursor_pos].chars().filter(|&c| c == '\n').count()
    }

    /// Get line start position
    fn line_start(&self, line: usize) -> usize {
        let text = self.buffer.text();
        let mut pos = 0;
        let mut current_line = 0;

        for (i, ch) in text.char_indices() {
            if current_line == line {
                return i;
            }
            if ch == '\n' {
                current_line += 1;
                pos = i + 1;
            }
        }

        pos
    }

    /// Get line end position
    fn line_end(&self, line: usize) -> usize {
        let text = self.buffer.text();
        let start = self.line_start(line);

        for (i, ch) in text[start..].char_indices() {
            if ch == '\n' {
                return start + i;
            }
        }

        text.len()
    }

    /// Get total line count
    fn line_count(&self) -> usize {
        let text = self.buffer.text();
        text.chars().filter(|&c| c == '\n').count() + 1
    }

    /// Update visual cursor position
    fn update_cursor_position(&mut self) {
        let line = self.cursor_line();
        let line_start = self.line_start(line);
        self.cursor_y = line as i16;
        self.cursor_x = (self.cursor_pos - line_start) as i16;
    }

    /// Ensure cursor is visible (scroll if needed)
    fn ensure_cursor_visible(&mut self) {
        let height = self.bounds.height() as usize;
        let width = self.bounds.width() as usize;

        // Vertical scrolling
        if self.cursor_y < self.top_line as i16 {
            self.top_line = self.cursor_y as usize;
        } else if self.cursor_y >= (self.top_line + height) as i16 {
            self.top_line = (self.cursor_y as usize).saturating_sub(height - 1);
        }

        // Horizontal scrolling
        if self.cursor_x < self.left_col as i16 {
            self.left_col = self.cursor_x as usize;
        } else if self.cursor_x >= (self.left_col + width) as i16 {
            self.left_col = (self.cursor_x as usize).saturating_sub(width - 1);
        }
    }
}
```

### Text Editing

```rust
impl Editor {
    /// Insert character at cursor
    pub fn insert_char(&mut self, ch: char) {
        // Delete selection if any
        if let Some(sel) = self.selection.take() {
            self.delete_selection(sel);
        }

        // Save for undo
        self.save_undo(UndoAction::Insert {
            pos: self.cursor_pos,
            text: ch.to_string(),
        });

        // Insert character
        self.buffer.insert_char(self.cursor_pos, ch);
        self.cursor_pos += 1;
        self.update_cursor_position();
        self.ensure_cursor_visible();
        self.modified = true;

        // Clear redo stack
        self.redo_stack.clear();
    }

    /// Insert text at cursor
    pub fn insert_text(&mut self, text: &str) {
        if let Some(sel) = self.selection.take() {
            self.delete_selection(sel);
        }

        self.save_undo(UndoAction::Insert {
            pos: self.cursor_pos,
            text: text.to_string(),
        });

        self.buffer.insert_text(self.cursor_pos, text);
        self.cursor_pos += text.len();
        self.update_cursor_position();
        self.ensure_cursor_visible();
        self.modified = true;
        self.redo_stack.clear();
    }

    /// Delete character before cursor (backspace)
    pub fn delete_char_before(&mut self) {
        if self.cursor_pos > 0 {
            // Save deleted char for undo
            let text = self.buffer.text();
            let deleted = text.chars().nth(self.cursor_pos - 1).unwrap();

            self.save_undo(UndoAction::Delete {
                pos: self.cursor_pos - 1,
                text: deleted.to_string(),
            });

            self.buffer.delete_char(self.cursor_pos - 1);
            self.cursor_pos -= 1;
            self.update_cursor_position();
            self.ensure_cursor_visible();
            self.modified = true;
            self.redo_stack.clear();
        }
    }

    /// Delete character at cursor (delete key)
    pub fn delete_char_at(&mut self) {
        if self.cursor_pos < self.buffer.len() {
            let text = self.buffer.text();
            let deleted = text.chars().nth(self.cursor_pos).unwrap();

            self.save_undo(UndoAction::Delete {
                pos: self.cursor_pos,
                text: deleted.to_string(),
            });

            self.buffer.delete_char(self.cursor_pos);
            self.modified = true;
            self.redo_stack.clear();
        }
    }

    /// Delete selection
    fn delete_selection(&mut self, sel: Selection) {
        let (start, end) = sel.range();
        let text = self.buffer.text();
        let deleted = text[start..end].to_string();

        self.save_undo(UndoAction::Delete {
            pos: start,
            text: deleted,
        });

        self.buffer.delete_range(start, end);
        self.cursor_pos = start;
        self.update_cursor_position();
        self.modified = true;
    }
}
```

---

## Undo and Redo

### Undo Action Types

```rust
pub enum UndoAction {
    Insert {
        pos: usize,
        text: String,
    },
    Delete {
        pos: usize,
        text: String,
    },
    Replace {
        pos: usize,
        old_text: String,
        new_text: String,
    },
}
```

### Implementing Undo

```rust
impl Editor {
    fn save_undo(&mut self, action: UndoAction) {
        self.undo_stack.push(action);

        // Limit undo stack size
        const MAX_UNDO: usize = 100;
        if self.undo_stack.len() > MAX_UNDO {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self) -> bool {
        if let Some(action) = self.undo_stack.pop() {
            match action {
                UndoAction::Insert { pos, text } => {
                    // Undo insert = delete
                    let end = pos + text.len();
                    self.buffer.delete_range(pos, end);
                    self.cursor_pos = pos;

                    // Save for redo
                    self.redo_stack.push(UndoAction::Insert { pos, text });
                }
                UndoAction::Delete { pos, text } => {
                    // Undo delete = insert
                    self.buffer.insert_text(pos, &text);
                    self.cursor_pos = pos + text.len();

                    // Save for redo
                    self.redo_stack.push(UndoAction::Delete { pos, text });
                }
                UndoAction::Replace { pos, old_text, new_text } => {
                    // Undo replace = replace back
                    let end = pos + new_text.len();
                    self.buffer.delete_range(pos, end);
                    self.buffer.insert_text(pos, &old_text);
                    self.cursor_pos = pos + old_text.len();

                    // Save for redo
                    self.redo_stack.push(UndoAction::Replace {
                        pos,
                        old_text: new_text,
                        new_text: old_text,
                    });
                }
            }

            self.update_cursor_position();
            self.ensure_cursor_visible();
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(action) = self.redo_stack.pop() {
            match action {
                UndoAction::Insert { pos, text } => {
                    self.buffer.insert_text(pos, &text);
                    self.cursor_pos = pos + text.len();
                    self.undo_stack.push(UndoAction::Insert { pos, text });
                }
                UndoAction::Delete { pos, text } => {
                    let end = pos + text.len();
                    self.buffer.delete_range(pos, end);
                    self.cursor_pos = pos;
                    self.undo_stack.push(UndoAction::Delete { pos, text });
                }
                UndoAction::Replace { pos, old_text, new_text } => {
                    let end = pos + old_text.len();
                    self.buffer.delete_range(pos, end);
                    self.buffer.insert_text(pos, &new_text);
                    self.cursor_pos = pos + new_text.len();
                    self.undo_stack.push(UndoAction::Replace {
                        pos,
                        old_text,
                        new_text,
                    });
                }
            }

            self.update_cursor_position();
            self.ensure_cursor_visible();
            true
        } else {
            false
        }
    }
}
```

---

## Clipboard Operations

### Selection Management

```rust
impl Editor {
    /// Start selection at cursor
    pub fn start_selection(&mut self) {
        self.selection = Some(Selection {
            start: self.cursor_pos,
            end: self.cursor_pos,
        });
    }

    /// Extend selection to cursor
    pub fn extend_selection(&mut self) {
        if let Some(ref mut sel) = self.selection {
            sel.end = self.cursor_pos;
        }
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    /// Get selected text
    pub fn selected_text(&self) -> Option<String> {
        self.selection.as_ref().map(|sel| {
            let (start, end) = sel.range();
            let text = self.buffer.text();
            text[start..end].to_string()
        })
    }
}
```

### Clipboard Operations

```rust
use clipboard::{ClipboardProvider, ClipboardContext};

impl Editor {
    /// Copy selection to clipboard
    pub fn copy(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(text) = self.selected_text() {
            let mut ctx: ClipboardContext = ClipboardProvider::new()?;
            ctx.set_contents(text)?;
        }
        Ok(())
    }

    /// Cut selection to clipboard
    pub fn cut(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(text) = self.selected_text() {
            let mut ctx: ClipboardContext = ClipboardProvider::new()?;
            ctx.set_contents(text)?;

            // Delete selection
            if let Some(sel) = self.selection.take() {
                self.delete_selection(sel);
            }
        }
        Ok(())
    }

    /// Paste from clipboard
    pub fn paste(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut ctx: ClipboardContext = ClipboardProvider::new()?;
        if let Ok(text) = ctx.get_contents() {
            self.insert_text(&text);
        }
        Ok(())
    }

    /// Delete selection
    pub fn delete(&mut self) {
        if let Some(sel) = self.selection.take() {
            self.delete_selection(sel);
        }
    }
}
```

---

## Search and Replace

### Search Implementation

```rust
pub struct SearchOptions {
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub regex: bool,
}

impl Editor {
    /// Find text in buffer
    pub fn find(
        &self,
        pattern: &str,
        start: usize,
        options: &SearchOptions,
    ) -> Option<(usize, usize)> {
        let text = self.buffer.text();

        if options.regex {
            self.find_regex(pattern, start, &text)
        } else {
            self.find_literal(pattern, start, &text, options)
        }
    }

    fn find_literal(
        &self,
        pattern: &str,
        start: usize,
        text: &str,
        options: &SearchOptions,
    ) -> Option<(usize, usize)> {
        let search_text = if options.case_sensitive {
            text[start..].to_string()
        } else {
            text[start..].to_lowercase()
        };

        let search_pattern = if options.case_sensitive {
            pattern.to_string()
        } else {
            pattern.to_lowercase()
        };

        if let Some(pos) = search_text.find(&search_pattern) {
            let abs_pos = start + pos;
            let end = abs_pos + pattern.len();

            // Check whole word
            if options.whole_word {
                if !self.is_word_boundary(text, abs_pos) ||
                   !self.is_word_boundary(text, end) {
                    // Not a whole word, continue searching
                    return self.find_literal(pattern, end, text, options);
                }
            }

            Some((abs_pos, end))
        } else {
            None
        }
    }

    fn find_regex(
        &self,
        pattern: &str,
        start: usize,
        text: &str,
    ) -> Option<(usize, usize)> {
        use regex::Regex;

        if let Ok(re) = Regex::new(pattern) {
            if let Some(mat) = re.find(&text[start..]) {
                let abs_start = start + mat.start();
                let abs_end = start + mat.end();
                return Some((abs_start, abs_end));
            }
        }

        None
    }

    fn is_word_boundary(&self, text: &str, pos: usize) -> bool {
        if pos == 0 || pos >= text.len() {
            return true;
        }

        let before = text.chars().nth(pos - 1).unwrap();
        let after = text.chars().nth(pos).unwrap();

        !before.is_alphanumeric() || !after.is_alphanumeric()
    }

    /// Find next occurrence
    pub fn find_next(
        &mut self,
        pattern: &str,
        options: &SearchOptions,
    ) -> bool {
        if let Some((start, end)) = self.find(pattern, self.cursor_pos, options) {
            self.cursor_pos = start;
            self.selection = Some(Selection { start, end });
            self.update_cursor_position();
            self.ensure_cursor_visible();
            true
        } else {
            // Wrap to beginning
            if let Some((start, end)) = self.find(pattern, 0, options) {
                self.cursor_pos = start;
                self.selection = Some(Selection { start, end });
                self.update_cursor_position();
                self.ensure_cursor_visible();
                true
            } else {
                false
            }
        }
    }

    /// Replace current selection
    pub fn replace(&mut self, replacement: &str) {
        if let Some(sel) = self.selection.take() {
            let (start, end) = sel.range();
            let old_text = self.buffer.text()[start..end].to_string();

            self.save_undo(UndoAction::Replace {
                pos: start,
                old_text,
                new_text: replacement.to_string(),
            });

            self.buffer.delete_range(start, end);
            self.buffer.insert_text(start, replacement);
            self.cursor_pos = start + replacement.len();
            self.update_cursor_position();
            self.ensure_cursor_visible();
            self.modified = true;
        }
    }

    /// Replace all occurrences
    pub fn replace_all(
        &mut self,
        pattern: &str,
        replacement: &str,
        options: &SearchOptions,
    ) -> usize {
        let mut count = 0;
        let mut pos = 0;

        while let Some((start, end)) = self.find(pattern, pos, options) {
            self.buffer.delete_range(start, end);
            self.buffer.insert_text(start, replacement);
            pos = start + replacement.len();
            count += 1;
        }

        if count > 0 {
            self.modified = true;
        }

        count
    }
}
```

---

## Memo Field

### What is a Memo?

A **Memo** is a multi-line input control for dialogs:

```rust
pub struct Memo {
    bounds: Rect,
    data: Rc<RefCell<String>>,
    lines: Vec<String>,
    cursor_line: usize,
    cursor_col: usize,
    top_line: usize,
    state: StateFlags,
}
```

### Creating a Memo

```rust
impl Memo {
    pub fn new(
        bounds: Rect,
        data: Rc<RefCell<String>>,
    ) -> Self {
        let lines = data.borrow().lines()
            .map(|s| s.to_string())
            .collect();

        Self {
            bounds,
            data,
            lines,
            cursor_line: 0,
            cursor_col: 0,
            top_line: 0,
            state: SF_VISIBLE | SF_SELECTABLE,
        }
    }

    fn sync_to_data(&mut self) {
        *self.data.borrow_mut() = self.lines.join("\n");
    }

    fn sync_from_data(&mut self) {
        self.lines = self.data.borrow().lines()
            .map(|s| s.to_string())
            .collect();
    }
}
```

### Memo Event Handling

```rust
impl View for Memo {
    fn handle_event(&mut self, event: &mut Event) {
        if event.what == EventType::Keyboard {
            match event.key_code {
                KB_UP => {
                    if self.cursor_line > 0 {
                        self.cursor_line -= 1;
                        self.cursor_col = self.cursor_col.min(
                            self.lines[self.cursor_line].len()
                        );
                    }
                    event.clear();
                }
                KB_DOWN => {
                    if self.cursor_line < self.lines.len() - 1 {
                        self.cursor_line += 1;
                        self.cursor_col = self.cursor_col.min(
                            self.lines[self.cursor_line].len()
                        );
                    }
                    event.clear();
                }
                KB_LEFT => {
                    if self.cursor_col > 0 {
                        self.cursor_col -= 1;
                    }
                    event.clear();
                }
                KB_RIGHT => {
                    if self.cursor_col < self.lines[self.cursor_line].len() {
                        self.cursor_col += 1;
                    }
                    event.clear();
                }
                KB_ENTER => {
                    // Split line at cursor
                    let line = &self.lines[self.cursor_line];
                    let before = line[..self.cursor_col].to_string();
                    let after = line[self.cursor_col..].to_string();

                    self.lines[self.cursor_line] = before;
                    self.lines.insert(self.cursor_line + 1, after);
                    self.cursor_line += 1;
                    self.cursor_col = 0;

                    self.sync_to_data();
                    event.clear();
                }
                KB_BACK => {
                    if self.cursor_col > 0 {
                        self.lines[self.cursor_line].remove(self.cursor_col - 1);
                        self.cursor_col -= 1;
                        self.sync_to_data();
                    } else if self.cursor_line > 0 {
                        // Join with previous line
                        let line = self.lines.remove(self.cursor_line);
                        self.cursor_line -= 1;
                        self.cursor_col = self.lines[self.cursor_line].len();
                        self.lines[self.cursor_line].push_str(&line);
                        self.sync_to_data();
                    }
                    event.clear();
                }
                _ => {
                    if let Some(ch) = key_to_char(event.key_code) {
                        self.lines[self.cursor_line].insert(self.cursor_col, ch);
                        self.cursor_col += 1;
                        self.sync_to_data();
                        event.clear();
                    }
                }
            }
        }
    }
}
```

### Memo Usage Example

```rust
// Create shared data
let notes_data = Rc::new(RefCell::new(String::from(
    "Enter your notes here.\n\
     Multiple lines are supported."
)));

// Create memo in dialog
let mut dialog = Dialog::new(
    Rect::new(10, 5, 70, 20),
    "Notes"
);

dialog.add(Box::new(Label::new(
    Rect::new(2, 2, 12, 3),
    "~N~otes:"
)));

dialog.add(Box::new(Memo::new(
    Rect::new(2, 3, 56, 13),
    notes_data.clone()
)));

// After dialog closes
if dialog.execute(&mut app) == CM_OK {
    println!("Notes: {}", notes_data.borrow());
}
```

---

## File Editor

### FileEditor Structure

```rust
pub struct FileEditor {
    editor: Editor,
    file_path: Option<PathBuf>,
}

impl FileEditor {
    pub fn new(bounds: Rect) -> Self {
        Self {
            editor: Editor::new(bounds),
            file_path: None,
        }
    }

    pub fn load_file(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        self.editor.buffer = EditorBuffer::with_text(&contents);
        self.editor.cursor_pos = 0;
        self.editor.update_cursor_position();
        self.editor.modified = false;
        self.file_path = Some(path.to_path_buf());
        Ok(())
    }

    pub fn save_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref path) = self.file_path {
            std::fs::write(path, self.editor.text())?;
            self.editor.modified = false;
            Ok(())
        } else {
            Err("No file path set".into())
        }
    }

    pub fn save_as(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::write(path, self.editor.text())?;
        self.file_path = Some(path.to_path_buf());
        self.editor.modified = false;
        Ok(())
    }

    pub fn file_name(&self) -> String {
        self.file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled")
            .to_string()
    }
}
```

---

## Editor Window

### EditorWindow Structure

```rust
pub struct EditorWindow {
    window: Window,
    editor: FileEditor,
}

impl EditorWindow {
    pub fn new(bounds: Rect, file_path: Option<&Path>) -> Self {
        let title = file_path
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled");

        let mut window = Window::new(bounds, title);

        let interior_bounds = window.interior_bounds();
        let mut editor = FileEditor::new(interior_bounds);

        if let Some(path) = file_path {
            if let Err(e) = editor.load_file(path) {
                eprintln!("Failed to load file: {}", e);
            }
        }

        Self { window, editor }
    }

    pub fn handle_command(&mut self, command: u16, app: &mut Application) -> bool {
        match command {
            CM_SAVE => {
                if let Err(e) = self.editor.save_file() {
                    MessageBox::error(&format!("Save failed: {}", e)).show(app);
                }
                true
            }
            CM_SAVE_AS => {
                if let Some(path) = self.show_save_as_dialog(app) {
                    if let Err(e) = self.editor.save_as(&path) {
                        MessageBox::error(&format!("Save failed: {}", e)).show(app);
                    } else {
                        self.window.set_title(&self.editor.file_name());
                    }
                }
                true
            }
            CM_UNDO => {
                self.editor.editor.undo();
                true
            }
            CM_REDO => {
                self.editor.editor.redo();
                true
            }
            CM_CUT => {
                if let Err(e) = self.editor.editor.cut() {
                    MessageBox::error(&format!("Cut failed: {}", e)).show(app);
                }
                true
            }
            CM_COPY => {
                if let Err(e) = self.editor.editor.copy() {
                    MessageBox::error(&format!("Copy failed: {}", e)).show(app);
                }
                true
            }
            CM_PASTE => {
                if let Err(e) = self.editor.editor.paste() {
                    MessageBox::error(&format!("Paste failed: {}", e)).show(app);
                }
                true
            }
            CM_FIND => {
                self.show_find_dialog(app);
                true
            }
            CM_REPLACE => {
                self.show_replace_dialog(app);
                true
            }
            _ => false,
        }
    }

    fn show_save_as_dialog(&mut self, app: &mut Application) -> Option<PathBuf> {
        FileDialog::new("Save As", "*.txt").execute(app)
    }

    fn show_find_dialog(&mut self, app: &mut Application) {
        // Show find dialog (implementation omitted for brevity)
    }

    fn show_replace_dialog(&mut self, app: &mut Application) {
        // Show replace dialog (implementation omitted for brevity)
    }
}
```

---

## Complete Examples

### Example 1: Simple Text Editor

```rust
use turbo_vision::prelude::*;
use std::path::Path;

pub struct SimpleEditor {
    app: Application,
}

impl SimpleEditor {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let app = Application::new()?;
        Ok(Self { app })
    }

    pub fn run(&mut self, file_path: Option<&Path>) -> Result<(), Box<dyn std::error::Error>> {
        // Create editor window
        let editor_window = EditorWindow::new(
            Rect::new(5, 2, 75, 22),
            file_path,
        );

        self.app.desktop.add(Box::new(editor_window));

        self.app.run()?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let file_path = if args.len() > 1 {
        Some(Path::new(&args[1]))
    } else {
        None
    };

    let mut editor = SimpleEditor::new()?;
    editor.run(file_path)?;

    Ok(())
}
```

### Example 2: Build Output Terminal

```rust
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};

pub struct BuildWindow {
    window: Window,
    terminal: Terminal,
}

impl BuildWindow {
    pub fn new(bounds: Rect) -> Self {
        let mut window = Window::new(bounds, "Build Output");

        let terminal_bounds = window.interior_bounds();
        let terminal = Terminal::new(terminal_bounds, 1000);

        Self { window, terminal }
    }

    pub fn run_build(&mut self, command: &str) {
        self.terminal.clear();
        self.terminal.writeln(&format!("$ {}", command));
        self.terminal.writeln("");

        // Run command
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        let child = Command::new(parts[0])
            .args(&parts[1..])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        if let Ok(mut child) = child {
            if let Some(stdout) = child.stdout.take() {
                let reader = BufReader::new(stdout);
                for line in reader.lines().flatten() {
                    self.terminal.writeln(&line);
                }
            }

            if let Ok(status) = child.wait() {
                self.terminal.writeln("");
                if status.success() {
                    self.terminal.writeln("Build succeeded");
                } else {
                    self.terminal.writeln("Build failed");
                }
            }
        } else {
            self.terminal.writeln("Failed to start command");
        }
    }
}

// Usage:
let mut build_window = BuildWindow::new(Rect::new(5, 3, 75, 20));
build_window.run_build("cargo build --release");
```

### Example 3: Multi-line Notes Dialog

```rust
pub fn show_notes_dialog(app: &mut Application) -> Option<String> {
    let notes_data = Rc::new(RefCell::new(String::new()));

    let mut dialog = Dialog::new(
        Rect::new(10, 5, 70, 18),
        "Notes"
    );

    // Label
    dialog.add(Box::new(Label::new(
        Rect::new(2, 2, 58, 3),
        "Enter your notes:"
    )));

    // Memo field
    dialog.add(Box::new(Memo::new(
        Rect::new(2, 3, 56, 11),
        notes_data.clone()
    )));

    // OK button
    dialog.add(Box::new(Button::new(
        Rect::new(20, 11, 30, 13),
        "~O~K",
        CM_OK,
        true
    )));

    // Cancel button
    dialog.add(Box::new(Button::new(
        Rect::new(32, 11, 42, 13),
        "~C~ancel",
        CM_CANCEL,
        false
    )));

    let result = dialog.execute(app);

    if result == CM_OK {
        Some(notes_data.borrow().clone())
    } else {
        None
    }
}

// Usage:
if let Some(notes) = show_notes_dialog(&mut app) {
    println!("Notes: {}", notes);
}
```

---

## Best Practices

### 1. Use Buffer Gap for Large Texts

```rust
// ✓ Good - buffer gap for editing
let editor = Editor::with_text(bounds, &large_text);

// ✗ Bad - String for large edits
// (Reallocates on every insertion)
```

### 2. Limit Undo Stack

```rust
// ✓ Good - limit undo history
const MAX_UNDO: usize = 100;
if self.undo_stack.len() > MAX_UNDO {
    self.undo_stack.remove(0);
}

// ✗ Bad - unlimited undo
// (Memory leak with large edits)
```

### 3. Clear Redo on Edit

```rust
// ✓ Good - clear redo after edit
self.insert_char(ch);
self.redo_stack.clear();  // Can't redo after new edit

// ✗ Bad - keep stale redo stack
// (Confusing redo behavior)
```

### 4. Save Before Close

```rust
// ✓ Good - prompt if modified
impl EditorWindow {
    pub fn close(&mut self, app: &mut Application) -> bool {
        if self.editor.editor.is_modified() {
            let result = MessageBox::new(
                "Unsaved Changes",
                "Save changes before closing?",
                MessageBoxButtons::YesNoCancel
            ).show(app);

            match result {
                CM_YES => {
                    if let Err(e) = self.editor.save_file() {
                        MessageBox::error(&format!("Save failed: {}", e)).show(app);
                        return false;
                    }
                    true
                }
                CM_NO => true,
                _ => false,
            }
        } else {
            true
        }
    }
}
```

### 5. Use Terminal for Append-Only

```rust
// ✓ Good - Terminal for logs
let terminal = Terminal::new(bounds, 1000);
terminal.writeln("Build started");

// ✗ Bad - Editor for logs
// (Unnecessary editing overhead)
```

---

## Pascal vs. Rust Summary

| Concept | Pascal | Rust |
|---------|--------|------|
| **Terminal** | `TTerminal` with `AssignDevice` | `Terminal` with `writeln()` |
| **Buffer** | Pointer `BufPtr`, manual `New` | `EditorBuffer` with `Vec<u8>` |
| **Gap** | `GapStart`, `GapEnd` pointers | `gap_start`, `gap_end` indices |
| **Undo** | Linked list `PUndoRec` | `Vec<UndoAction>` |
| **Clipboard** | `GetClipboardData` | `clipboard` crate |
| **Selection** | `SetSelect(start, end)` | `Selection { start, end }` |
| **File I/O** | `TDosStream` | `std::fs` |
| **Memo** | `TMemo = object(TEditor)` | `Memo` struct (composition) |
| **Editor Window** | `TEditWindow = object(TWindow)` | `EditorWindow` struct |

---

## Summary

### Key Concepts

1. **Terminal View** - Write-only scrolling text for logging
2. **Buffer Gap** - Efficient insertion/deletion at cursor (O(1))
3. **Editor** - Full text editing with undo/redo
4. **Undo/Redo** - Action-based undo with Vec stack
5. **Clipboard** - System clipboard integration
6. **Search/Replace** - Literal and regex search
7. **Memo Field** - Multi-line input control for dialogs
8. **File Editor** - Load/save file operations
9. **Editor Window** - Window container with menu commands

### The Editor Pattern

```rust
// 1. Create editor
let mut editor = Editor::with_text(bounds, initial_text);

// 2. Handle keyboard events
editor.handle_event(&mut event);

// 3. Edit operations
editor.insert_char('a');
editor.delete_char_before();
editor.undo();
editor.redo();

// 4. Clipboard
editor.copy()?;
editor.cut()?;
editor.paste()?;

// 5. Search
let found = editor.find_next("pattern", &options);

// 6. Get text
let text = editor.text();
```

---

## See Also

- **Chapter 8** - Views and Groups (Drawing)
- **Chapter 9** - Event-Driven Programming (Keyboard handling)
- **Chapter 11** - Windows and Dialogs (Editor window)
- **Chapter 12** - Control Objects (Memo field)
- **docs/TURBOVISION-DESIGN.md** - Implementation details
- **examples/editor_demo.rs** - Editor examples

---

**Note:** This chapter focuses on Terminal and Editor views. The original Pascal Chapter 15 also covered Collections and Streams, which are separate architectural topics that may be covered in future chapters on data structures and serialization.

---

Editors are powerful components for text manipulation in Turbo Vision. Master the buffer gap architecture and undo/redo mechanisms to build professional text editing features.
