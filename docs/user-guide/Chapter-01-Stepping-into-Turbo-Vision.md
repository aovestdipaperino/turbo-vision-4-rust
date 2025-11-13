# Chapter 1 — Stepping into Turbo Vision

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

Examples demonstrating these concepts can be found in the `examples/` directory of the Turbo Vision Rust distribution. The examples are named descriptively (e.g., `menu.rs`, `dialog_example.rs`, `validator_demo.rs`) and demonstrate various stages of application development.

---

## What's in a Turbo Vision Application?

Before you start building your first Turbo Vision application, let's take a look at what Turbo Vision gives you to build your applications.

A Turbo Vision application is a cooperating society of **views**, **events**, and **engines**.

### Views

A view is any program element that is visible on the screen—and all such elements are objects.
In a Turbo Vision context, if you can see it, it's a view. Fields, field captions, window borders, scroll bars, menu bars, and push buttons are all views. Views can be combined to form more complex elements like windows and dialog boxes. These collective views are called **groups**, and they operate together as though they were a single view. Conceptually, groups may be considered views.

Views are always rectangular. This includes rectangles that contain a single character, or lines which are only one character high or one character wide.

### Events

An event is some sort of occurrence to which your application must respond.
Events come from the keyboard, from the mouse, or from other parts of Turbo Vision. For example, a keystroke is an event, as is a click of a mouse button. Events are queued by Turbo Vision's application skeleton as they occur, then processed in order by an event handler.

The `Application` struct, which is the body of your application, contains an event handler.
Events not serviced by `Application` are passed along to other views owned by the program until either a view is found to handle the event or the event is consumed.

For example, an **F1 keystroke** invokes the help system. Unless each view has its own help entry, the F1 keystroke is handled by the main program's event handler. Alphanumeric keys or line-editing keys, by contrast, need to be handled by the view that currently has the focus.

### Engines

Engines, sometimes called "mute objects," are any other objects in the program that are not views. They don't speak to the screen themselves. They perform calculations, communicate with peripherals, and generally do the work of the application. When an engine needs to display some output to the screen, it must do so through a cooperating view.

This concept is important for keeping order in a Turbo Vision application: **Only views may access the display.**
Writing directly to the display with Rust's `println!` or `print!` statements would disrupt Turbo Vision's output and cause visual corruption.

---

## Step 1 — Creating an Application

The usual way to get started with a new library is to write the simplest program possible, such as a very short one that displays "Hello, World!".
In this step, you'll:

- Create the absolute minimum Turbo Vision program
- Extend the basic application

The application object provides the framework on which you'll build a real application. The simplest Turbo Vision program is just an instance of the base `Application` struct.

### Listing 1.1 — The Simplest Turbo Vision Program

```rust
use turbo_vision::app::Application;

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;
    app.run();
    Ok(())
}
```

If you run this program, you'll see a screen with a blank desktop and basic functionality. The application will respond to `Alt+X`, `Ctrl+C`, or `F10` to exit cleanly.

The `Application::new()` method initializes the terminal and sets up the desktop. The `run()` method starts the event loop, which continues until the application receives a quit command. When the `app` variable goes out of scope, the `Drop` trait implementation automatically cleans up and restores the terminal.

### Extending the Application

The minimal program shows the default behavior of the `Application` struct. To add new abilities to your application, you can create a custom struct that wraps `Application` and provides additional functionality.

### Listing 1.2 — An Extensible Application

```rust
use turbo_vision::app::Application;

struct TutorApp {
    app: Application,
}

impl TutorApp {
    fn new() -> std::io::Result<Self> {
        Ok(Self {
            app: Application::new()?,
        })
    }

    fn run(&mut self) {
        self.app.run();
    }
}

fn main() -> std::io::Result<()> {
    let mut tutor_app = TutorApp::new()?;
    tutor_app.run();
    Ok(())
}
```

This behaves exactly like the minimal program, since `TutorApp` currently just wraps `Application` without modification. Later, you'll extend it with new fields and methods to customize behavior.

### Creating a Command Module

Turbo Vision commands are `u16` constants (type alias `CommandId`). The Turbo Vision library provides standard commands in the `turbo_vision::core::command` module:

```rust
pub const CM_QUIT: CommandId = 24;
pub const CM_OK: CommandId = 10;
pub const CM_CANCEL: CommandId = 11;
pub const CM_NEW: CommandId = 102;
pub const CM_OPEN: CommandId = 103;
pub const CM_SAVE: CommandId = 104;
// ... and more
```

For your own application-specific commands, you can define them as constants in your main file or in a separate module:

### Listing 1.3 — Custom Command Constants

```rust
// In a separate module file: src/commands.rs
use turbo_vision::core::command::CommandId;

pub const CMD_ORDER_NEW: CommandId = 251;
pub const CMD_ORDER_WIN: CommandId = 252;
pub const CMD_ORDER_SAVE: CommandId = 253;
pub const CMD_ORDER_CANCEL: CommandId = 254;
pub const CMD_ORDER_NEXT: CommandId = 255;
pub const CMD_ORDER_PREV: CommandId = 250;
pub const CMD_CLIP_SHOW: CommandId = 260;
pub const CMD_ABOUT: CommandId = 270;
pub const CMD_FIND_ORDER_WINDOW: CommandId = 2000;

pub const CMD_OPTIONS_VIDEO: CommandId = 1502;
pub const CMD_OPTIONS_SAVE: CommandId = 1503;
pub const CMD_OPTIONS_LOAD: CommandId = 1504;
```

You can then import these constants in your main application with:

```rust
mod commands;
use commands::*;
```

Keeping constants in a separate module provides a single, central location for all constants, avoids duplication, and makes your code more maintainable.

---

At this point, your minimal Turbo Vision application runs, displays its framework, and exits cleanly.
In the next step, you'll **customize menus and status lines** to make the interface more useful.

---

**Next:** [Chapter 2 — Responding to Commands](Chapter-2-Responding-to-Commands.md)
