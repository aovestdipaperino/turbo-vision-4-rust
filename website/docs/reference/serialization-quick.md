# Turbo Vision Serialization Quick Reference

## Overview

The Turbo Vision Rust implementation focuses on **resource definitions** (Menus, Status Lines) and **basic file I/O**, with **NO serialization frameworks** (no serde, JSON, TOML, etc.).

## Menu Resource Definition

**File**: `src/core/menu_data.rs`

```rust
use turbo_vision::core::menu_data::{Menu, MenuItem, MenuBuilder};
use turbo_vision::core::command::{CM_OPEN, CM_SAVE, CM_QUIT};
use turbo_vision::core::event::{KB_F3, KB_F2, KB_ALT_X};

// Method 1: Using from_items()
let menu = Menu::from_items(vec![
    MenuItem::new("~O~pen", CM_OPEN, KB_F3, 0),
    MenuItem::new("~S~ave", CM_SAVE, KB_F2, 0),
    MenuItem::separator(),
    MenuItem::new("E~x~it", CM_QUIT, KB_ALT_X, 0),
]);

// Method 2: Using builder
let menu = MenuBuilder::new()
    .item("~O~pen", CM_OPEN, KB_F3)
    .item("~S~ave", CM_SAVE, KB_F2)
    .separator()
    .item("E~x~it", CM_QUIT, KB_ALT_X)
    .build();
```

## Status Line Resource Definition

**File**: `src/core/status_data.rs`

```rust
use turbo_vision::core::status_data::{StatusItem, StatusDef, StatusLine, StatusLineBuilder};
use turbo_vision::core::command::{CM_HELP, CM_QUIT};
use turbo_vision::core::event::{KB_F1, KB_ALT_X};

// Method 1: Direct instantiation
let status = StatusLine::single(vec![
    StatusItem::new("~F1~ Help", KB_F1, CM_HELP),
    StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
]);

// Method 2: Using builder with command ranges
let status = StatusLineBuilder::new()
    .add_def(0, 100, vec![
        StatusItem::new("~F1~ Help", KB_F1, CM_HELP),
        StatusItem::new("~F2~ Save", 0, CM_SAVE),
    ])
    .add_def(101, 200, vec![
        StatusItem::new("~F1~ Help", KB_F1, CM_HELP),
        StatusItem::new("~Esc~ Cancel", 0x011B, 102),
    ])
    .build();
```

## File I/O Operations

**File**: `src/views/editor.rs`

```rust
use turbo_vision::views::editor::Editor;

let mut editor = Editor::new(bounds);

// Load file
editor.load_file("path/to/file.txt")?;

// Check if modified
if editor.is_modified() {
    println!("File has unsaved changes");
}

// Save to new file
editor.save_as("path/to/new_file.txt")?;

// Get current filename
if let Some(filename) = editor.get_filename() {
    println!("Editing: {}", filename);
}
```

## Application State Structure

**File**: `src/app/application.rs`

```rust
use turbo_vision::app::Application;
use turbo_vision::views::menu_bar::MenuBar;
use turbo_vision::views::status_line::StatusLine;

let mut app = Application::new()?;

// Set menu bar (optional)
app.set_menu_bar(menu_bar);

// Set status line (optional)
app.set_status_line(status_line);

// Add window to desktop
app.desktop.add(Box::new(window));

// Get window count
let count = app.desktop.child_count();

// Remove window by index
app.desktop.remove_child(0);

// Main loop
while app.running {
    if let Some(event) = app.get_event() {
        app.handle_event(&event);
    }
}
```

## History Management

**File**: `src/core/history.rs`

```rust
use turbo_vision::core::history::HistoryManager;

// History IDs (application-defined)
const HISTORY_SEARCH: u16 = 100;
const HISTORY_FILENAME: u16 = 101;

// Add to history
HistoryManager::add(HISTORY_SEARCH, "search term".to_string());

// Get history list
let items = HistoryManager::get_list(HISTORY_SEARCH);
for item in items {
    println!("  {}", item);  // Most recent first
}

// Check if history exists
if HistoryManager::has_history(HISTORY_FILENAME) {
    println!("Found {} items", HistoryManager::count(HISTORY_FILENAME));
}

// Clear history
HistoryManager::clear(HISTORY_SEARCH);
HistoryManager::clear_all();  // Clear all
```

## What's NOT Supported

These features are NOT implemented:

| Feature | Reason |
|---------|--------|
| Serialization frameworks (serde, JSON, TOML, etc.) | Not in Cargo.toml |
| Application state persistence | No save/restore mechanism |
| Desktop layout saving | Windows not persisted |
| Window position/size saving | State not saved |
| Configuration files | No ~/.turbo-vision/ or config.toml |
| History persistence | Cleared on exit |
| Undo/redo persistence | Cleared on file load |
| Recent files list | Not implemented |
| User preferences | Not implemented |

## Adding Serialization (If Needed)

To add serde support:

```toml
# Cargo.toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MenuItem {
    pub text: String,
    pub command: u16,
    pub key_code: u16,
    pub enabled: bool,
}

// Now you can serialize
let json = serde_json::to_string(&menu_item)?;

// And deserialize
let restored: MenuItem = serde_json::from_str(&json)?;
```

## Key Architecture Points

1. **Menu/Status as Data** - Not serialized, defined in code
2. **Editor State** - In-memory only, cleared on new/load
3. **Desktop State** - In-memory only, rebuilt on startup
4. **File Operations** - Plain UTF-8 text using `std::fs`
5. **History** - Global singleton, cleared on exit
6. **Command Set** - Static constants, no persistence needed

## Example: Complete Editor Setup

```rust
use turbo_vision::app::Application;
use turbo_vision::core::menu_data::{Menu, MenuItem, MenuBuilder};
use turbo_vision::core::command::{CM_QUIT, CM_OPEN, CM_SAVE};
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
use turbo_vision::views::editor::Editor;
use turbo_vision::core::geometry::Rect;

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Define menu
    let file_menu = MenuBuilder::new()
        .item("~O~pen", CM_OPEN, 0x3D00)
        .item("~S~ave", CM_SAVE, 0x3C00)
        .separator()
        .item("E~x~it", CM_QUIT, 0x2D00)
        .build();

    let mut menu_bar = MenuBar::new(Rect::new(0, 0, width as i16, 1));
    menu_bar.add_submenu(SubMenu::new("~F~ile", file_menu));
    app.set_menu_bar(menu_bar);

    // Add editor window
    let editor_bounds = Rect::new(1, 1, width as i16 - 1, height as i16 - 1);
    let mut editor = Editor::new(editor_bounds);
    editor.load_file("example.txt")?;

    // Run application
    app.desktop.add(Box::new(editor));
    app.run();

    Ok(())
}
```

## See Also

- Full documentation: `SERIALIZATION_AND_PERSISTENCE.md`
- Menu API: `src/core/menu_data.rs`
- Status API: `src/core/status_data.rs`
- Editor API: `src/views/editor.rs`
- History API: `src/core/history.rs`
- Demo app: `demo/rust_editor.rs`
