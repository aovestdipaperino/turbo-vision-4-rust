# Chapter 15: Editor and Text Views

## Overview

This chapter covers Turbo Vision's text editing components, from simple terminal views to full-featured editors with syntax highlighting, search/replace, and file operations.

## The Terminal View

The Terminal type provides a write-only scrolling view for displaying text output. It's most useful for debugging, logging, or monitoring text streams. Terminal views are defined in `src/terminal/mod.rs`.

### Basic Terminal Usage

Terminal objects manage text display with automatic buffering and rendering. The terminal maintains an internal buffer of cells (character + color attribute pairs) and efficiently updates only the changed portions of the screen.

### Key Terminal Features

- **Character Cell Buffer**: Each position stores a character and color attributes
- **Efficient Rendering**: Only changed cells are redrawn
- **Event Handling**: Processes keyboard and mouse input
- **Clipping Regions**: Supports nested clipping for view hierarchies

### Terminal Example

```rust
use turbo_vision::app::Application;
use turbo_vision::terminal::Terminal;

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;
    // Terminal is created and managed by Application
    // Views write to it via the draw() method
    app.run()
}
```

## The Editor Object

The `Editor` struct in `src/views/editor.rs` implements a powerful multi-line text editor with:

- Unlimited undo/redo (configurable history size)
- Clipboard operations (cut, copy, paste)
- Text selection with keyboard and mouse
- Find and replace with options
- Auto-indent and tab handling
- Insert/overwrite modes
- Syntax highlighting support
- Scroll bars and status indicators

### How the Editor Works

Unlike traditional fixed-size buffers, the Rust editor stores text as a `Vec<String>` where each string represents one line. This approach:

- Handles files of any size (limited only by memory)
- Supports UTF-8 text natively
- Simplifies line-based operations
- Integrates naturally with Rust's string handling

### Editor Buffer Structure

The editor maintains these key fields (see `src/views/editor.rs:79-103`):

```rust
pub struct Editor {
    lines: Vec<String>,           // Text content, one line per entry
    cursor: Point,                 // Current cursor position (x=col, y=line)
    delta: Point,                  // Scroll offset
    selection_start: Option<Point>, // Selection anchor
    undo_stack: Vec<EditAction>,   // Undo history
    redo_stack: Vec<EditAction>,   // Redo history
    insert_mode: bool,             // Insert vs overwrite
    auto_indent: bool,             // Auto-indent on newline
    // ... other fields
}
```

### Creating an Editor

```rust
use turbo_vision::views::editor::Editor;
use turbo_vision::core::geometry::Rect;

// Basic editor
let editor = Editor::new(Rect::new(0, 0, 80, 24));

// Editor with scrollbars and indicator
let editor = Editor::new(Rect::new(0, 0, 80, 24))
    .with_scrollbars_and_indicator();
```

### Setting Editor Options

```rust
// Configure editor behavior
editor.set_tab_size(4);
editor.set_auto_indent(true);
editor.set_read_only(false);

// Enable syntax highlighting
use turbo_vision::views::syntax::RustHighlighter;
editor.set_highlighter(Box::new(RustHighlighter::new()));
```

### Undo and Redo

The editor maintains full undo/redo support through an action stack (see `src/views/editor.rs:53-76`):

```rust
// Undo last edit
editor.undo();

// Redo last undone edit
editor.redo();
```

**Edit Actions**: Each editing operation (insert character, delete text, etc.) is recorded as an `EditAction` enum variant. The undo system:

- Stores up to `MAX_UNDO_HISTORY` actions (default: 100)
- Clears the redo stack when a new edit is made
- Supports inverting any action for undo

### Text Selection

Selection is managed through `selection_start` and `cursor` points:

```rust
// Select all text
editor.select_all();

// Get selected text
if let Some(text) = editor.get_selection() {
    println!("Selected: {}", text);
}

// Clear selection
editor.selection_start = None;
```

### Clipboard Operations

The editor integrates with the system clipboard via `src/core/clipboard.rs`:

```rust
use turbo_vision::core::clipboard;

// Copy selection to clipboard
if let Some(selection) = editor.get_selection() {
    clipboard::set_clipboard(&selection);
}

// Paste from clipboard
let text = clipboard::get_clipboard();
if !text.is_empty() {
    editor.insert_text(&text);
}
```

**Keyboard Shortcuts**:
- `Ctrl+C`: Copy
- `Ctrl+X`: Cut
- `Ctrl+V`: Paste
- `Ctrl+A`: Select All
- `Ctrl+Z`: Undo
- `Ctrl+Y`: Redo

## Search and Replace

The editor provides comprehensive search and replace functionality matching the original Turbo Vision API (see `src/views/editor.rs:296-470`).

### Search Options

```rust
use turbo_vision::views::editor::SearchOptions;

let options = SearchOptions {
    case_sensitive: false,
    whole_words_only: false,
    backwards: false,
};
```

### Finding Text

```rust
// Find first occurrence
if let Some(pos) = editor.find("search_term", options) {
    println!("Found at {:?}", pos);
}

// Find next occurrence
if let Some(pos) = editor.find_next() {
    println!("Next match at {:?}", pos);
}
```

The search automatically:
- Wraps around to the beginning when reaching the end
- Highlights matches by setting selection
- Scrolls the view to make matches visible

### Replace Operations

```rust
// Replace current selection
if editor.replace_selection("new_text") {
    println!("Replaced");
}

// Replace next occurrence
if editor.replace_next("find", "replace", options) {
    println!("Replaced next");
}

// Replace all occurrences
let count = editor.replace_all("find", "replace", options);
println!("Replaced {} occurrences", count);
```

## Scroll Bars and Indicators

Editors can have optional scrollbars and status indicators:

```rust
// Scrollbars automatically track:
// - Content size (lines and max line length)
// - Viewport size (editor dimensions)
// - Current position (delta/scroll offset)

// Indicators show:
// - Current line and column
// - Modified flag
// - Insert/overwrite mode
```

The editor automatically updates scrollbars and indicators when:
- Text is edited
- Cursor moves
- View is resized

## Syntax Highlighting

The editor supports pluggable syntax highlighting via the `SyntaxHighlighter` trait (see `src/views/syntax.rs`).

### Using Syntax Highlighting

```rust
use turbo_vision::views::syntax::RustHighlighter;

let mut editor = Editor::new(bounds)
    .with_scrollbars_and_indicator();

// Enable Rust syntax highlighting
editor.set_highlighter(Box::new(RustHighlighter::new()));

// Disable highlighting
editor.clear_highlighter();
```

### Implementing Custom Highlighters

Create a custom highlighter by implementing the `SyntaxHighlighter` trait:

```rust
use turbo_vision::views::syntax::{SyntaxHighlighter, Token, TokenType};

struct MyHighlighter;

impl SyntaxHighlighter for MyHighlighter {
    fn highlight_line(&self, line: &str, _line_num: usize) -> Vec<Token> {
        // Return tokens with positions and types
        // The editor will color them appropriately
        vec![
            Token {
                start: 0,
                end: line.len(),
                token_type: TokenType::Text,
            }
        ]
    }
}
```

## The Memo Control

The `Memo` struct (`src/views/memo.rs`) is a simplified editor designed for use in dialog boxes. It provides basic text editing without undo/redo or advanced features.

### Memo vs Editor

**Memo** is designed for:
- Simple text input in dialogs
- Limited editing needs
- Smaller memory footprint
- Integration with data transfer

**Editor** is designed for:
- Full-featured text editing
- Undo/redo support
- Syntax highlighting
- File editing

### Using a Memo

```rust
use turbo_vision::views::memo::Memo;

// Create memo with optional scrollbars
let mut memo = Memo::new(Rect::new(5, 3, 45, 10))
    .with_scrollbars(true);

// Set properties
memo.set_max_length(Some(500));  // Limit characters
memo.set_read_only(false);
memo.set_tab_size(4);

// Get/set text
memo.set_text("Initial content");
let text = memo.get_text();
```

## File Editors

The `FileEditor` struct (`src/views/file_editor.rs`) extends `Editor` with file management:

- File name tracking
- Load/save operations
- Modified flag handling
- Save prompt on close

### Using File Editor

```rust
use turbo_vision::views::file_editor::FileEditor;
use std::path::PathBuf;

// Create file editor
let mut editor = FileEditor::new(Rect::new(0, 0, 80, 24));

// Load a file
editor.load_file(PathBuf::from("example.rs"))?;

// Make edits...
editor.editor_mut().set_text("Modified content");

// Save
if editor.is_modified() {
    editor.save()?;  // Save to current file

    // Or save as new file
    editor.save_as(PathBuf::from("new_file.rs"))?;
}
```

### File Name and Title

```rust
// Get filename
if let Some(path) = editor.filename() {
    println!("Editing: {:?}", path);
}

// Get display title ("Untitled" if new file)
let title = editor.get_title();
```

### Save Confirmation

The `FileEditor::valid()` method prompts for save confirmation when closing with unsaved changes:

```rust
use turbo_vision::core::command::CM_CLOSE;

// Check if close is allowed (prompts if modified)
if editor.valid(app, CM_CLOSE) {
    // User chose to save, discard, or already saved
    // Safe to close
} else {
    // User cancelled
}
```

## Editor Key Bindings

The editor supports these keyboard shortcuts (see `src/views/editor.rs:1174-1305`):

### Navigation
- Arrow keys: Move cursor
- `Home`: Move to line start
- `End`: Move to line end
- `PgUp`/`PgDn`: Scroll page up/down
- `Shift` + navigation: Extend selection

### Editing
- `Enter`: Insert newline (with auto-indent if enabled)
- `Backspace`: Delete character before cursor
- `Delete`: Delete character at cursor
- `Tab`: Insert tab (spaces)
- `Insert`: Toggle insert/overwrite mode (if supported)

### Clipboard
- `Ctrl+X`: Cut selection
- `Ctrl+C`: Copy selection
- `Ctrl+V`: Paste
- `Ctrl+A`: Select all

### Undo/Redo
- `Ctrl+Z`: Undo last action
- `Ctrl+Y`: Redo last undone action

## Editor Configuration

### Read-Only Mode

```rust
editor.set_read_only(true);
// User can view and select but not edit
```

### Tab Size

```rust
// Set number of spaces for tab
editor.set_tab_size(2);  // 2 spaces
editor.set_tab_size(4);  // 4 spaces (default)
```

### Auto-Indent

```rust
// Enable auto-indent on newline
editor.set_auto_indent(true);
// Pressing Enter indents to match previous line
```

### Modified Flag

```rust
// Check if modified
if editor.is_modified() {
    println!("File has unsaved changes");
}

// Clear flag (after save)
editor.clear_modified();
```

## Drawing and Rendering

Editors implement the `View` trait and handle drawing automatically (see `src/views/editor.rs:1030-1172`):

```rust
impl View for Editor {
    fn draw(&mut self, terminal: &mut Terminal) {
        // Renders visible portion of text
        // Applies syntax highlighting if enabled
        // Draws selection highlight
        // Shows cursor
        // Updates scrollbars and indicator
    }
}
```

The rendering process:
1. Calculates visible text area (content area minus scrollbars/indicator)
2. Renders visible lines with syntax highlighting
3. Applies selection highlighting
4. Draws cursor if focused
5. Updates child views (scrollbars, indicator)

## Complete Example

Here's a complete example combining all editor features:

```rust
use turbo_vision::app::Application;
use turbo_vision::views::{
    window::Window,
    editor::Editor,
    syntax::RustHighlighter,
    view::View,
};
use turbo_vision::core::geometry::Rect;

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;

    // Create window using the builder pattern
    let mut window = WindowBuilder::new()
        .bounds(Rect::new(5, 2, 75, 22))
        .title("Editor Demo")
        .build();

    // Create editor with all features
    let mut editor = Editor::new(Rect::new(1, 1, 68, 18))
        .with_scrollbars_and_indicator();

    // Configure editor
    editor.set_tab_size(4);
    editor.set_auto_indent(true);
    editor.set_highlighter(Box::new(RustHighlighter::new()));

    // Set initial content
    editor.set_text(
        "fn main() {\n\
         \    println!(\"Hello, Turbo Vision!\");\n\
         }\n"
    );

    // Add to window
    window.add(Box::new(editor));

    // Show window
    app.insert_window(Box::new(window));

    // Run application
    app.run()
}
```

See the `examples/editor_demo.rs` file for a comprehensive demonstration of all editor features including search, replace, file operations, and syntax highlighting.

## Summary

Turbo Vision provides a complete hierarchy of text editing components:

- **Terminal**: Low-level display and event handling
- **Memo**: Simple text input for dialogs
- **Editor**: Full-featured text editor with undo/redo, search/replace, and syntax highlighting
- **FileEditor**: Editor with file management and save prompts

All components:
- Support UTF-8 text natively
- Integrate with the clipboard
- Handle keyboard and mouse input
- Work within Turbo Vision's view hierarchy
- Follow Rust ownership and borrowing rules

For file editing, use `FileEditor`. For text input in dialogs, use `Memo`. For custom text display, work directly with `Terminal` and the drawing system.
