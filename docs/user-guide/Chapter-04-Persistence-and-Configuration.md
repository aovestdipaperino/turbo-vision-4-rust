# Chapter 4: Persistence and Configuration

**Previous:** [Chapter 3 — Adding Windows](Chapter-3-Adding-Windows.md)

---

Now that you've made your tutorial application do actual work, the next logical step would be to save that work and configure your application's interface efficiently. This chapter explores how Turbo Vision applications handle persistence and resource management, and explains the architectural differences between the original Pascal implementation and the current Rust implementation.

The topics in this chapter include:

- Understanding persistence patterns (original vs. current implementation)
- Defining menus programmatically using builders
- Defining status lines with context sensitivity
- Creating reusable dialog box configurations

## Understanding Persistence in Turbo Vision

### The Original Pascal Approach: Binary Streams and Resources

The original Turbo Vision used a sophisticated stream-based persistence system. Applications could save their entire desktop state—including windows, editors, and all UI elements—to binary files and restore them later. This system had several key features:

**Stream Registration**: Each object type registered itself with the stream system, providing methods for serialization (`Store`) and deserialization (`Load`). Types like `TDesktop`, `TWindow`, and `TEditor` could all be written to and read from streams.

**Resource Files**: Complex UI elements like menus, status lines, and dialog boxes could be stored in separate resource files (`*.TVR`). Programs would load these resources by name at runtime, separating UI definition from program logic.

**Type Safety**: Streams encoded type information, so when reading objects back, the system knew which constructor to call for deserialization.

This approach provided excellent separation of concerns and made it easy to persist application state between sessions.

### The Current Rust Implementation: Programmatic Definition

The current Rust implementation takes a different, more modern approach:

**No Binary Persistence**: There's no built-in system for saving desktop state to disk. The codebase doesn't use `serde` or other serialization frameworks. Each application session starts fresh.

**Programmatic Resources**: Menus and status lines are defined directly in Rust code using builder patterns and data structures. This provides compile-time type safety and excellent IDE support.

**Plain Text I/O**: File operations are limited to plain text content (loading and saving files in the editor), using standard `std::fs` operations.

This approach offers different tradeoffs:
- ✅ **Type Safety**: Compile-time checking of all UI definitions
- ✅ **Simplicity**: No complex serialization infrastructure
- ✅ **Modern Rust**: Idiomatic patterns and zero-cost abstractions
- ❌ **No State Persistence**: Can't save/restore window layouts
- ❌ **No External Resources**: UI must be compiled into the binary

### When You Might Want Persistence

While the current implementation doesn't include persistence, there are scenarios where you might want to add it:

1. **Saving Window Layouts**: Users could save their preferred window arrangements
2. **Configuration Files**: Store user preferences, recent files, or application settings
3. **Session Restore**: Resume work exactly where the user left off
4. **Resource Externalization**: Load UI definitions from configuration files

If your application needs these features, you could add the `serde` crate to serialize application state to formats like JSON, TOML, or RON. See the "Implementing Persistence" section at the end of this chapter.

## Defining Menus with MenuBuilder

Turbo Vision's Rust implementation provides a clean, type-safe way to define menus using the builder pattern. Menus are defined using data structures from `src/core/menu_data.rs`:

### Basic Menu Structure

Menus consist of `MenuItem` enums that can be:
- **Regular items**: Execute a command when selected
- **Submenus**: Open nested menus
- **Separators**: Visual dividers

Here's how to create a basic File menu:

```rust
use turbo_vision::core::menu_data::{Menu, MenuItem, MenuBuilder};
use turbo_vision::core::command::*;
use turbo_vision::core::event::*;

// Define your commands
const CM_SAVE_AS: u16 = 105;

// Build the menu using MenuBuilder
let file_menu = MenuBuilder::new()
    .item_with_shortcut("~N~ew", CM_NEW, KB_CTRL_N, "Ctrl+N")
    .item_with_shortcut("~O~pen", CM_OPEN, KB_F3, "F3")
    .item_with_shortcut("~S~ave", CM_SAVE, KB_F2, "F2")
    .item("Save ~a~s...", CM_SAVE_AS, 0)
    .separator()
    .item_with_shortcut("E~x~it", CM_QUIT, KB_ALT_X, "Alt+X")
    .build();
```

**Key Points**:
- The `~` character marks the accelerator key (e.g., `~N~ew` means Alt+N)
- `item_with_shortcut()` displays the shortcut text (like "Ctrl+N") in the menu
- `item()` creates menu items without visible shortcuts
- `separator()` adds visual dividers
- The method calls chain fluently, ending with `build()`

### Creating Nested Menus (Submenus)

You can create hierarchical menu structures by nesting menus:

```rust
// Create a submenu for Help Topics
let help_topics_menu = MenuBuilder::new()
    .item("~I~ndex", CM_HELP_INDEX, 0)
    .item("~K~eyboard", CM_HELP_KEYBOARD, 0)
    .item("~C~ommands", CM_HELP_COMMANDS, 0)
    .build();

// Create the main Help menu with the nested submenu
let help_menu = Menu::from_items(vec![
    MenuItem::new("~C~ontents", CM_HELP_CONTENTS, KB_F1, 0),
    MenuItem::submenu("~T~opics", 0, help_topics_menu, 0),
    MenuItem::separator(),
    MenuItem::new("~A~bout", CM_HELP_ABOUT, 0, 0),
]);
```

### Manual Menu Construction

For more control, you can construct menus directly without the builder:

```rust
let edit_menu = Menu::from_items(vec![
    MenuItem::with_shortcut("~U~ndo", CM_UNDO, KB_CTRL_Z, "Ctrl+Z", 0),
    MenuItem::separator(),
    MenuItem::with_shortcut("Cu~t~", CM_CUT, KB_SHIFT_DEL, "Shift+Del", 0),
    MenuItem::with_shortcut("~C~opy", CM_COPY, KB_CTRL_INS, "Ctrl+Ins", 0),
    MenuItem::with_shortcut("~P~aste", CM_PASTE, KB_SHIFT_INS, "Shift+Ins", 0),
    MenuItem::separator(),
    MenuItem::new("~C~lear", CM_CLEAR, 0, 0),
]);
```

### Attaching Menus to Your Application

Once you've defined your menu structure, attach it to the application:

```rust
fn create_menu_bar(width: u16) -> MenuBar {
    let file_menu = MenuBuilder::new()
        .item_with_shortcut("~N~ew", CM_NEW, KB_CTRL_N, "Ctrl+N")
        .item_with_shortcut("~O~pen", CM_OPEN, KB_F3, "F3")
        .separator()
        .item_with_shortcut("E~x~it", CM_QUIT, KB_ALT_X, "Alt+X")
        .build();

    let edit_menu = MenuBuilder::new()
        .item_with_shortcut("~U~ndo", CM_UNDO, KB_CTRL_Z, "Ctrl+Z")
        .separator()
        .item_with_shortcut("Cu~t~", CM_CUT, KB_SHIFT_DEL, "Shift+Del")
        .item_with_shortcut("~C~opy", CM_COPY, KB_CTRL_INS, "Ctrl+Ins")
        .item_with_shortcut("~P~aste", CM_PASTE, KB_SHIFT_INS, "Shift+Ins")
        .build();

    MenuBar::new(
        Rect::new(0, 0, width as i16, 1),
        vec![
            SubMenu::new("~F~ile", file_menu),
            SubMenu::new("~E~dit", edit_menu),
        ],
    )
}

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;
    let (width, _height) = app.terminal.size();

    let menu_bar = create_menu_bar(width);
    app.set_menu_bar(menu_bar);

    // ... rest of application
}
```

## Defining Status Lines

Status lines provide context-sensitive help and shortcuts at the bottom of the screen. The Rust implementation supports sophisticated status line definitions with context switching based on command ranges.

### Simple Status Lines

For basic applications, create a single status line with fixed items:

```rust
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::core::geometry::Rect;

let status_line = StatusLine::new(
    Rect::new(0, height as i16 - 1, width as i16, height as i16),
    vec![
        StatusItem::new("~F1~ Help", KB_F1, CM_HELP),
        StatusItem::new("~F2~ Save", KB_F2, CM_SAVE),
        StatusItem::new("~F3~ Open", KB_F3, CM_OPEN),
        StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
    ],
);

app.set_status_line(status_line);
```

The status line items:
- Display keyboard shortcuts to the user
- Execute commands when clicked with the mouse
- Use `~` to mark accelerator keys for visual emphasis

### Context-Sensitive Status Lines

More sophisticated applications can show different status items depending on the active view. This is done by defining multiple `StatusDef` instances, each active for a specific command range:

```rust
use turbo_vision::core::status_data::{StatusItem, StatusLine, StatusLineBuilder};

let context_status = StatusLineBuilder::new()
    // Default status for all contexts (command range 0-65535)
    .add_default_def(vec![
        StatusItem::new("~F1~ Help", KB_F1, CM_HELP),
        StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
    ])
    // Editor context (command set 100-199)
    .add_def(100, 199, vec![
        StatusItem::new("~F1~ Help", KB_F1, CM_HELP),
        StatusItem::new("~F2~ Save", KB_F2, CM_SAVE),
        StatusItem::new("~F3~ Open", KB_F3, CM_OPEN),
        StatusItem::new("~Ctrl+Y~ Delete line", KB_CTRL_Y, CM_DELETE_LINE),
        StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
    ])
    // Dialog context (command set 200-299)
    .add_def(200, 299, vec![
        StatusItem::new("~F1~ Help", KB_F1, CM_HELP),
        StatusItem::new("~Tab~ Next", KB_TAB, CM_NEXT),
        StatusItem::new("~Esc~ Cancel", KB_ESC, CM_CANCEL),
    ])
    .build();
```

The status line will automatically switch between these definitions based on the command set used by the currently focused view. This provides users with context-appropriate help and shortcuts.

### How Context Switching Works

Each view in Turbo Vision belongs to a command set (a range of command IDs). When a view receives focus, the status line queries which command set is active and displays the appropriate `StatusDef`:

```rust
// Status line finds the right definition for the current context
if let Some(def) = status_line.get_def_for(current_command) {
    // Display the items from this definition
    for item in &def.items {
        // ... render the item
    }
}
```

This mechanism allows the status line to show:
- File operations when an editor has focus
- Dialog navigation when a dialog box is active
- General commands when the desktop has focus

## Creating Dialog Boxes

Dialog boxes are complex views that contain multiple controls (buttons, input fields, checkboxes, etc.). While the original Pascal version could store dialog definitions in resource files, the Rust implementation defines them programmatically.

### Basic Dialog Structure

A dialog box is created by:
1. Defining the dialog's boundaries
2. Constructing the dialog
3. Adding controls (buttons, static text, input fields, etc.)

Here's an example of creating an "About" dialog:

```rust
use turbo_vision::views::dialog::DialogBuilder;
use turbo_vision::views::button::ButtonBuilder;
use turbo_vision::views::static_text::StaticTextBuilder;
use turbo_vision::core::geometry::Rect;

fn create_about_dialog() -> Dialog {
    // Create the dialog with title using the builder pattern
    let mut dialog = DialogBuilder::new()
        .bounds(Rect::new(0, 0, 40, 11))
        .title("About Tutorial")
        .build();
    dialog.set_centered(true);  // Center on screen

    // Add static text
    let text = StaticTextBuilder::new()
        .bounds(Rect::new(4, 2, 36, 4))
        .text("Turbo Vision\nTutorial program")
        .build_boxed();
    dialog.add(text);

    // Add copyright text
    let copyright = StaticTextBuilder::new()
        .bounds(Rect::new(4, 5, 36, 7))
        .text("Copyright 1992-2025\nBorland International / Rust Port")
        .build_boxed();
    dialog.add(copyright);

    // Add OK button
    let ok_button = ButtonBuilder::new()
        .bounds(Rect::new(15, 8, 25, 10))
        .title("O~k~")
        .command(CM_OK)
        .default(true)  // true = default
        .build_boxed();
    dialog.add(ok_button);

    dialog
}
```

### Executing Dialog Boxes

Once defined, dialogs are executed through the application's event loop. The application handles the dialog's events until the user closes it:

```rust
// Show the about dialog
let about_dialog = create_about_dialog();
app.desktop.add(Box::new(about_dialog));

// The dialog will stay visible and handle events
// until a button generates CM_OK, CM_CANCEL, or similar
```

### Reusable Dialog Functions

For common dialogs, define factory functions:

```rust
fn show_confirmation(app: &mut Application, title: &str, message: &str) -> bool {
    use turbo_vision::views::msgbox::confirmation_box;

    let result = confirmation_box(
        &mut app.terminal,
        title,
        message,
    );

    // Returns true if user clicked Yes, false for No or Cancel
    result
}

// Usage:
if show_confirmation(&mut app, "Save Changes?", "Do you want to save your changes?") {
    save_file(&mut app, &mut editor_state);
}
```

The `msgbox` module provides several pre-built dialog functions:
- `message_box_ok`: Simple information message
- `message_box_error`: Error message
- `confirmation_box`: Yes/No/Cancel dialog
- `search_box`: Search input dialog
- `search_replace_box`: Find and replace dialog
- `goto_line_box`: Jump to line number dialog

## Implementing Persistence (Optional)

If your application needs to save and restore state between sessions, you can add serialization using the `serde` ecosystem. Here's a high-level overview of how to implement it:

### Adding Dependencies

First, add serialization dependencies to your `Cargo.toml`:

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"  # or ron = "0.8" for RON format
```

### Making State Serializable

Define serializable structures for your application state:

```rust
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct WindowState {
    title: String,
    x: i16,
    y: i16,
    width: u16,
    height: u16,
    file_path: Option<PathBuf>,
}

#[derive(Serialize, Deserialize)]
struct AppConfig {
    windows: Vec<WindowState>,
    last_directory: Option<PathBuf>,
    recent_files: Vec<PathBuf>,
}
```

### Saving State

Implement a function to capture and save the application state:

```rust
fn save_state(app: &Application, path: &Path) -> std::io::Result<()> {
    let config = AppConfig {
        windows: app.desktop.windows()
            .map(|w| WindowState {
                title: w.title().to_string(),
                x: w.bounds().x,
                y: w.bounds().y,
                width: w.bounds().width(),
                height: w.bounds().height(),
                file_path: w.get_file_path(),
            })
            .collect(),
        last_directory: get_last_directory(),
        recent_files: get_recent_files(),
    };

    let json = serde_json::to_string_pretty(&config)?;
    std::fs::write(path, json)?;
    Ok(())
}
```

### Restoring State

Implement a function to load and restore saved state:

```rust
fn load_state(app: &mut Application, path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        return Ok(());  // No saved state, that's fine
    }

    let json = std::fs::read_to_string(path)?;
    let config: AppConfig = serde_json::from_str(&json)?;

    // Restore windows
    for window_state in config.windows {
        let bounds = Rect::new(
            window_state.x,
            window_state.y,
            window_state.x + window_state.width as i16,
            window_state.y + window_state.height as i16,
        );

        if let Some(file_path) = window_state.file_path {
            // Open the file in a new window
            open_file_in_window(app, &file_path, bounds);
        }
    }

    // Restore other settings
    set_last_directory(config.last_directory);
    set_recent_files(config.recent_files);

    Ok(())
}
```

### Integration Points

Call these functions at appropriate times:

```rust
fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;
    let config_path = get_config_path()?;

    // Load saved state on startup
    let _ = load_state(&mut app, &config_path);

    // Run the application
    app.run()?;

    // Save state on exit
    let _ = save_state(&app, &config_path);

    Ok(())
}

fn get_config_path() -> std::io::Result<PathBuf> {
    let mut path = dirs::config_dir()
        .ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Config directory not found"
        ))?;
    path.push("turbo-vision");
    std::fs::create_dir_all(&path)?;
    path.push("state.json");
    Ok(path)
}
```

### Alternative: Using RON Format

For a more human-readable format that better preserves Rust types, consider RON (Rusty Object Notation):

```rust
// Save as RON
let ron = ron::ser::to_string_pretty(&config, Default::default())?;
std::fs::write(path, ron)?;

// Load from RON
let ron = std::fs::read_to_string(path)?;
let config: AppConfig = ron::from_str(&ron)?;
```

RON files look like this:

```ron
AppConfig(
    windows: [
        WindowState(
            title: "main.rs",
            x: 10,
            y: 5,
            width: 80,
            height: 25,
            file_path: Some("/path/to/main.rs"),
        ),
    ],
    last_directory: Some("/path/to/project"),
    recent_files: [
        "/path/to/main.rs",
        "/path/to/lib.rs",
    ],
)
```

## Comparison: Original vs. Current Implementation

| Feature | Original Pascal | Current Rust |
|---------|----------------|--------------|
| **Menu Definition** | Binary resources or code | Programmatic (builders) |
| **Status Lines** | Binary resources or code | Programmatic (builders) |
| **Dialog Boxes** | Binary resources or code | Programmatic construction |
| **Desktop Persistence** | Built-in (`TStream`, `TResourceFile`) | Not implemented (can add with `serde`) |
| **Type Safety** | Runtime (stream registration) | Compile-time |
| **UI/Code Separation** | External resource files | Compiled into binary |
| **Hot Reloading** | Possible (reload resources) | Not supported |
| **Configuration** | Binary format | Can use JSON/TOML/RON |

## Summary

This chapter covered how Turbo Vision handles UI definition and configuration:

1. **Persistence Philosophy**: The original Pascal version used binary streams and resources for everything. The current Rust version favors programmatic definition with optional persistence through `serde`.

2. **Menu Definition**: Use `MenuBuilder` and `MenuItem` for type-safe, fluent menu construction. Menus support nesting, shortcuts, accelerators, and separators.

3. **Status Line Definition**: Create simple or context-sensitive status lines using `StatusLine` and `StatusLineBuilder`. Context switching allows different shortcuts based on the active view.

4. **Dialog Boxes**: Define dialogs programmatically by creating a `Dialog` and adding controls. Use the pre-built `msgbox` functions for common dialog patterns.

5. **Adding Persistence**: If needed, add the `serde` crate to serialize application state to JSON, TOML, or RON formats. This enables saving window layouts, preferences, and session state.

The current Rust implementation prioritizes compile-time safety and simplicity over runtime flexibility. For most applications, defining UI elements in code provides excellent IDE support, refactoring capabilities, and type safety. When external configuration is needed, Rust's serialization ecosystem provides modern, flexible alternatives to binary resource files.

**Key Files to Explore**:
- `src/core/menu_data.rs` - Menu data structures and builders (356 lines)
- `src/core/status_data.rs` - Status line data structures (268 lines)
- `src/views/menu_bar.rs` - Menu bar view implementation
- `src/views/status_line.rs` - Status line view implementation
- `demo/rust_editor.rs` - Complete example showing menus, status lines, and dialogs
- `examples/menu_status_data.rs` - Focused examples of menu and status line construction

---

**Next:** [Chapter 5 — Creating Data Entry Forms](Chapter-5-Creating-Data-Entry-Forms.md)
