# Chapter 2 — Responding to Commands

**Previous:** [Chapter 1 — Stepping into Turbo Vision](Chapter-1-Stepping-into-Turbo-Vision.md)

---

In Chapter 1, you built a minimal Turbo Vision application. In this chapter, you'll learn how to make it **respond to user commands**, such as keystrokes or menu selections. You'll extend the tutorial application by adding an **About box** and handling custom commands, and learn how to enable or disable menu items dynamically.

---

## Understanding Events and Commands

Turbo Vision applications are **event-driven**. The framework generates events to signal that something has happened — for example, the user pressed a key, clicked the mouse, or chose a menu command.

Your application's job is to **respond to those events**.

Events are represented by the `Event` struct (defined in `src/core/event.rs`) and contain the following important fields:

- `what` — describes the kind of event (`EventType` enum: keyboard, mouse, command, broadcast, etc.)
- `command` — specifies which command was issued (of type `CommandId`)
- `key_code` — keyboard scan code and character (for keyboard events)
- `mouse` — mouse position and button state (for mouse events)

Keyboard and mouse input are transformed by Turbo Vision into high-level commands such as `CM_QUIT`, `CM_NEW`, or `CM_ABOUT`.

### Event Types

The `EventType` enum defines the kinds of events your application can handle:

```rust
pub enum EventType {
    Nothing,        // No event
    Keyboard,       // Key press
    MouseDown,      // Mouse button pressed
    MouseUp,        // Mouse button released
    MouseMove,      // Mouse moved
    MouseAuto,      // Mouse auto-repeat
    MouseWheelUp,   // Mouse wheel scrolled up
    MouseWheelDown, // Mouse wheel scrolled down
    Command,        // High-level command
    Broadcast,      // Broadcast message to all views
}
```

### Command Identifiers

Commands are represented as `u16` values (type alias `CommandId`). Turbo Vision defines standard commands in `src/core/command.rs`:

```rust
// Standard commands
pub const CM_QUIT: CommandId = 24;
pub const CM_CLOSE: CommandId = 25;
pub const CM_OK: CommandId = 10;
pub const CM_CANCEL: CommandId = 11;
pub const CM_YES: CommandId = 12;
pub const CM_NO: CommandId = 13;

// Custom commands (user-defined)
pub const CM_ABOUT: CommandId = 100;
pub const CM_NEW: CommandId = 102;
pub const CM_OPEN: CommandId = 103;
pub const CM_SAVE: CommandId = 104;
// ... and more
```

You can define your own custom commands by choosing values outside the standard range (typically 100+).

---

## Step 3 — Responding to Commands

You'll now add your first custom command handler by extending the `Application` struct.

In Rust, we don't use inheritance like the original Pascal/C++ Turbo Vision. Instead, we handle custom commands by:
1. Catching unhandled command events from the application's event loop
2. Processing them in the main application logic
3. Or creating custom views that handle specific commands

### Listing 2.1 — Handling Commands in Main Event Loop

Here's a simple example that displays an "About" message box when the user triggers the `CM_ABOUT` command:

```rust
use turbo_vision::prelude::*;
use turbo_vision::app::Application;
use turbo_vision::core::event::{Event, EventType};
use turbo_vision::core::command::{CM_QUIT, CM_ABOUT};
use turbo_vision::views::msgbox::{message_box, MF_INFORMATION, MF_OK_BUTTON};
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;

    // Application loop
    loop {
        // Draw the screen
        app.draw();
        app.terminal.flush()?;

        // Get next event
        if let Ok(Some(mut event)) = app.terminal.poll_event(Duration::from_millis(50)) {
            // Let the application handle standard events
            app.handle_event(&mut event);

            // Check if we need to quit
            if !app.running {
                break;
            }

            // Handle custom commands
            if event.what == EventType::Command {
                match event.command {
                    CM_ABOUT => {
                        message_box(
                            &mut app,
                            "Turbo Vision Tutorial\n\nRust Edition 2.0",
                            MF_INFORMATION | MF_OK_BUTTON
                        );
                        event.clear(); // Mark event as handled
                    }
                    _ => {}
                }
            }
        }

        // Idle processing (broadcast command set changes)
        app.idle();
    }

    Ok(())
}
```

This example adds an `About` command handler. When the user chooses **About**, the application displays a message box using the `message_box` function.

**Key points:**
- `app.handle_event(&mut event)` handles standard application events (quit, etc.)
- We check `event.what == EventType::Command` to handle command events
- `event.clear()` marks the event as handled (sets `what` to `EventType::Nothing`)
- The `message_box` function creates and executes a modal dialog

> **⚠️ Important Note:** This listing shows the **command handling code**, but doesn't yet provide a way for the user to trigger `CM_ABOUT`. The desktop is empty with no menu or buttons. This is intentional — we're focusing on the event handling pattern first. In **Listing 2.2** below, you'll add a menu bar that lets the user actually trigger this command. For now, understand that this code is *ready* to handle `CM_ABOUT` whenever it arrives (which will happen once we add the menu in the next section).

---

## Step 4 — Customizing Menus and Status Lines

By default, Turbo Vision applications display an empty desktop. You'll now customize your application by adding a menu bar and status line **that will trigger the `CM_ABOUT` command** we just learned to handle.

This completes the loop:
- **Listing 2.1** showed how to *handle* commands
- **Listing 2.2** (below) shows how to *trigger* commands through menus

### Understanding Menu Data Structures

Turbo Vision Rust uses declarative data structures for building menus, closely matching Borland's architecture:

- `Menu` — A collection of menu items (matches Borland's `TMenu`)
- `MenuItem` — A single menu item, which can be:
  - **Regular** — Executes a command
  - **SubMenu** — Opens a nested menu
  - **Separator** — Visual divider
- `MenuBuilder` — Fluent builder for constructing menus

### Listing 2.2 — Adding a Menu Bar and Status Line

```rust
use turbo_vision::prelude::*;
use turbo_vision::app::Application;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::command::*;
use turbo_vision::core::event::*;
use turbo_vision::core::menu_data::{Menu, MenuItem, MenuBuilder};
use turbo_vision::core::status_data::{StatusLine, StatusItem};
use turbo_vision::views::menu_bar::MenuBar;
use turbo_vision::views::status_line::StatusLine as StatusLineView;
use turbo_vision::views::msgbox::{message_box, MF_INFORMATION, MF_OK_BUTTON};

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Create menu bar with File, Options, and Help menus
    let file_menu = MenuBuilder::new()
        .item_with_shortcut("E~x~it", CM_QUIT, KB_ALT_X, "Alt+X")
        .build();

    let options_menu = MenuBuilder::new()
        .item("~V~ideo Mode", CM_VIDEO_MODE, 0)
        .build();

    let help_menu = MenuBuilder::new()
        .item_with_shortcut("~A~bout", CM_ABOUT, KB_F1, "F1")
        .build();

    // Create top-level menu bar
    let menu_bar_menus = vec![
        MenuItem::submenu("~F~ile", KB_ALT_F, file_menu, 0),
        MenuItem::submenu("~O~ptions", KB_ALT_O, options_menu, 0),
        MenuItem::submenu("~H~elp", KB_ALT_H, help_menu, 0),
    ];

    let menu_bar = MenuBar::new(
        Rect::new(0, 0, width as i16, 1),
        Menu::from_items(menu_bar_menus)
    );
    app.set_menu_bar(menu_bar);

    // Create status line
    let status_line = StatusLine::single(vec![
        StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
    ]);

    let status_line_view = StatusLineView::new(
        Rect::new(0, height as i16 - 1, width as i16, height as i16),
        status_line
    );
    app.set_status_line(status_line_view);

    // Main event loop
    loop {
        app.draw();
        app.terminal.flush()?;

        if let Ok(Some(mut event)) = app.terminal.poll_event(std::time::Duration::from_millis(50)) {
            app.handle_event(&mut event);

            if !app.running {
                break;
            }

            // Handle custom commands
            if event.what == EventType::Command {
                match event.command {
                    CM_ABOUT => {
                        message_box(
                            &mut app,
                            "Turbo Vision Tutorial\n\nRust Edition 2.0",
                            MF_INFORMATION | MF_OK_BUTTON
                        );
                        event.clear();
                    }
                    CM_VIDEO_MODE => {
                        // Video mode toggle would go here
                        message_box(
                            &mut app,
                            "Video mode toggle",
                            MF_INFORMATION | MF_OK_BUTTON
                        );
                        event.clear();
                    }
                    _ => {}
                }
            }
        }

        app.idle();
    }

    Ok(())
}
```

### Breaking Down the Menu Construction

**Menu Builder Pattern:**
```rust
let file_menu = MenuBuilder::new()
    .item_with_shortcut("E~x~it", CM_QUIT, KB_ALT_X, "Alt+X")
    .build();
```
- Use `~x~` to mark the accelerator key (shows as underlined)
- `item_with_shortcut()` displays a keyboard shortcut in the menu
- `build()` creates the final `Menu` structure

**Top-Level Menu Items:**
```rust
let menu_bar_menus = vec![
    MenuItem::submenu("~F~ile", KB_ALT_F, file_menu, 0),
    // ...
];
```
- Each top-level item is a submenu
- `KB_ALT_F` allows opening the menu with Alt+F

**Status Line:**
```rust
let status_line = StatusLine::single(vec![
    StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
]);
```
- `StatusLine::single()` creates a simple status line with one definition
- For context-sensitive status lines, use `StatusLineBuilder`

### Running the Program

When you run this version of the program:

- A **menu bar** appears with three main menus: *File*, *Options*, and *Help*
- Choosing **Help → About** (or pressing **F1**) triggers the `CM_ABOUT` command
- The command handler from Listing 2.1 displays the message box
- The status line shows `Alt+X Exit` at the bottom
- You can navigate menus with mouse or keyboard (Alt+F, Alt+O, Alt+H)

> **💡 Now It All Makes Sense:** In Listing 2.1, we set up the command handler. In Listing 2.2, we added the menu that triggers it. When you select **Help → About** (or press **F1**), the menu generates a `CM_ABOUT` command event, which flows through the event loop and gets caught by our handler. This is the core pattern of event-driven programming in Turbo Vision!

---

## Enabling and Disabling Commands

Turbo Vision allows you to dynamically enable or disable menu items and status line hints. This is useful when certain actions are unavailable (for example, *Paste* when the clipboard is empty, or *Save* when there are no changes).

### How Command Sets Work

Turbo Vision maintains a global **command set** that tracks which commands are currently enabled. The system uses this to:
- Gray out disabled menu items
- Hide disabled status line hints
- Prevent disabled commands from executing

The command set is managed through the `Application` methods:

```rust
// Enable a command
app.enable_command(CM_SAVE);

// Disable a command
app.disable_command(CM_PASTE);

// Check if enabled
if app.command_enabled(CM_SAVE) {
    // Command is available
}
```

### Command Set Change Broadcasting

When commands are enabled or disabled, Turbo Vision automatically broadcasts a `CM_COMMAND_SET_CHANGED` message to all views during idle processing. This ensures:
- Menu bars update their display
- Status lines show/hide appropriate hints
- All views stay synchronized with the current command availability

This happens automatically in the `app.idle()` call in your event loop.

### Example: Dynamic Command Management

```rust
use turbo_vision::core::command::*;

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;

    // Initially disable save (no document loaded)
    app.disable_command(CM_SAVE);
    app.disable_command(CM_SAVE_AS);

    // Enable them when a document is loaded
    // (In your event handling logic)
    if document_loaded {
        app.enable_command(CM_SAVE);
        app.enable_command(CM_SAVE_AS);
    }

    // Disable paste when clipboard is empty
    if clipboard.is_empty() {
        app.disable_command(CM_PASTE);
    } else {
        app.enable_command(CM_PASTE);
    }

    // ... rest of event loop

    Ok(())
}
```

### Context-Sensitive Status Lines

For more advanced applications, you can create context-sensitive status lines that display different hints based on which view has focus:

```rust
use turbo_vision::core::status_data::StatusLineBuilder;

let status_line = StatusLineBuilder::new()
    // Default status (all contexts)
    .add_default_def(vec![
        StatusItem::new("~F1~ Help", KB_F1, CM_HELP),
        StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
    ])
    // Editor context (commands 100-199)
    .add_def(100, 199, vec![
        StatusItem::new("~F1~ Help", KB_F1, CM_HELP),
        StatusItem::new("~F2~ Save", KB_F2, CM_SAVE),
        StatusItem::new("~F3~ Open", KB_F3, CM_OPEN),
        StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
    ])
    // Dialog context (commands 200-299)
    .add_def(200, 299, vec![
        StatusItem::new("~F1~ Help", KB_F1, CM_HELP),
        StatusItem::new("~Tab~ Next", KB_TAB, CM_NEXT),
        StatusItem::new("~Esc~ Cancel", KB_ESC, CM_CANCEL),
    ])
    .build();
```

The status line automatically switches between definitions based on the focused view's help context.

---

## Architecture Notes: From C++ to Rust

The Rust implementation of Turbo Vision maintains the same event-driven architecture as the original Borland version, but adapts it to Rust's ownership and type system:

### Event Handling Pattern

**Original Borland (C++):**
```cpp
void TTutorApp::handleEvent(TEvent& event) {
    TApplication::handleEvent(event);  // Call parent
    if (event.what == evCommand) {
        // Handle commands
        clearEvent(event);
    }
}
```

**Rust Implementation:**
```rust
// Events are handled in the main loop
app.handle_event(&mut event);

// Then check for unhandled custom commands
if event.what == EventType::Command {
    match event.command {
        CM_ABOUT => { /* handle */ event.clear(); }
        _ => {}
    }
}
```

**Key differences:**
- Rust uses composition rather than inheritance
- Events are passed by mutable reference (`&mut`)
- `event.clear()` replaces `clearEvent()`
- Pattern matching (`match`) replaces `switch/case`

### Command Set Management

**Original Borland:** Command sets were stored as static member variables in `TView`

**Rust Implementation:** Uses thread-local storage via the `command_set` module
- Matches Borland's architecture where `TView::curCommandSet` is static
- Provides global command state accessible to all views
- Automatically broadcasts changes during idle processing

### Menu Construction

**Original Borland:** Used linked lists of `TMenuItem` with manual memory management

**Rust Implementation:** Uses `Vec<MenuItem>` with enum-based item types
- Type-safe (no raw pointers)
- Declarative builders for ergonomic construction
- Automatic memory management
- Same conceptual model, safer implementation

---

## Summary

In this chapter, you learned:

- How Turbo Vision events and commands work in Rust
- The `Event` struct and `EventType` enum
- How to handle commands in the main event loop
- How to create menu bars using `MenuBuilder`
- How to create status lines with `StatusLine` and `StatusItem`
- How to enable or disable commands dynamically
- How command set changes are automatically broadcast to all views

### Key Types and Functions

| Type/Function | Purpose | Module |
|--------------|---------|---------|
| `Event` | Represents a user or system event | `core::event` |
| `EventType` | Enum of event kinds | `core::event` |
| `CommandId` | Type alias for command identifiers (`u16`) | `core::command` |
| `Menu` | Collection of menu items | `core::menu_data` |
| `MenuItem` | Single menu item (regular, submenu, or separator) | `core::menu_data` |
| `MenuBuilder` | Fluent builder for menus | `core::menu_data` |
| `StatusLine` | Status line definition | `core::status_data` |
| `StatusItem` | Single status line item | `core::status_data` |
| `message_box()` | Display a modal message dialog | `views::msgbox` |
| `app.enable_command()` | Enable a command | `app::Application` |
| `app.disable_command()` | Disable a command | `app::Application` |

### Next Steps

In the next chapter, you'll learn how to **add windows** to your Turbo Vision application, creating movable, resizable containers for your views.

---

## See Also

- **Chapter 1**: Creating a basic Turbo Vision application
- **Chapter 8**: Views and Groups (detailed event processing)
- **Chapter 9**: Event-Driven Programming (advanced event handling)
- **Example Code**: `examples/menu_status_data.rs` — Demonstrates menu and status line construction
- **Example Code**: `examples/command_set_demo.rs` — Shows command enable/disable in action

---

**Next:** [Chapter 3 — Adding Windows](Chapter-3-Adding-Windows.md)
