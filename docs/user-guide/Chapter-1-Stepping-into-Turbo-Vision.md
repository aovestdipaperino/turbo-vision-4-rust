# Chapter 1 — Stepping into Turbo Vision (Rust Edition)

**Version:** 0.2.11
**Updated:** 2025-11-04

In the next several chapters, you'll build a complete Turbo Vision application, starting from the very simplest instance of the bare framework and working up to a fairly complex data-entry system with input validation and context-sensitive prompts.

The walk-through consists of twelve steps:

1. Step 1: Creating an application
2. Step 2: Customizing menus and status lines
3. Step 3: Responding to commands
4. Step 4: Adding a window
5. Step 5: Adding a clipboard window
6. Step 6: Saving and loading the desktop
7. Step 7: Using resources
8. Step 8: Creating a data-entry window
9. Step 9: Setting control values
10. Step 10: Validating data entry
11. Step 11: Adding collections of data
12. Step 12: Creating a custom view

The source code for the application in this tutorial is provided in the `examples/` directory. The files are named `tutorial_01.rs`, `tutorial_02.rs`, and so on, corresponding to the numbered steps in the tutorial.

---

## What's in a Turbo Vision Application?

Before you start building your first Turbo Vision application, let's take a look at what Turbo Vision gives you to build your applications.

A Turbo Vision application is a cooperating society of **views**, **events**, and **engines**.

### Views

A view is any program element that is visible on the screen—and all such elements implement the `View` trait.
In a Turbo Vision context, if you can see it, it's a view. Fields, field captions, window borders, scroll bars, menu bars, and push buttons are all views. Views can be combined to form more complex elements like windows and dialog boxes. These collective views are called **groups**, and they operate together as though they were a single view. Conceptually, groups may be considered views.

Views are always rectangular. This includes rectangles that contain a single character, or lines which are only one character high or one character wide.

In Rust, views are trait objects that implement the `View` trait:

```rust
pub trait View {
    fn bounds(&self) -> Rect;
    fn set_bounds(&mut self, bounds: Rect);
    fn draw(&mut self, terminal: &mut Terminal);
    fn handle_event(&mut self, event: &mut Event);
    fn can_focus(&self) -> bool;
    fn set_focus(&mut self, focused: bool);
    fn state(&self) -> StateFlags;
    fn options(&self) -> OptionsFlags;
}
```

### Events

An event is some sort of occurrence to which your application must respond.
Events come from the keyboard, from the mouse, or from other parts of Turbo Vision. For example, a keystroke is an event, as is a click of a mouse button. Events are queued by Turbo Vision's application skeleton as they occur, then processed in order by an event handler.

The `Application` struct, which is the body of your application, contains an event handler.
Events not serviced by `Application` are passed along to other views owned by the program until either a view is found to handle the event or the event is cleared (marked as consumed).

For example, an **F1 keystroke** can invoke a menu or help system. Unless each view has its own response to F1, the F1 keystroke is handled by the main program's event handler. Alphanumeric keys or line-editing keys, by contrast, need to be handled by the view that currently has the focus.

Events in Rust are represented by the `Event` struct:

```rust
pub struct Event {
    pub what: EventType,
    pub key_code: u16,
    pub command: u16,
    pub mouse: MouseEvent,
    pub key_modifiers: KeyModifiers,
}

pub enum EventType {
    Nothing,        // No event or consumed
    Keyboard,       // Keyboard input
    MouseDown,      // Mouse button press
    MouseUp,        // Mouse button release
    MouseMove,      // Mouse movement
    MouseDrag,      // Mouse drag
    Command,        // High-level command
    Broadcast,      // Broadcast message
}
```

### Three-Phase Event Processing

**New in v0.1.9** - Turbo Vision implements three-phase event processing matching Borland's architecture:

1. **PreProcess Phase** - Views with `OF_PRE_PROCESS` flag (e.g., StatusLine)
2. **Focused Phase** - Currently focused view
3. **PostProcess Phase** - Views with `OF_POST_PROCESS` flag (e.g., Button hotkeys)

This allows views to intercept events before or after the focused view processes them.

### Engines

Engines, sometimes called "mute objects," are any other objects in the program that are not views. They don't speak to the screen themselves. They perform calculations, communicate with peripherals, and generally do the work of the application. When an engine needs to display some output to the screen, it must do so through a cooperating view.

This concept is important for keeping order in a Turbo Vision application: **Only views may access the display.**
Writing directly to the terminal with standard output (like `println!`) would disrupt Turbo Vision's output and cause visual corruption. All terminal operations must go through the `Terminal` abstraction provided by the framework.

---

## Step 1 — Creating an Application

The usual way to get started with a new library is to write the simplest program possible, such as a very short one that displays "Hello, World!".
In this step, you'll:

- Create the absolute minimum Turbo Vision program
- Extend the basic application with proper event handling

The application object provides the framework on which you'll build a real application. The simplest Turbo Vision program is just an instance of the base `Application` struct.

### Listing 1.1 — The Simplest Turbo Vision Program

```rust
// minimal.rs
use turbo_vision::app::Application;

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;
    app.run();
    Ok(())
}
```

If you run this program, you'll see a screen with a blank desktop area (filled with a pattern character). The application responds to standard exit keys:

- **F10** - Exit the application
- **Ctrl+C** - Exit the application
- **Alt+X** - Exit the application (on systems that support it)
- **ESC X** - Exit the application (ESC sequence alternative)

### Understanding the Application Lifecycle

The Rust version of Turbo Vision uses modern patterns:

1. **`Application::new()`** - Initializes the terminal, sets up the desktop, and prepares the application framework. This is analogous to the Pascal `Init` method.

2. **`app.run()`** - Enters the main event loop, processing user input and drawing the interface. This is analogous to the Pascal `Run` method.

3. **Automatic cleanup** - Rust's `Drop` trait handles cleanup automatically when the `Application` goes out of scope. This replaces the Pascal `Done` method.

```rust
impl Drop for Application {
    fn drop(&mut self) {
        let _ = self.terminal.shutdown();
    }
}
```

### Custom Event Loop (Recommended for Applications)

For more control over event handling and to implement the proper three-phase event processing, use a custom event loop:

```rust
// tutorial_01.rs - Proper event loop with three-phase processing
use turbo_vision::app::Application;
use turbo_vision::core::event::{EventType, KB_F10};
use turbo_vision::core::command::CM_QUIT;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;

    // Custom event loop with proper three-phase event processing
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

        // Poll for events
        if let Ok(Some(mut event)) = app.terminal.poll_event(Duration::from_millis(50)) {
            // Phase 1: PreProcess - Status line handles events first
            // StatusLine has OF_PRE_PROCESS flag set
            if let Some(ref mut status_line) = app.status_line {
                status_line.handle_event(&mut event);
            }

            // Special handling: Menu bar (F10 and Alt keys)
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

            // Phase 2: Focused - Desktop (and its children) handle events
            app.desktop.handle_event(&mut event);

            // Phase 3: PostProcess happens inside Desktop/Group
            // Buttons with OF_POST_PROCESS intercept their hotkeys

            // Application-level event handling
            if event.what == EventType::Command {
                match event.command {
                    CM_QUIT => {
                        app.running = false;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
```

This custom event loop approach gives you full control over:
- Three-phase event processing (PreProcess → Focused → PostProcess)
- When and how the screen is drawn
- The order in which views handle events
- Application-level command processing
- Application state management

### Creating a Command Module

Turbo Vision commands are unsigned 16-bit integer constants (`u16`). The easiest way to handle them is to create a separate module containing only constant definitions.

### Listing 1.3 — `commands.rs`, a Module Defining Command Constants

```rust
// commands.rs - Command constant definitions
// Re-export commonly used commands from core
pub use turbo_vision::core::command::{
    CM_QUIT, CM_OK, CM_CANCEL, CM_YES, CM_NO, CM_CLOSE,
    CM_NEW, CM_OPEN, CM_SAVE,
};

// Application-specific commands
pub const CM_ORDER_NEW: u16 = 251;
pub const CM_ORDER_WIN: u16 = 252;
pub const CM_ORDER_SAVE: u16 = 253;
pub const CM_ORDER_CANCEL: u16 = 254;
pub const CM_ORDER_NEXT: u16 = 255;
pub const CM_ORDER_PREV: u16 = 250;

pub const CM_CLIP_SHOW: u16 = 260;
pub const CM_ABOUT: u16 = 270;
pub const CM_FIND_ORDER_WINDOW: u16 = 2000;

pub const CM_OPTIONS_VIDEO: u16 = 1502;
pub const CM_OPTIONS_SAVE: u16 = 1503;
pub const CM_OPTIONS_LOAD: u16 = 1504;

// Custom commands for search/replace
pub const CMD_SEARCH: u16 = 300;
pub const CMD_REPLACE: u16 = 301;
pub const CMD_GOTO_LINE: u16 = 302;
```

Using this in your main application:

```rust
// main.rs
mod commands;
use commands::*;

use turbo_vision::app::Application;
use turbo_vision::core::event::EventType;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;

    // ... setup code ...

    app.running = true;
    while app.running {
        // ... draw code ...

        if let Ok(Some(mut event)) = app.terminal.poll_event(Duration::from_millis(50)) {
            // ... event handling ...

            if event.what == EventType::Command {
                match event.command {
                    CM_QUIT => {
                        app.running = false;
                    }
                    CM_ABOUT => {
                        show_about_dialog(&mut app);
                    }
                    CM_ORDER_NEW => {
                        create_new_order(&mut app);
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
```

Keeping constants in a separate module provides:
- A single, central location for all command constants
- Avoids duplication and naming conflicts
- Makes it easy to see all available commands at a glance
- Follows Rust's module system conventions

### Rust-Specific Considerations

**Error Handling**

Unlike Pascal, Rust requires explicit error handling. The `Application::new()` method returns a `Result`:

```rust
fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;  // ? propagates errors
    // ... rest of code ...
    Ok(())
}
```

**Ownership and Borrowing**

Views are typically owned by their parent containers using `Box<dyn View>`:

```rust
// Adding a view to a dialog
dialog.add(Box::new(Button::new(
    Rect::new(10, 5, 20, 7),
    "OK",
    CM_OK,
    true  // is_default
)));
```

**Shared State**

For shared mutable state (like input field data), use `Rc<RefCell<T>>`:

```rust
use std::rc::Rc;
use std::cell::RefCell;

let data = Rc::new(RefCell::new(String::new()));

// Share with an input field
let input = InputLine::new(
    Rect::new(10, 5, 40, 6),
    50,  // max_length
    data.clone()
);

// Later, read the data
let value = data.borrow().clone();
```

---

## Complete Example: Minimal Application with Menu and Status Line

Here's a complete, working example that combines all the concepts with proper three-phase event processing:

```rust
// tutorial_01_complete.rs
use turbo_vision::app::Application;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::event::{EventType, KB_F10, KB_ALT_X};
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
use turbo_vision::views::status_line::{StatusLine, StatusItem};
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Create menu bar
    let mut menu_bar = MenuBar::new(Rect::new(0, 0, width as i16, 1));

    // File menu
    let file_menu_items = vec![
        MenuItem::with_shortcut("~Q~uit", CM_QUIT, 0, "Alt+X", 0),
    ];
    let file_menu = SubMenu::new("~F~ile", Menu::from_items(file_menu_items));
    menu_bar.add_submenu(file_menu);

    app.set_menu_bar(menu_bar);

    // Create status line (with OF_PRE_PROCESS flag set automatically)
    let status_line = StatusLine::new(
        Rect::new(0, height as i16 - 1, width as i16, height as i16),
        vec![
            StatusItem::new("~F10~ Menu", KB_F10, 0),
            StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
        ],
    );
    app.set_status_line(status_line);

    // Run the application with proper three-phase event processing
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
            // Phase 1: PreProcess - Status line handles events first (OF_PRE_PROCESS)
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
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
```

To build and run:

```bash
cargo build
cargo run --example tutorial_01_complete
```

---

At this point, your minimal Turbo Vision application runs, displays its framework, and exits cleanly.
In the next step, you'll **customize menus and status lines** to make the interface more useful and add command handling to respond to user actions.

## Key Differences from Pascal Version

1. **No explicit Init/Done**: Rust uses constructors (`new()`) and the `Drop` trait for cleanup
2. **Error handling**: Functions return `Result<T, E>` instead of using exceptions
3. **Ownership**: Views are owned via `Box<dyn View>` trait objects
4. **Shared state**: Use `Rc<RefCell<T>>` for shared mutable data
5. **Module system**: Commands defined in modules instead of units
6. **Type safety**: Stronger type checking and trait-based polymorphism
7. **No global state**: Terminal and application state passed explicitly
8. **Three-phase processing**: Explicit PreProcess → Focused → PostProcess event flow
9. **StatusLine PreProcess**: StatusLine intercepts events first with OF_PRE_PROCESS flag

## New in Recent Versions

### v0.1.9 - Three-Phase Event Processing
- PreProcess phase for views like StatusLine
- PostProcess phase for buttons to intercept hotkeys
- Matches Borland's TGroup::handleEvent() exactly

### v0.2.11 - StatusLine Improvements
- StatusLine now uses OF_PRE_PROCESS flag (matches Borland tstatusl.cc:33)
- Fixed highlighting with proper spacing around text
- Separator "│ " always drawn in normal color
- Click detection includes leading and trailing spaces

### Standard Library Dialogs
- `message_box_ok()` - Simple information message
- `message_box_error()` - Error message
- `confirmation_box()` - Yes/No/Cancel dialog
- `search_box()` - Search text input
- `search_replace_box()` - Find and replace
- `goto_line_box()` - Go to line number

See `src/views/msgbox.rs` for all available dialog functions.

## Summary

You've learned:

- The architecture of a Turbo Vision application (views, events, engines)
- How to create a minimal Turbo Vision application in Rust
- The three-phase event processing system (PreProcess → Focused → PostProcess)
- How to implement a proper custom event loop with correct event ordering
- How to define command constants in a separate module
- Rust-specific patterns for ownership, error handling, and state management
- The complete structure of a working Turbo Vision application
- How StatusLine uses OF_PRE_PROCESS to intercept events first

The foundation is now in place to build more sophisticated applications with menus, dialogs, windows, and custom views.

## See Also

- **docs/TURBOVISION-DESIGN.md** - Complete architecture documentation
- **docs/TURBOVISION-DESIGN.md#class-hierarchy-and-architecture** - Class hierarchy diagrams
- **docs/TURBOVISION-DESIGN.md#event-system-architecture** - Three-phase event processing
- **docs/TURBOVISION-DESIGN.md#syntax-highlighting-system** - Editor with syntax highlighting
- **demo/rust_editor.rs** - Complete working example of all concepts
- **examples/README.md** - Guide to all 16 examples
