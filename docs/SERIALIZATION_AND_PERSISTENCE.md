# Turbo Vision Rust Implementation: Serialization, Resources, and Persistence

## Executive Summary

The Rust implementation of Turbo Vision does **NOT** use any serialization frameworks like serde. Instead, it provides:
1. **Resource definitions** through Rust structures (Menu, MenuItem, StatusDef, StatusItem)
2. **File I/O operations** for editors using standard Rust `std::fs`
3. **In-memory state management** with no built-in persistence layer
4. **History management** for input fields (similar to Borland's THistory system)

---

## 1. Resource Files and Data Structures

### 1.1 Menu Resources (`src/core/menu_data.rs`)

**Location**: `/Users/enzo/Code/turbo-vision/src/core/menu_data.rs` (356 lines)

The menu system defines resources declaratively using Rust enums and structs:

```rust
#[derive(Clone, Debug)]
pub enum MenuItem {
    Regular {
        text: String,
        command: CommandId,
        key_code: KeyCode,
        help_ctx: u16,
        enabled: bool,
        shortcut: Option<String>,
    },
    SubMenu {
        text: String,
        key_code: KeyCode,
        help_ctx: u16,
        menu: Menu,
    },
    Separator,
}

pub struct Menu {
    pub items: Vec<MenuItem>,
    pub default_index: Option<usize>,
}
```

**Key Features:**
- Enum-based design for type-safe menu items
- Support for regular items, submenus, and separators
- Keyboard shortcuts and help context
- Default item tracking
- Builder pattern for fluent API (MenuBuilder)

**Example Usage** (from examples):
```rust
let file_menu = Menu::from_items(vec![
    MenuItem::new("~O~pen", CM_OPEN, KB_F3, 0),
    MenuItem::new("~S~ave", CM_SAVE, KB_F2, 0),
    MenuItem::separator(),
    MenuItem::new("E~x~it", CM_QUIT, KB_ALT_X, 0),
]);
```

### 1.2 Status Line Resources (`src/core/status_data.rs`)

**Location**: `/Users/enzo/Code/turbo-vision/src/core/status_data.rs` (268 lines)

Status line defines resources for the bottom line of the application:

```rust
#[derive(Clone, Debug)]
pub struct StatusItem {
    pub text: String,
    pub key_code: KeyCode,
    pub command: CommandId,
}

#[derive(Clone, Debug)]
pub struct StatusDef {
    pub min: u16,  // Min command range
    pub max: u16,  // Max command range
    pub items: Vec<StatusItem>,
}

#[derive(Clone, Debug)]
pub struct StatusLine {
    pub defs: Vec<StatusDef>,
}
```

**Key Features:**
- Command-range based definitions (different items for different command contexts)
- Support for help text and keyboard shortcuts
- Multiple definitions can apply to different command ranges
- Builder pattern (StatusLineBuilder)

**Example Usage**:
```rust
let status = StatusLineBuilder::new()
    .add_def(0, 0xFFFF, vec![
        StatusItem::new("~F1~ Help", KB_F1, CM_HELP),
        StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
    ])
    .build();
```

---

## 2. File Serialization and I/O

### 2.1 Editor File Operations (`src/views/editor.rs`)

**Location**: `/Users/enzo/Code/turbo-vision/src/views/editor.rs`

The Editor provides basic file I/O using standard Rust file operations:

```rust
pub fn load_file(&mut self, path: &str) -> std::io::Result<()> {
    let content = std::fs::read_to_string(path)?;
    self.set_text(&content);
    self.filename = Some(path.to_string());
    self.modified = false;
    self.undo_stack.clear();
    self.redo_stack.clear();
    self.update_indicator();
    Ok(())
}

pub fn save_as(&mut self, path: &str) -> std::io::Result<()> {
    let content = self.get_text();
    std::fs::write(path, content)?;
    self.filename = Some(path.to_string());
    self.modified = false;
    self.update_indicator();
    Ok(())
}
```

**Key Features:**
- Plain text file reading/writing
- Modified flag tracking
- Filename tracking
- Undo/redo stack management
- Works with UTF-8 text

**Dependencies**: Uses only `std::fs::read_to_string()` and `std::fs::write()`

### 2.2 FileEditor Wrapper (`src/views/file_editor.rs`)

**Location**: `/Users/enzo/Code/turbo-vision/src/views/file_editor.rs`

Extends Editor with file-specific features:

```rust
pub struct FileEditor {
    editor: Editor,
    filename: Option<PathBuf>,
}

pub fn load_file(&mut self, path: PathBuf) -> std::io::Result<()> {
    self.editor.load_file(path.to_str().unwrap())?;
    self.filename = Some(path);
    Ok(())
}

pub fn save(&mut self) -> std::io::Result<bool> {
    if let Some(path) = self.filename.clone() {
        self.editor.save_file()?;
        Ok(true)
    } else {
        Ok(false)  // Need to call save_as
    }
}
```

### 2.3 EditWindow Wrapper (`src/views/edit_window.rs`)

**Location**: `/Users/enzo/Code/turbo-vision/src/views/edit_window.rs`

High-level window for editing:

```rust
pub fn load_file(&mut self, path: &str) -> std::io::Result<()> {
    self.editor.load_file(path)
}

pub fn save_file(&mut self) -> std::io::Result<()> {
    self.editor.save_file()
}

pub fn save_as(&mut self, path: &str) -> std::io::Result<()> {
    self.editor.save_as(path)
}
```

### 2.4 ANSI Dump Utilities (`src/core/ansi_dump.rs`)

**Location**: `/Users/enzo/Code/turbo-vision/src/core/ansi_dump.rs` (80+ lines)

Specialized file I/O for debugging:

```rust
pub fn dump_buffer_to_file(
    buffer: &[Vec<Cell>],
    width: usize,
    height: usize,
    path: &str,
) -> io::Result<()> {
    let mut file = File::create(path)?;
    // ... writes ANSI escape codes to file
}
```

**Purpose**: Dump terminal buffers to ANSI text files for debugging
- Converts TvColor to RGB values
- Writes ANSI escape sequences
- Viewable with `cat` or text editors supporting ANSI codes

---

## 3. Application State Management

### 3.1 Application Structure (`src/app/application.rs`)

**Location**: `/Users/enzo/Code/turbo-vision/src/app/application.rs` (310 lines)

The Application struct holds the desktop state:

```rust
pub struct Application {
    pub terminal: Terminal,
    pub menu_bar: Option<MenuBar>,
    pub status_line: Option<StatusLine>,
    pub desktop: Desktop,
    pub running: bool,
    needs_redraw: bool,
}
```

**State Management:**
- Menu bar state (open/closed dropdowns)
- Status line state (selected item, hint text)
- Desktop state (window list, z-order)
- Running flag for the main loop

**No Built-in Serialization:**
The Application struct does NOT serialize. State is held in memory during runtime.

### 3.2 Desktop State (`src/views/desktop.rs`)

**Location**: `/Users/enzo/Code/turbo-vision/src/views/desktop.rs` (260 lines)

```rust
pub struct Desktop {
    bounds: Rect,
    children: Group,  // Windows collection
}
```

**Methods for State Management:**
- `add()` - Add window to desktop
- `child_count()` - Get number of windows
- `child_at()` - Access window by index
- `remove_child()` - Remove window
- `remove_closed_windows()` - Cleanup marked for deletion
- `handle_moved_windows()` - Track window movement

**Window Z-order:**
- Windows are stored in order (last = topmost)
- `bring_to_front()` brings window to top
- Modal windows capture all events

### 3.3 History Management (`src/core/history.rs`)

**Location**: `/Users/enzo/Code/turbo-vision/src/core/history.rs` (279 lines)

In-memory history for input fields (similar to Borland's THistory):

```rust
pub struct HistoryList {
    items: Vec<String>,
    max_items: usize,
}

pub struct HistoryManager;

impl HistoryManager {
    pub fn add(history_id: u16, item: String) { ... }
    pub fn get_list(history_id: u16) -> Vec<String> { ... }
    pub fn has_history(history_id: u16) -> bool { ... }
    pub fn count(history_id: u16) -> usize { ... }
    pub fn clear(history_id: u16) { ... }
    pub fn clear_all() { ... }
}
```

**Features:**
- Global singleton using OnceLock
- Per-ID history lists (e.g., separate history for file path input)
- Max items limit (default 20)
- Most recent items at front
- Thread-safe (uses Mutex)

**No Persistence:**
- History is cleared when application exits
- Not saved to disk in current implementation

---

## 4. Serialization Frameworks

### 4.1 What's NOT Used

The project does **NOT** include:
- **serde** - No serde crate in Cargo.toml
- **ron** - No RON (Rusty Object Notation)
- **toml** - No TOML support
- **json** - No JSON support
- **bincode** - No binary serialization
- **protobuf** - No Protocol Buffers

### 4.2 Cargo.toml Dependencies

**Location**: `/Users/enzo/Code/turbo-vision/Cargo.toml` (44 lines)

```toml
[dependencies]
crossterm = "0.27"
unicode-width = "0.1"
arboard = "3.3"

[dev-dependencies]
tempfile = "3.8"
```

**Observations:**
- `crossterm` - Terminal abstraction
- `unicode-width` - Text width calculation
- `arboard` - Clipboard access
- `tempfile` - Test fixture file creation
- No serialization crates

---

## 5. Demo Application: Rust Editor (`demo/rust_editor.rs`)

**Location**: `/Users/enzo/Code/turbo-vision/demo/rust_editor.rs` (17915 bytes)

Shows practical usage of menu/status resources and file operations:

```rust
struct EditorState {
    filename: Option<PathBuf>,
}

// Menu definition
let file_menu = Menu::from_items(vec![
    MenuItem::new("~N~ew", CM_NEW, 0, 0),
    MenuItem::new("~O~pen", CM_OPEN, KB_F3, 0),
    MenuItem::new("~S~ave", CM_SAVE, KB_F2, 0),
    MenuItem::new("Save ~A~s", CMD_SAVE_AS, KB_ALT_S, 0),
    MenuItem::separator(),
    MenuItem::new("E~x~it", CM_QUIT, KB_ALT_X, 0),
]);

// Status definition
let status_line = StatusLine::new(
    Rect::new(0, height as i16 - 1, width as i16, height as i16),
    vec![
        StatusItem::new("~F10~ Menu", KB_F10, 0),
        StatusItem::new("~Ctrl+S~ Save", 0, CM_SAVE),
        StatusItem::new("~Ctrl+F~ Find", 0, CMD_SEARCH),
    ],
);

// File operations
fn handle_open_file(app: &mut Application, editor: &mut Editor) -> bool {
    let dialog = FileDialog::new(/* ... */);
    let cmd = app.exec_view(Box::new(dialog));
    if cmd == CM_OK {
        if let Some(path) = selected_path {
            return editor.load_file(&path).is_ok();
        }
    }
    false
}

fn handle_save_file(app: &mut Application, editor: &mut Editor, state: &mut EditorState) {
    if let Some(path) = &state.filename {
        editor.save_as(path.to_str().unwrap()).ok();
    } else {
        // Show save dialog
    }
}
```

---

## 6. State Persistence Patterns

### 6.1 Application-Level State (Not Persisted)

The demo shows application state is **transient**:

```rust
struct EditorState {
    filename: Option<PathBuf>,
}

// In main loop:
if event.what == EventType::Command && event.command == CM_CLOSE {
    // Prompt user to save
    if prompt_save_if_dirty(&mut app, &mut editor_state, true) {
        // Allow close
        app.desktop.remove_child(0);
        editor_state = EditorState::new();  // Reset state
    }
}
```

**Characteristics:**
- Window list rebuilt on startup
- Editor state reset on new/close
- No application.json or .config files
- No desktop save/restore

### 6.2 File-Based Persistence

Only individual files are saved:

```rust
// Save current editor content to file
pub fn save_as(&mut self, path: &str) -> std::io::Result<()> {
    let content = self.get_text();
    std::fs::write(path, content)?;  // Direct file write
    self.filename = Some(path.to_string());
    self.modified = false;
    Ok(())
}
```

**No Configuration Files:**
- No ~/.turbo-vision/ directory
- No settings.toml
- No window positions/sizes saved
- No recent files list
- No user preferences persisted

---

## 7. Command Architecture

### 7.1 Command Set (`src/core/command_set.rs`)

**Location**: `/Users/enzo/Code/turbo-vision/src/core/command_set.rs`

Commands are defined as constants:

```rust
pub const CM_QUIT: CommandId = 3;
pub const CM_OPEN: CommandId = 20;
pub const CM_SAVE: CommandId = 21;
pub const CM_CANCEL: CommandId = 104;
pub const CM_CLOSE: CommandId = 202;
pub const CM_NEW: CommandId = 300;
// ... more command IDs
```

**No Serialization:**
- Commands are just u16 constants
- Menu/status items reference commands directly
- No serialization needed

---

## 8. Text Editing Features (Non-Persisted)

### 8.1 Undo/Redo Stack

**Location**: `/Users/enzo/Code/turbo-vision/src/views/editor.rs`

```rust
#[derive(Clone, Debug)]
enum EditAction {
    InsertChar { pos: Point, ch: char },
    DeleteChar { pos: Point, ch: char },
    InsertText { pos: Point, text: String },
    DeleteText { pos: Point, text: String },
    InsertLine { line: usize, text: String },
    DeleteLine { line: usize, text: String },
}

pub struct Editor {
    undo_stack: Vec<EditAction>,
    redo_stack: Vec<EditAction>,
    modified: bool,
    // ...
}
```

**Limitations:**
- Undo/redo cleared on load_file()
- Undo/redo NOT saved to disk
- Limited to 100 actions (MAX_UNDO_HISTORY)

### 8.2 Find/Replace State

**Location**: `/Users/enzo/Code/turbo-vision/src/views/editor.rs`

```rust
pub struct SearchOptions {
    pub case_sensitive: bool,
    pub whole_words_only: bool,
    pub backwards: bool,
}

pub struct Editor {
    last_search: String,
    last_search_options: SearchOptions,
    // ...
}
```

**Characteristics:**
- Search state kept during session
- Not persisted across sessions

---

## 9. Terminal State (`src/terminal/mod.rs`)

The Terminal manages low-level display state, not application persistence:

```rust
pub struct Terminal {
    // Screen buffer management
    cell_buffer: Vec<Vec<Cell>>,
    cursor_pos: (u16, u16),
    // ...
}
```

**No Serialization:**
- Terminal state is ephemeral
- Rebuilt on each frame

---

## 10. Summary: Serialization Patterns

| Feature | Implemented? | Method | Location |
|---------|--------------|--------|----------|
| Menu Resources | Yes | Rust enums/structs | `core/menu_data.rs` |
| Status Resources | Yes | Rust enums/structs | `core/status_data.rs` |
| File I/O | Yes | std::fs::read_to_string/write | `views/editor.rs` |
| Application State | In-memory only | struct fields | `app/application.rs` |
| Desktop State | In-memory only | Desktop struct | `views/desktop.rs` |
| Window State | In-memory only | Window/View traits | `views/window.rs` |
| Configuration | None | N/A | N/A |
| serde Framework | No | N/A | N/A |
| History Persistence | No | Memory only | `core/history.rs` |
| Undo/Redo Persistence | No | Memory only | `views/editor.rs` |
| Preferences File | No | N/A | N/A |

---

## 11. Recommendations for Adding Persistence

If you want to add serialization:

### Option 1: Add serde Support
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

Then derive on structures:
```rust
#[derive(Serialize, Deserialize)]
pub struct Menu {
    pub items: Vec<MenuItem>,
    pub default_index: Option<usize>,
}
```

### Option 2: Manual Serialization
Write custom to_string/from_str methods for each structure.

### Option 3: Configuration Format
Use existing configs (TOML, YAML) with `toml` or `yaml` crates.

---

## 12. Key Files Summary

| File | Lines | Purpose |
|------|-------|---------|
| `src/core/menu_data.rs` | 356 | Menu resource definitions |
| `src/core/status_data.rs` | 268 | Status line resource definitions |
| `src/views/editor.rs` | 1400+ | Text editing with file I/O |
| `src/views/file_editor.rs` | 150+ | File editor wrapper |
| `src/views/edit_window.rs` | 150+ | Edit window wrapper |
| `src/core/history.rs` | 279 | In-memory history manager |
| `src/core/ansi_dump.rs` | 80+ | Debug output dumping |
| `src/app/application.rs` | 310 | Application state container |
| `src/views/desktop.rs` | 260 | Desktop/window management |
| `demo/rust_editor.rs` | 17915 | Complete editor example |

---

## Conclusion

The Turbo Vision Rust implementation:

1. **Uses NO serialization frameworks** - No serde, JSON, TOML, etc.
2. **Defines resources as Rust structures** - Menu, MenuItem, StatusDef, StatusItem
3. **Implements basic file I/O** - Editor can load/save plain text files
4. **Manages state in memory only** - No persistence between sessions
5. **Provides history system** - Per-ID history for input fields (not persisted)
6. **Matches Borland's architecture** - Commands, menu bar, status line, desktop

The design is simple and idiomatic Rust, focusing on the core UI framework rather than serialization concerns. Adding persistence is straightforward if needed.
