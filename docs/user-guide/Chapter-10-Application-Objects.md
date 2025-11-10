# Chapter 10: Application Objects

The `Application` struct is the foundation of every Turbo Vision program. It manages the terminal interface, coordinates the user interface components (menu bar, status line, and desktop), and controls the application's main event loop. Understanding the application object is essential for building complete Turbo Vision applications.

This chapter covers:
- Creating and initializing an application
- The application event loop
- Modal view execution
- Idle processing and command set management
- Context-sensitive help

---

## The Application Structure

The `Application` struct is defined in `src/app/application.rs` and serves as the top-level container for your application. It manages:

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

Key components:

- **terminal** — Manages the physical terminal (screen, keyboard, mouse)
- **menu_bar** — Optional menu bar at the top of the screen
- **status_line** — Optional status line at the bottom showing keyboard hints
- **desktop** — The main workspace containing windows and dialogs
- **running** — Controls whether the application continues its event loop
- **needs_redraw** — Internal flag tracking when a full screen redraw is required

---

## Creating an Application

Every Turbo Vision application begins by creating an `Application` instance:

```rust
use turbo_vision::app::Application;

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;

    // Configure the application...

    Ok(())
}
```

The `Application::new()` method:
1. Initializes the terminal in raw mode
2. Creates the desktop with appropriate bounds (leaving room for menu/status)
3. Initializes the global command set
4. Sets up initial state

The desktop is automatically sized to fit between the menu bar (row 0) and status line (bottom row), occupying rows 1 through height-2.

---

## Adding Application Components

After creating the application, you typically add a menu bar and status line:

### Adding a Menu Bar

```rust
use turbo_vision::views::menu_bar::MenuBar;
use turbo_vision::core::menu_data::*;
use turbo_vision::core::command::*;
use turbo_vision::core::event::*;

let menu = MenuBuilder::new()
    .item(MenuItem::submenu(
        "~F~ile",
        KB_ALT_F,
        Menu::new(vec![
            MenuItem::new("~O~pen", CM_OPEN, KB_F3, 0),
            MenuItem::new("~S~ave", CM_SAVE, KB_F2, 0),
            MenuItem::separator(),
            MenuItem::new("E~x~it", CM_QUIT, KB_ALT_X, 0),
        ]),
        0,
    ))
    .build();

let menu_bar = MenuBar::new(menu);
app.set_menu_bar(menu_bar);
```

The menu bar is drawn at row 0 and intercepts keyboard events for menu navigation.

### Adding a Status Line

```rust
use turbo_vision::views::status_line::*;

let status_items = vec![
    StatusItem::new("~F10~ Menu", KB_F10, 0),
    StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
];

let status_line = StatusLine::new(
    Rect::new(0, height - 1, width, height),
    status_items
);

app.set_status_line(status_line);
```

The status line displays keyboard shortcuts and can show context-sensitive hints.

---

## The Application Event Loop

The heart of every Turbo Vision application is the event loop, controlled by the `run()` method:

```rust
fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;

    // Set up menu bar, status line, add windows...

    app.run();  // Enter the event loop

    Ok(())
}
```

### How the Event Loop Works

The `run()` method (defined in `src/app/application.rs:127-188`) implements the main application loop:

```rust
pub fn run(&mut self) {
    self.running = true;

    // Initial draw
    self.update_active_view_bounds();
    self.draw();
    let _ = self.terminal.flush();

    while self.running {
        // Poll for events (keyboard, mouse)
        if let Ok(Some(mut event)) = self.terminal.poll_event(Duration::from_millis(50)) {
            self.handle_event(&mut event);
        }

        // Idle processing - broadcast command set changes
        self.idle();

        // Remove closed windows
        let had_closed_windows = self.desktop.remove_closed_windows();

        // Handle window movement
        let had_moved_windows = self.desktop.handle_moved_windows(&mut self.terminal);

        // Update display as needed
        self.update_active_view_bounds();

        // Optimized drawing - only redraw when needed
        if self.needs_redraw || had_moved_windows || had_event {
            self.draw();
            let _ = self.terminal.flush();
        }
    }
}
```

The event loop continues until `self.running` is set to `false`, which typically happens when:
- The user presses a quit key (F10, Alt+X, Ctrl+C)
- A `CM_QUIT` command is handled
- The application explicitly sets `app.running = false`

### Event Handling Chain

Events are dispatched through a hierarchical chain (see `src/app/application.rs:219-259`):

1. **Menu Bar** — Gets first opportunity to handle the event
2. **Desktop** — Dispatches to windows and their child views
3. **Status Line** — Handles status line interactions
4. **Application** — Handles application-level commands like `CM_QUIT`

Each component can "consume" an event by clearing it (`event.clear()`), preventing further processing.

### The Draw Method

The `draw()` method (defined at `src/app/application.rs:202-217`) renders all visible components:

```rust
pub fn draw(&mut self) {
    // Draw desktop first (windows and their contents)
    self.desktop.draw(&mut self.terminal);

    // Draw menu bar on top (so dropdown menus appear over windows)
    if let Some(ref mut menu_bar) = self.menu_bar {
        menu_bar.draw(&mut self.terminal);
    }

    // Draw status line
    if let Some(ref mut status_line) = self.status_line {
        status_line.draw(&mut self.terminal);
    }

    // Update cursor to focused control
    self.desktop.update_cursor(&mut self.terminal);
}
```

The drawing order ensures that menus appear above windows and the cursor is positioned correctly.

---

## Modal View Execution

In addition to the main event loop, applications can execute views **modally** using `exec_view()`. Modal execution is used for dialogs that need to block and wait for user input before continuing.

### The exec_view Method

```rust
pub fn exec_view(&mut self, view: Box<dyn View>) -> CommandId
```

This method (defined at `src/app/application.rs:76-125`):
1. Adds the view to the desktop
2. Checks if the view has the `SF_MODAL` flag
3. If modal, runs a local event loop until the view closes
4. Returns the view's `end_state` (the command that closed it)

### Example: Executing a Modal Dialog

```rust
use turbo_vision::views::dialog::DialogBuilder;
use turbo_vision::views::button::ButtonBuilder;
use turbo_vision::core::command::{CM_OK, CM_CANCEL};

// Create a modal dialog using the builder pattern
let mut dialog = DialogBuilder::new()
    .bounds(Rect::new(20, 8, 60, 16))
    .title("Confirm Action")
    .modal(true)
    .build();

// Add buttons
dialog.add(ButtonBuilder::new()
    .bounds(Rect::new(10, 5, 20, 7))
    .title("  OK  ")
    .command(CM_OK)
    .default(true)
    .build_boxed()
);

dialog.add(ButtonBuilder::new()
    .bounds(Rect::new(22, 5, 32, 7))
    .title("Cancel")
    .command(CM_CANCEL)
    .build_boxed()
);

// Execute modally and get result
let result = app.exec_view(Box::new(dialog));

match result {
    CM_OK => println!("User clicked OK"),
    CM_CANCEL => println!("User cancelled"),
    _ => {}
}
```

The `exec_view()` method blocks until the dialog is closed, making it easy to get user decisions synchronously.

### Modal vs. Modeless Views

- **Modal views** (with `SF_MODAL` flag) block in `exec_view()` and return a result
- **Modeless views** are added to the desktop and `exec_view()` returns immediately with 0

You can also use the `execute()` method on views directly, which internally calls `app.exec_view()`.

---

## Idle Processing and Command Set Management

The application performs important background tasks during idle time through the `idle()` method.

### The Idle Method

The `idle()` method (defined at `src/app/application.rs:284-303`) is called during every iteration of the event loop:

```rust
pub fn idle(&mut self) {
    // Check if command set changed and broadcast to all views
    if command_set::command_set_changed() {
        let mut event = Event::broadcast(CM_COMMAND_SET_CHANGED);

        // Broadcast to desktop (propagates to all children)
        self.desktop.handle_event(&mut event);

        // Also send to menu bar and status line
        if let Some(ref mut menu_bar) = self.menu_bar {
            menu_bar.handle_event(&mut event);
        }
        if let Some(ref mut status_line) = self.status_line {
            status_line.handle_event(&mut event);
        }

        command_set::clear_command_set_changed();
    }
}
```

### Command Set Changes

The command set is a global registry of enabled/disabled commands. When commands are enabled or disabled (for example, when a document becomes active or inactive), the command set changes, and all views need to update their appearance.

**Enabling and disabling commands:**

```rust
// Enable a command
app.enable_command(CM_SAVE);

// Disable a command
app.disable_command(CM_SAVE);

// Check if a command is enabled
if app.command_enabled(CM_SAVE) {
    // Command is available
}
```

When you enable or disable commands, the `idle()` method detects the change and broadcasts `CM_COMMAND_SET_CHANGED` to all views. Views respond by updating their display (for example, graying out disabled menu items).

This mechanism ensures that the user interface stays synchronized with the application state without requiring manual updates.

---

## Context-sensitive Help

Turbo Vision has built-in tools that help you implement context-sensitive help within your application. You can assign a help context number to a view, and Turbo Vision ensures that whenever that view becomes focused, its help context number becomes the application's current help context number.

### Help Context IDs

Help contexts are represented as `u16` values (`HelpContextId` type). The special constant `HC_NO_CONTEXT` (value 0) represents no context and doesn't change the current help context.

### Setting Up Help

To create context-sensitive help:

1. **Create a help file** — Typically a markdown file containing help topics
2. **Register help contexts** — Map context IDs to topic IDs
3. **Assign contexts to views** — Each view can have a help context
4. **Show help on demand** — Display help when F1 is pressed

### Example: Help System

```rust
use turbo_vision::views::help_file::HelpFile;
use turbo_vision::views::help_window::HelpWindow;
use turbo_vision::views::help_context::{HelpContext, HC_NO_CONTEXT};
use std::rc::Rc;
use std::cell::RefCell;

// Load help file
let help_file = Rc::new(RefCell::new(
    HelpFile::new("help.md").expect("Failed to load help file")
));

// Create help context manager
let mut help_ctx = HelpContext::new();
help_ctx.register(100, "welcome");
help_ctx.register(101, "file-menu");
help_ctx.register(102, "edit-menu");

// Show help for a specific context
fn show_help(app: &mut Application, help_file: Rc<RefCell<HelpFile>>, topic_id: &str) {
    let mut help_window = HelpWindow::new(
        Rect::new(10, 3, 70, 22),
        "Help",
        help_file
    );

    if help_window.show_topic(topic_id) {
        help_window.execute(app);
    }
}

// Show help for context 100
if let Some(topic) = help_ctx.get_topic(100) {
    show_help(&mut app, help_file.clone(), topic);
}
```

### Help in Menus and Status Lines

Help contexts are also used by menus and status lines:

- **Menu items** have a `help_ctx` field that identifies their help topic
- **Status line items** can change based on the current help context
- When a view receives focus, its help context becomes active

This allows the status line to display different keyboard shortcuts based on what view is currently focused, providing context-appropriate guidance to users.

### Implementing Help Views

A complete help system typically includes:

1. **HelpFile** — Parses and manages markdown help files (see `src/views/help_file.rs`)
2. **HelpViewer** — Displays help content with scrolling (see `src/views/help_viewer.rs`)
3. **HelpWindow** — Window wrapper for the help viewer (see `src/views/help_window.rs`)
4. **HelpContext** — Maps context IDs to topics (see `src/views/help_context.rs`)

A working example is provided in `examples/help_system.rs`.

Context-sensitive help is probably one of the last features you'll implement in your application. Turbo Vision views are initialized with a default context of `HC_NO_CONTEXT`, which doesn't change the current context. When the time comes, you can work out a system of help numbers, then assign the appropriate number to each view.

---

## Application Shutdown

The application automatically shuts down when the event loop exits. The `Application` struct implements the `Drop` trait to ensure proper cleanup:

```rust
impl Drop for Application {
    fn drop(&mut self) {
        let _ = self.terminal.shutdown();
    }
}
```

The terminal shutdown:
- Restores the terminal to its original mode
- Re-enables line buffering and echo
- Shows the cursor
- Clears any terminal state

You can also manually quit the application by:

```rust
// Set the running flag to false
app.running = false;

// Or generate a quit command
let mut event = Event::command(CM_QUIT);
app.handle_event(&mut event);
```

---

## Advanced Application Methods

The `Application` struct provides several advanced methods for specialized use cases:

### get_event Method

For custom event loops or modal processing, you can use `get_event()` instead of `run()`:

```rust
pub fn get_event(&mut self) -> Option<Event>
```

This method (defined at `src/app/application.rs:53-67`):
1. Performs idle processing
2. Updates active view bounds
3. Draws the screen
4. Polls for an event
5. Returns the event

Modal views use `get_event()` internally to implement their local event loops.

### Direct Event Handling

You can handle events directly without entering the main loop:

```rust
loop {
    if let Ok(Some(mut event)) = app.terminal.poll_event(Duration::from_millis(50)) {
        app.handle_event(&mut event);

        if event.what == EventType::Nothing {
            // Event was handled
        }
    }

    if !app.running {
        break;
    }
}
```

This gives you fine-grained control over the event loop for specialized applications.

---

## Summary

The `Application` struct is the backbone of Turbo Vision applications:

- **Creation** — `Application::new()` initializes the terminal and desktop
- **Configuration** — Add menu bar, status line, and initial windows
- **Event Loop** — `run()` manages the main application loop
- **Modal Execution** — `exec_view()` runs dialogs modally
- **Idle Processing** — Broadcasts command set changes to keep UI synchronized
- **Help System** — Context-sensitive help maps help IDs to topics
- **Shutdown** — Automatic cleanup when the application exits

Understanding the application object's lifecycle and methods enables you to build sophisticated, responsive Turbo Vision applications that handle events efficiently and provide a polished user experience.

In the next chapters, we'll explore windows, dialogs, and the various control types that make up complete user interfaces.
