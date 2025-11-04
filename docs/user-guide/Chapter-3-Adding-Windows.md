# Chapter 3 — Adding Windows (Rust Edition)

**Version:** 0.2.11
**Updated:** 2025-11-04

So far you've customized your application's menu bar and status line and seen how to respond to their commands. In this chapter, you'll start adding windows to the desktop and managing them.

In this chapter, you'll do the following:

- Add a simple window
- Add file editor windows
- Use a standard file open dialog box
- Manage multiple windows

---

## Understanding the Desktop

One of the great benefits of Turbo Vision is that it makes it easy to create and manage multiple, overlapping, resizeable windows.

The key to managing windows is the **desktop**, which knows how to keep track of all the windows you give it. The desktop is one example of a **group** in Turbo Vision; that is, a visible object that holds and manages other visible items. You've already used groups—the application itself handles the menu bar, status line, and desktop. As you proceed, you'll find that windows and dialog boxes are also groups.

By default, the desktop covers all of the application screen that isn't covered by the menu bar and status line.

---

## Step 3 — Adding a Simple Window

Adding a window to the desktop in an application takes three steps:

1. **Assign the boundaries for the window**
2. **Construct the window object**
3. **Insert the window into the desktop**

As a first step, you can add a plain window to the desktop in response to the **New** item on the File menu. That item generates a `CM_NEW` command, so you need to define a response to that command in the application's event loop.

### Listing 3.1 — Adding a Simple Window

```rust
// tutorial_05.rs
use turbo_vision::app::Application;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::command::{CM_QUIT, CM_NEW, CM_ABOUT};
use turbo_vision::core::event::{EventType, KB_F1, KB_ALT_X};
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
use turbo_vision::views::status_line::{StatusLine, StatusItem};
use turbo_vision::views::window::Window;
use turbo_vision::views::msgbox::message_box_ok;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Initialize menu bar
    let menu_bar = create_menu_bar(width);
    app.set_menu_bar(menu_bar);

    // Initialize status line
    let status_line = create_status_line(height, width);
    app.set_status_line(status_line);

    // Custom event loop with three-phase processing
    app.running = true;
    while app.running {
        // Draw the interface
        app.desktop.draw(&mut app.terminal);
        if let Some(ref mut menu_bar) = app.menu_bar {
            menu_bar.draw(&mut app.terminal);
        }
        if let Some(ref mut status_line) = app.status_line {
            status_line.draw(&mut app.terminal);
        }
        let _ = app.terminal.flush();

        // Handle events
        if let Ok(Some(mut event)) = app.terminal.poll_event(Duration::from_millis(50)) {
            // Phase 1: PreProcess - Status line first
            if let Some(ref mut status_line) = app.status_line {
                status_line.handle_event(&mut event);
            }

            // Special: Menu bar handles F10 and Alt keys
            if let Some(ref mut menu_bar) = app.menu_bar {
                menu_bar.handle_event(&mut event);

                // Check for cascading submenus
                if event.what == EventType::Keyboard || event.what == EventType::MouseUp {
                    if let Some(command) = menu_bar.check_cascading_submenu(&mut app.terminal) {
                        if command != 0 {
                            event = turbo_vision::core::event::Event::command(command);
                        }
                    }
                }
            }

            // Phase 2: Focused - Desktop handles events
            app.desktop.handle_event(&mut event);

            // Application-level command handling
            if event.what == EventType::Command {
                match event.command {
                    CM_QUIT => {
                        app.running = false;
                    }
                    CM_NEW => {
                        new_window(&mut app);
                    }
                    CM_ABOUT => {
                        show_about_box(&mut app);
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn new_window(app: &mut Application) {
    // Step 1: Assign boundaries for the window
    let bounds = Rect::new(5, 3, 65, 18);

    // Step 2: Construct the window object
    let window = Window::new(bounds, "A Window");

    // Step 3: Insert the window into the desktop
    app.desktop.add(Box::new(window));
}

fn create_menu_bar(width: u16) -> MenuBar {
    let mut menu_bar = MenuBar::new(Rect::new(0, 0, width as i16, 1));

    // File menu
    let file_menu_items = vec![
        MenuItem::with_shortcut("~N~ew", CM_NEW, 0, "", 0),
        MenuItem::with_shortcut("E~x~it", CM_QUIT, KB_ALT_X, "Alt+X", 0),
    ];
    let file_menu = SubMenu::new("~F~ile", Menu::from_items(file_menu_items));
    menu_bar.add_submenu(file_menu);

    // Help menu
    let help_menu_items = vec![
        MenuItem::with_shortcut("~A~bout", CM_ABOUT, KB_F1, "F1", 0),
    ];
    let help_menu = SubMenu::new("~H~elp", Menu::from_items(help_menu_items));
    menu_bar.add_submenu(help_menu);

    menu_bar
}

fn create_status_line(height: u16, width: u16) -> StatusLine {
    StatusLine::new(
        Rect::new(0, height as i16 - 1, width as i16, height as i16),
        vec![
            StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
        ],
    )
}

fn show_about_box(app: &mut Application) {
    message_box_ok(
        app,
        "About",
        "  Turbo Vision Tutorial App 3.0\n\n  Rust Edition"
    );
}
```

### Understanding the Code

#### Assigning the Window Boundaries

```rust
let bounds = Rect::new(5, 3, 65, 18);
```

This creates a rectangle with:
- Top-left corner at (5, 3)
- Bottom-right corner at (65, 18)
- Width of 60 characters
- Height of 15 lines

Unlike the menu bar and status line (which used the application's full extent), here you assign the new window an absolute set of coordinates.

#### Constructing the Window Object

```rust
let window = Window::new(bounds, "A Window");
```

This constructs a new window with the specified boundaries and title. The window comes with:
- A frame (border) with the title
- Close button (mouse click on top-left corner)
- Zoom button (mouse click on top-right corner)
- Resize handles
- Interior group for adding child views

#### Inserting the Window

```rust
app.desktop.add(Box::new(window));
```

`add()` is a method common to all Turbo Vision groups. When you insert a window into the desktop, you're telling the desktop that it is supposed to manage that window. The desktop will:
- Draw the window when needed
- Handle window focus and activation
- Manage Z-order (which window is on top)
- Route events to the window

**Note:** In Rust, we use `Box::new(window)` to transfer ownership of the window to the desktop. This is Rust's ownership system at work—once added, the desktop owns the window.

### Running the Program

If you run the program now and choose **New** from the File menu, an empty window with the title "A Window" appears on the desktop. If you choose **New** again, another identical window appears in the same place (because `new_window` assigns exact coordinates).

Using your mouse, you can:
- Select different windows by clicking on them
- Move windows by dragging their title bar
- Resize windows by dragging their borders
- Close windows by clicking the close box (top-left corner)
- Zoom windows by clicking the zoom box (top-right corner)

---

## Step 4 — Adding an Editor Window

Now that you've seen how windows behave in general, you might want to include a more useful window, such as a file editor window. Turbo Vision's `Editor` view provides comprehensive text editing capabilities.

Adding an editor window requires:

1. **Creating a window**
2. **Creating an editor view**
3. **Adding the editor to the window**

### Listing 3.2 — Creating an Editor Window

```rust
// tutorial_06.rs
use turbo_vision::app::Application;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::command::{CM_QUIT, CM_NEW, CM_OPEN, CM_ABOUT};
use turbo_vision::core::event::{EventType, KB_F1, KB_F3, KB_ALT_X};
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
use turbo_vision::views::status_line::{StatusLine, StatusItem};
use turbo_vision::views::window::Window;
use turbo_vision::views::editor::Editor;
use turbo_vision::views::msgbox::message_box_ok;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Initialize menu bar
    let menu_bar = create_menu_bar(width);
    app.set_menu_bar(menu_bar);

    // Initialize status line
    let status_line = create_status_line(height, width);
    app.set_status_line(status_line);

    // Custom event loop
    app.running = true;
    while app.running {
        // Draw
        app.desktop.draw(&mut app.terminal);
        if let Some(ref mut menu_bar) = app.menu_bar {
            menu_bar.draw(&mut app.terminal);
        }
        if let Some(ref mut status_line) = app.status_line {
            status_line.draw(&mut app.terminal);
        }
        let _ = app.terminal.flush();

        // Handle events
        if let Ok(Some(mut event)) = app.terminal.poll_event(Duration::from_millis(50)) {
            // Phase 1: PreProcess
            if let Some(ref mut status_line) = app.status_line {
                status_line.handle_event(&mut event);
            }

            // Menu bar
            if let Some(ref mut menu_bar) = app.menu_bar {
                menu_bar.handle_event(&mut event);
                if event.what == EventType::Keyboard || event.what == EventType::MouseUp {
                    if let Some(command) = menu_bar.check_cascading_submenu(&mut app.terminal) {
                        if command != 0 {
                            event = turbo_vision::core::event::Event::command(command);
                        }
                    }
                }
            }

            // Phase 2: Focused
            app.desktop.handle_event(&mut event);

            // Application commands
            if event.what == EventType::Command {
                match event.command {
                    CM_QUIT => {
                        app.running = false;
                    }
                    CM_NEW => {
                        new_editor_window(&mut app);
                    }
                    CM_ABOUT => {
                        show_about_box(&mut app);
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn new_editor_window(app: &mut Application) {
    // Create window
    let window_bounds = Rect::new(5, 3, 75, 20);
    let mut window = Window::new(window_bounds, "Untitled");

    // Create editor with interior bounds (inset by 1 for the frame)
    let editor_bounds = Rect::new(1, 1, 69, 16);
    let editor = Editor::new(editor_bounds).with_scrollbars_and_indicator();

    // Add editor to window
    window.add(Box::new(editor));

    // Add window to desktop
    app.desktop.add(Box::new(window));
}

fn create_menu_bar(width: u16) -> MenuBar {
    let mut menu_bar = MenuBar::new(Rect::new(0, 0, width as i16, 1));

    // File menu
    let file_menu_items = vec![
        MenuItem::with_shortcut("~N~ew", CM_NEW, 0, "", 0),
        MenuItem::with_shortcut("~O~pen...", CM_OPEN, KB_F3, "F3", 0),
        MenuItem::separator(),
        MenuItem::with_shortcut("E~x~it", CM_QUIT, KB_ALT_X, "Alt+X", 0),
    ];
    let file_menu = SubMenu::new("~F~ile", Menu::from_items(file_menu_items));
    menu_bar.add_submenu(file_menu);

    // Help menu
    let help_menu_items = vec![
        MenuItem::with_shortcut("~A~bout", CM_ABOUT, KB_F1, "F1", 0),
    ];
    let help_menu = SubMenu::new("~H~elp", Menu::from_items(help_menu_items));
    menu_bar.add_submenu(help_menu);

    menu_bar
}

fn create_status_line(height: u16, width: u16) -> StatusLine {
    StatusLine::new(
        Rect::new(0, height as i16 - 1, width as i16, height as i16),
        vec![
            StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
            StatusItem::new("~F3~ Open", KB_F3, CM_OPEN),
        ],
    )
}

fn show_about_box(app: &mut Application) {
    message_box_ok(
        app,
        "About",
        "  Turbo Vision Tutorial App 3.0\n\n  Rust Edition"
    );
}
```

### Understanding Editor Windows

#### Window Title

The window is created with the title "Untitled", which indicates that whatever you type into the editor has not yet been assigned to a specific file. This is the standard convention for new, unsaved documents.

#### Editor Bounds

```rust
let editor_bounds = Rect::new(1, 1, 69, 16);
```

The editor bounds are **relative to the window's interior**. We inset by 1 from each edge to account for the window's frame. The coordinates are in the window's coordinate system, not screen coordinates.

#### Editor Features

```rust
let editor = Editor::new(editor_bounds).with_scrollbars_and_indicator();
```

The `with_scrollbars_and_indicator()` method adds:
- **Horizontal scrollbar** - For long lines
- **Vertical scrollbar** - For many lines
- **Indicator** - Shows current line/column position

The editor supports:
- **Navigation** - Arrow keys, Home, End, PgUp, PgDn
- **Editing** - Insert, Delete, Backspace
- **Selection** - Shift+Arrow keys, Ctrl+A
- **Clipboard** - Ctrl+C (copy), Ctrl+X (cut), Ctrl+V (paste)
- **Undo/Redo** - Ctrl+Z (undo), Ctrl+Y (redo)
- **Auto-indent** - Preserves indentation on new lines

---

## Step 5 — Using Standard Dialog Boxes

Having a file editor that creates new files is useful, but you need to be able to edit existing files, too. To do that, you need to tell the file editor which file you want to edit. Turbo Vision's `FileDialog` provides a dialog box that lets users browse directories and select files.

To edit existing files, you need to:

1. **Construct a file dialog box**
2. **Execute the dialog box to prompt for a file name**
3. **Construct an editor window for that file**

### Listing 3.3 — Opening Files with FileDialog

```rust
// tutorial_07.rs (additions to previous code)
use turbo_vision::views::file_dialog::FileDialog;
use std::fs;

// Add to main event loop, in the Command handling:
CM_OPEN => {
    open_window(&mut app);
}

fn open_window(app: &mut Application) {
    // Step 1: Construct a file dialog box
    let (term_width, term_height) = app.terminal.size();
    let dialog_width = 60;
    let dialog_height = 20;
    let dialog_x = (term_width as i16 - dialog_width) / 2;
    let dialog_y = (term_height as i16 - dialog_height) / 2;

    let mut file_dialog = FileDialog::new(
        Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height),
        "Open File",
        "*.rs",      // Wildcard pattern: *.rs for Rust files, * for all files
        None,        // Start in current directory
    ).build();

    // Step 2: Execute the dialog box to get the file name
    if let Some(path) = file_dialog.execute(app) {
        // User selected a file

        // Step 3: Load the file and create an editor window
        match fs::read_to_string(&path) {
            Ok(content) => {
                // Create window with filename as title
                let window_bounds = Rect::new(5, 3, 75, 20);
                let title = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Untitled");
                let mut window = Window::new(window_bounds, title);

                // Create editor and load content
                let editor_bounds = Rect::new(1, 1, 69, 16);
                let mut editor = Editor::new(editor_bounds).with_scrollbars_and_indicator();
                editor.set_text(&content);

                // Add editor to window
                window.add(Box::new(editor));

                // Add window to desktop
                app.desktop.add(Box::new(window));
            }
            Err(_) => {
                message_box_ok(app, "Error", "Failed to open file");
            }
        }
    }
    // If None was returned, the user canceled the dialog
}
```

### Understanding FileDialog

#### Constructor Parameters

```rust
FileDialog::new(bounds, title, wildcard, start_dir)
```

- **bounds**: The position and size of the dialog on the screen
- **title**: The dialog box title (e.g., "Open File")
- **wildcard**: File filter pattern
  - `"*"` - Show all files
  - `"*.rs"` - Show only Rust files
  - `"*.toml"` - Show only TOML files
  - `"*.txt"` - Show only text files
- **start_dir**: Starting directory (None = current directory)

#### Executing the Dialog

```rust
if let Some(path) = file_dialog.execute(app) {
    // User selected a file
} else {
    // User canceled
}
```

The `execute()` method:
- Makes the dialog **modal** (blocks interaction with the rest of the app)
- Returns `Option<PathBuf>`:
  - `Some(path)` if the user selected a file
  - `None` if the user canceled

**Modal** means the dialog is the only active part of the application—you can't interact with windows behind it until you close the dialog.

> **Note:** The status line is always available, no matter what dialog box is modal.

#### FileDialog Features

The dialog supports:
- **Mouse clicks** - Select files and directories
- **Double-click** - Open directories or select files
- **Arrow keys** - Navigate the file list
- **Enter** - Open directories or select files
- **Backspace** - Go to parent directory
- **Directory navigation** - Browse the file system
- **Wildcard filtering** - Only show matching files

---

## Managing Multiple Windows

### Window Z-Order

When you add multiple windows to the desktop, they are layered on top of each other. This layering is known as **Z-order**, and it determines:
- Which window appears in front
- Which window receives keyboard input
- Which window is "active"

```rust
// First window (bottom)
app.desktop.add(Box::new(window1));

// Second window (on top of first)
app.desktop.add(Box::new(window2));

// Third window (on top)
app.desktop.add(Box::new(window3));
```

### Selecting Windows

Users can select (activate) windows in several ways:
- **Mouse click** - Click anywhere on a window to bring it to the front
- **Keyboard** - The focused window receives keyboard input
- **Menu commands** - Standard window management commands

### Closing Windows

Windows can be closed by:
- **Close button** - Click the close box in the top-left corner
- **Alt+F3** - Standard close window shortcut
- **Programmatically** - Calling `window.close()`

The desktop automatically removes closed windows when you call:

```rust
app.desktop.remove_closed_windows();
```

Call this after handling events but before drawing to clean up closed windows.

---

## Working with Window Contents

### Adding Views to Windows

Windows are groups, which means they can contain other views:

```rust
let mut window = Window::new(bounds, "Window Title");

// Add an editor
let editor = Editor::new(editor_bounds).with_scrollbars_and_indicator();
window.add(Box::new(editor));

// Or add a static text label
let text = StaticText::new(text_bounds, "Hello, World!");
window.add(Box::new(text));

// Or add a button
let button = Button::new(button_bounds, "Click Me", 100, true);
window.add(Box::new(button));
```

### Coordinate Systems

When adding views to windows, remember that coordinates are **relative to the window's interior**:

```rust
// Window covers screen coordinates (10, 5) to (70, 20)
let window_bounds = Rect::new(10, 5, 70, 20);
let mut window = Window::new(window_bounds, "My Window");

// Editor uses window-relative coordinates
// (1, 1) is one character in from the window's top-left interior corner
let editor_bounds = Rect::new(1, 1, 59, 14);
let editor = Editor::new(editor_bounds);

window.add(Box::new(editor));
```

The window's frame takes up one character on each edge, so:
- Window bounds: `(10, 5)` to `(70, 20)` = 60×15
- Interior bounds: `(0, 0)` to `(58, 13)` = 58×13 (inset by 1 on each side)

---

## Complete Example

Here's a complete example that ties everything together:

```rust
// tutorial_08_complete.rs
use turbo_vision::app::Application;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::command::{CM_QUIT, CM_NEW, CM_OPEN, CM_ABOUT};
use turbo_vision::core::event::{EventType, KB_F1, KB_F3, KB_ALT_X};
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
use turbo_vision::views::status_line::{StatusLine, StatusItem};
use turbo_vision::views::window::Window;
use turbo_vision::views::editor::Editor;
use turbo_vision::views::file_dialog::FileDialog;
use turbo_vision::views::msgbox::message_box_ok;
use std::time::Duration;
use std::fs;

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Initialize menu bar and status line
    let menu_bar = create_menu_bar(width);
    app.set_menu_bar(menu_bar);

    let status_line = create_status_line(height, width);
    app.set_status_line(status_line);

    // Main event loop
    app.running = true;
    while app.running {
        // Draw
        app.desktop.draw(&mut app.terminal);
        if let Some(ref mut menu_bar) = app.menu_bar {
            menu_bar.draw(&mut app.terminal);
        }
        if let Some(ref mut status_line) = app.status_line {
            status_line.draw(&mut app.terminal);
        }
        let _ = app.terminal.flush();

        // Handle events
        if let Ok(Some(mut event)) = app.terminal.poll_event(Duration::from_millis(50)) {
            // Phase 1: PreProcess
            if let Some(ref mut status_line) = app.status_line {
                status_line.handle_event(&mut event);
            }

            // Menu bar
            if let Some(ref mut menu_bar) = app.menu_bar {
                menu_bar.handle_event(&mut event);
                if event.what == EventType::Keyboard || event.what == EventType::MouseUp {
                    if let Some(command) = menu_bar.check_cascading_submenu(&mut app.terminal) {
                        if command != 0 {
                            event = turbo_vision::core::event::Event::command(command);
                        }
                    }
                }
            }

            // Phase 2: Focused
            app.desktop.handle_event(&mut event);

            // Clean up closed windows
            app.desktop.remove_closed_windows();

            // Application commands
            if event.what == EventType::Command {
                match event.command {
                    CM_QUIT => {
                        app.running = false;
                    }
                    CM_NEW => {
                        new_editor_window(&mut app);
                    }
                    CM_OPEN => {
                        open_window(&mut app);
                    }
                    CM_ABOUT => {
                        show_about_box(&mut app);
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn new_editor_window(app: &mut Application) {
    let window_bounds = Rect::new(5, 3, 75, 20);
    let mut window = Window::new(window_bounds, "Untitled");

    let editor_bounds = Rect::new(1, 1, 69, 16);
    let editor = Editor::new(editor_bounds).with_scrollbars_and_indicator();

    window.add(Box::new(editor));
    app.desktop.add(Box::new(window));
}

fn open_window(app: &mut Application) {
    // Create file dialog
    let (term_width, term_height) = app.terminal.size();
    let dialog_width = 60;
    let dialog_height = 20;
    let dialog_x = (term_width as i16 - dialog_width) / 2;
    let dialog_y = (term_height as i16 - dialog_height) / 2;

    let mut file_dialog = FileDialog::new(
        Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height),
        "Open File",
        "*",
        None,
    ).build();

    // Execute dialog
    if let Some(path) = file_dialog.execute(app) {
        match fs::read_to_string(&path) {
            Ok(content) => {
                let window_bounds = Rect::new(5, 3, 75, 20);
                let title = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Untitled");
                let mut window = Window::new(window_bounds, title);

                let editor_bounds = Rect::new(1, 1, 69, 16);
                let mut editor = Editor::new(editor_bounds).with_scrollbars_and_indicator();
                editor.set_text(&content);

                window.add(Box::new(editor));
                app.desktop.add(Box::new(window));
            }
            Err(_) => {
                message_box_ok(app, "Error", "Failed to open file");
            }
        }
    }
}

fn create_menu_bar(width: u16) -> MenuBar {
    let mut menu_bar = MenuBar::new(Rect::new(0, 0, width as i16, 1));

    let file_menu_items = vec![
        MenuItem::with_shortcut("~N~ew", CM_NEW, 0, "", 0),
        MenuItem::with_shortcut("~O~pen...", CM_OPEN, KB_F3, "F3", 0),
        MenuItem::separator(),
        MenuItem::with_shortcut("E~x~it", CM_QUIT, KB_ALT_X, "Alt+X", 0),
    ];
    let file_menu = SubMenu::new("~F~ile", Menu::from_items(file_menu_items));
    menu_bar.add_submenu(file_menu);

    let help_menu_items = vec![
        MenuItem::with_shortcut("~A~bout", CM_ABOUT, KB_F1, "F1", 0),
    ];
    let help_menu = SubMenu::new("~H~elp", Menu::from_items(help_menu_items));
    menu_bar.add_submenu(help_menu);

    menu_bar
}

fn create_status_line(height: u16, width: u16) -> StatusLine {
    StatusLine::new(
        Rect::new(0, height as i16 - 1, width as i16, height as i16),
        vec![
            StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
            StatusItem::new("~F3~ Open", KB_F3, CM_OPEN),
        ],
    )
}

fn show_about_box(app: &mut Application) {
    message_box_ok(
        app,
        "About",
        "  Turbo Vision Tutorial App 3.0\n\n  Rust Edition"
    );
}
```

---

## Summary

In this chapter, you learned:

- How to create and add windows to the desktop
- How windows are managed by the desktop (Z-order, focus, activation)
- How to create editor windows with full text editing capabilities
- How to use FileDialog to let users select files
- How to load files into editor windows
- How to add child views to windows
- How coordinate systems work (screen vs. window-relative)

### Key Differences from Pascal

| Pascal | Rust (v0.2.11) |
|--------|----------------|
| `TWindow` | `Window` struct |
| `New(PWindow, Init(R, title, number))` | `Window::new(bounds, title)` |
| `Desktop^.Insert(window)` | `app.desktop.add(Box::new(window))` |
| `PEditWindow` | `Window` + `Editor` |
| `TFileDialog` | `FileDialog` |
| `ExecuteDialog(dialog, @data)` | `dialog.execute(app)` returns `Option<PathBuf>` |
| Raw pointers | Rust ownership with `Box<dyn View>` |
| Window numbers (Alt+1, Alt+2) | Not yet implemented in Rust version |
| Tile/Cascade | Not yet implemented in Rust version |

### Best Practices

1. **Clean up closed windows** - Call `app.desktop.remove_closed_windows()` after handling events
2. **Use relative coordinates** - Child view coordinates are relative to their parent window
3. **Check file operations** - Always handle `Result` from file I/O operations
4. **Use descriptive titles** - Window titles should indicate content ("Untitled" vs. filename)
5. **Center dialogs** - Position modal dialogs in the center of the screen
6. **Add scrollbars to editors** - Use `with_scrollbars_and_indicator()` for full editor functionality

---

## See Also

- **Chapter 1** - Stepping into Turbo Vision (basics, event loop)
- **Chapter 2** - Responding to Commands (menus, status lines, commands)
- **docs/TURBOVISION-DESIGN.md** - Complete architecture documentation
- **examples/editor_demo.rs** - Comprehensive editor examples
- **examples/file_dialog.rs** - FileDialog usage examples
- **examples/window_resize_demo.rs** - Window manipulation examples

---

In the next chapter, you'll learn about **data entry** and **controls**, creating forms with input fields, checkboxes, radio buttons, and more interactive UI elements.
