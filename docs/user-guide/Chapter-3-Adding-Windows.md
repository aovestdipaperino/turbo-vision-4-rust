# Chapter 3 — Adding Windows

So far you've customized your application's menu bar and status line and seen how to respond to their commands. In this chapter, you'll start adding windows to the desktop and managing them.

In this chapter, you'll do the following steps:

- Add a simple window
- Understand window management and the desktop
- Work with window properties and behavior
- Learn about modal dialogs

---

## Step 4 — Adding a Window

**Progress:** Step 1: Basic App → Step 2: Menu/Status → Step 3: Commands → **Step 4: Windows** → Step 5: Clipboard → Step 6: Streams → Step 7: Resources → Step 8: Data entry → Step 9: Controls → Step 10: Validating → Step 11: Collections → Step 12: Custom view

One of the great benefits of Turbo Vision is that it makes it easy to create and manage multiple, overlapping, resizable windows.

The key to managing windows is the **desktop**, which knows how to keep track of all the windows you give it and which can handle such operations as cascading, tiling, and cycling through the available windows.

The desktop is one example of a **group** in Turbo Vision; that is, a visible object that holds and manages other visible items. You've already used one group—the application itself, which handles the menu bar, the status line, and the desktop. As you proceed, you'll find that windows and dialog boxes are also groups.

---

## Adding a Simple Window

The desktop object is created automatically when you initialize the `Application` and is accessible via the `app.desktop` field. By default, the desktop covers all of the application screen that isn't covered by the menu bar and status line.

Adding a window to the desktop in an application takes three steps:

1. Defining the boundaries for the window
2. Constructing the window object
3. Inserting the window into the desktop

As a first step, you can add a plain window to the desktop in response to the New item on the File menu. That item generates a `CM_NEW` command, so you need to define a response to that command in your application's event loop.

### Listing 3.1 — Adding a Simple Window

```rust
use turbo_vision::prelude::*;
use turbo_vision::app::Application;
use turbo_vision::views::window::WindowBuilder;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::command::{CM_NEW, CM_QUIT};
use turbo_vision::core::event::{Event, EventType};
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;

    // Set up menu bar and status line (as in Chapter 2)
    // ... menu bar setup code ...
    // ... status line setup code ...

    loop {
        app.draw();
        app.terminal.flush()?;

        if let Ok(Some(mut event)) = app.terminal.poll_event(Duration::from_millis(50)) {
            app.handle_event(&mut event);

            if !app.running {
                break;
            }

            // Handle custom commands
            if event.what == EventType::Command {
                match event.command {
                    CM_NEW => {
                        new_window(&mut app);
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

fn new_window(app: &mut Application) {
    // Define boundaries for the window
    let bounds = Rect::new(0, 0, 60, 20);

    // Construct the window using the builder pattern
    let window = WindowBuilder::new()
        .bounds(bounds)
        .title("A Window")
        .build_boxed();

    // Insert window into desktop
    app.desktop.add(window);
}
```

### Understanding the Window Creation Process

Let's break down what happens in the `new_window` function:

#### 1. Assigning the Window Boundaries

```rust
let bounds = Rect::new(0, 0, 60, 20);
```

You've seen `Rect` variables before. However, for the menu bar and status line, you set their sizes based on the size of the application (using the terminal's size). In `new_window`, you assign the new window an absolute set of coordinates using `Rect::new`.

The `Rect::new(x1, y1, x2, y2)` function creates a rectangle from coordinates `(x1, y1)` to `(x2, y2)`. This window will be 60 columns wide and 20 rows tall, positioned at the top-left of the desktop.

#### 2. Constructing the Window Object

```rust
let window = WindowBuilder::new()
    .bounds(bounds)
    .title("A Window")
    .build_boxed();
```

The window builder pattern provides a fluent, type-safe API for constructing windows. You start with `WindowBuilder::new()`, then chain methods to set properties:
- `.bounds(bounds)` sets the window's screen position and size
- `.title("A Window")` sets the window title
- `.build_boxed()` creates the window and wraps it in a `Box` for adding to the desktop

The builder pattern is preferred over direct constructors because it:
- Makes optional parameters explicit
- Allows for future extensibility without breaking existing code
- Provides clear, self-documenting code

Unlike the original Turbo Vision, the Rust implementation doesn't support window numbers (the `wnNoNumber` parameter from the Pascal version is not needed). Window selection is handled entirely through mouse clicks and keyboard navigation.

#### 3. Inserting the Window

```rust
app.desktop.add(window);
```

`add` is a method common to all Turbo Vision groups, and it's the way a group gets control of the objects within it. When you add the window to the desktop, you're telling the desktop that it is supposed to manage that window.

The window is already boxed (returned by `.build_boxed()`) because the desktop stores views as trait objects (`Box<dyn View>`), allowing it to manage different types of views polymorphically.

If you run the program now and choose New from the File menu, an empty window with the title 'A Window' appears on the desktop. If you choose New again, another identical window appears in the same place, because `new_window` assigns exact coordinates for the window. Using your mouse, you can select different windows.

The menu items under Window (like Tile, Cascade, Next) and the hot keys bound in the status line now operate on the windows. Note that the menu and status line items haven't changed. They don't know anything about your windows. They just issue commands which the windows and desktop already know how to respond to.

---

## Window Properties and Behavior

Windows in Turbo Vision have several built-in capabilities that work automatically:

### Dragging Windows

Windows can be dragged by clicking and holding the title bar. The window follows the mouse cursor until you release the button. The Rust implementation handles this through the `Frame` component, which detects title bar clicks and enters drag mode.

### Resizing Windows

Windows can be resized by clicking and dragging the bottom-right corner. The implementation enforces a minimum window size (typically 16 columns wide by 6 rows tall) to ensure windows remain usable.

### Z-Order Management

Z-order is the layering of visible objects, determining which windows show up in front of others. When you click on a window that's partially covered by another window, Turbo Vision automatically brings it to the front.

The desktop manages z-order by maintaining windows in a list, where later windows (those added more recently or brought to front) are drawn on top of earlier ones. You can see this in action in `src/views/desktop.rs:226-247`.

### Shadows

Windows display shadows by default, giving a three-dimensional appearance to the interface. Shadows are drawn one cell to the right and below the window boundary, using a darkened attribute (see `src/views/view.rs` for the `draw_shadow` function).

---

## Working with Modal Windows

Modal windows are windows that capture all user input until they are closed. While a modal window is active, you cannot interact with other windows or menu items (except the status line, which remains available).

### Creating Modal Dialogs

The `Dialog` type (which wraps a `Window`) is typically used for modal interactions. Here's an example:

### Listing 3.2 — Creating a Modal Dialog

```rust
use turbo_vision::views::dialog::DialogBuilder;
use turbo_vision::views::button::ButtonBuilder;
use turbo_vision::views::static_text::StaticTextBuilder;
use turbo_vision::core::command::{CM_OK, CM_CANCEL};

fn show_dialog(app: &mut Application) {
    // Create a dialog (40 wide x 10 tall, positioned at 20,8) using the builder pattern
    let mut dialog = DialogBuilder::new()
        .bounds(Rect::new(20, 8, 60, 18))
        .title("Sample Dialog")
        .modal(true)
        .build();

    // Add a message to the dialog
    let text = StaticTextBuilder::new()
        .bounds(Rect::new(2, 1, 36, 3))
        .text("This is a modal dialog box.\nClick OK to continue.")
        .build_boxed();
    dialog.add(text);

    // Add OK button
    let ok_button = ButtonBuilder::new()
        .bounds(Rect::new(10, 5, 20, 7))
        .title("  ~O~K  ")
        .command(CM_OK)
        .default(true)  // This is the default button
        .build_boxed();
    dialog.add(ok_button);

    // Add Cancel button
    let cancel_button = ButtonBuilder::new()
        .bounds(Rect::new(22, 5, 32, 7))
        .title("~C~ancel")
        .command(CM_CANCEL)
        .build_boxed();
    dialog.add(cancel_button);

    // Set initial focus
    dialog.set_initial_focus();

    // Execute the dialog (runs modal event loop)
    let result = dialog.execute(app);

    // Check result
    match result {
        CM_OK => {
            // User clicked OK
        }
        CM_CANCEL => {
            // User clicked Cancel or pressed ESC
        }
        _ => {}
    }
}
```

### Understanding Modal Execution

The `execute` method is key to modal behavior. It:

1. Adds the dialog to the desktop
2. Runs a modal event loop that only processes events for this dialog
3. Blocks all interaction with other windows
4. Returns when the dialog is closed (typically by clicking OK, Cancel, or the close button)

The return value is a `CommandId` indicating which command closed the dialog. This is usually `CM_OK`, `CM_CANCEL`, or another application-specific command.

You can see the modal execution implementation in `src/views/group.rs:200-217` and how the application handles modal views in `src/app/application.rs:76-124`.

---

## Understanding Groups

Windows and dialogs are both **groups** — views that can contain other views. The containment hierarchy looks like this:

```
Application
├── MenuBar
├── StatusLine
└── Desktop (Group)
    ├── Window 1 (Group)
    │   ├── Button
    │   ├── Editor
    │   └── StaticText
    └── Window 2 (Group)
        └── TextViewer
```

Each group is responsible for:

- **Layout management**: Positioning child views
- **Event routing**: Passing events to children
- **Focus management**: Tracking which child has input focus
- **Drawing**: Rendering all children in the correct order

When you add a view to a group using `group.add(Box::new(view))`, the group takes ownership of that view and includes it in its event handling and drawing cycles.

---

## Coordinate Systems

Turbo Vision uses **absolute coordinates** for most operations:

- All `Rect` boundaries are in absolute screen coordinates
- When a view is added to a group, its coordinates are in absolute terms
- Child views' positions are relative to the screen, not to their parent

For example, if you create a button at `Rect::new(5, 5, 15, 7)` and add it to a window at `Rect::new(10, 10, 50, 30)`, the button will appear at screen position `(5, 5)`, **not** at position `(5, 5)` relative to the window's top-left corner.

However, for **dialogs and windows**, there's a convenience: when you add child views to a `Dialog` or `Window`, you typically use **relative coordinates** (relative to the window's interior), and the window translates them to absolute coordinates. This is handled by the interior `Group` within the window.

See the `Window` implementation in `src/views/window.rs:28-50` for details on how the interior group is positioned.

---

## Window Architecture Details

The `Window` struct in the Rust implementation is composed of several parts:

```rust
pub struct Window {
    bounds: Rect,           // Window's screen position
    frame: Frame,           // Title bar, borders, close button
    interior: Group,        // Contains child views
    state: StateFlags,      // Window state (modal, dragging, etc.)
    // ... additional fields for dragging/resizing
}
```

The **frame** handles:
- Drawing the window border and title
- Detecting clicks on the title bar (for dragging) and close button
- Drawing the close button

The **interior** handles:
- Managing child views (buttons, editors, etc.)
- Routing events to children
- Focus management within the window

This separation of concerns makes the window implementation clean and maintainable.

---

## Advanced Window Creation Example

Here's a more complete example that creates a window with multiple child views:

### Listing 3.3 — Window with Multiple Views

```rust
use turbo_vision::views::window::WindowBuilder;
use turbo_vision::views::button::ButtonBuilder;
use turbo_vision::views::static_text::StaticTextBuilder;
use turbo_vision::views::text_viewer::TextViewerBuilder;

fn create_custom_window(app: &mut Application) {
    // Create window using builder pattern
    let mut window = WindowBuilder::new()
        .bounds(Rect::new(10, 5, 70, 20))
        .title("Custom Window")
        .build();

    // Add a title label
    let label = StaticTextBuilder::new()
        .bounds(Rect::new(2, 1, 40, 2))
        .text("Welcome to Turbo Vision!")
        .build_boxed();
    window.add(label);

    // Add a text viewer
    let text_content = "This is a text viewer.\nYou can scroll through content here.";
    let mut viewer = TextViewerBuilder::new()
        .bounds(Rect::new(2, 3, 56, 10))
        .build();
    viewer.load_text(text_content);
    window.add(Box::new(viewer));

    // Add a button
    let button = ButtonBuilder::new()
        .bounds(Rect::new(20, 12, 40, 14))
        .title("  ~C~lose  ")
        .command(CM_CLOSE)
        .default(true)
        .build_boxed();
    window.add(button);

    // Set initial focus to the first focusable view
    window.set_initial_focus();

    // Add to desktop
    app.desktop.add(Box::new(window));
}
```

---

## Event Flow in Windows

Understanding how events flow through windows is crucial:

1. **Application receives event** (keyboard, mouse, or command)
2. **Menu bar gets first shot** at the event
3. **Desktop handles the event**, which:
   - Checks for window clicks (z-order changes)
   - Routes event to the topmost window
4. **Window processes the event**:
   - Frame checks for title bar/close button clicks
   - Interior group routes to focused child
5. **Child view handles the event** (button click, text input, etc.)
6. **Status line gets remaining events**
7. **Application handles application-level commands** (like `CM_QUIT`)

If at any point a handler "consumes" the event by calling `event.clear()`, the event stops propagating. This is how Turbo Vision ensures that each event is handled by exactly one component.

The event handling chain is implemented in `src/app/application.rs:219-258`.

---

## Practical Tips

### Window Positioning

Instead of hardcoding window positions, you can calculate positions dynamically:

```rust
let (width, height) = app.terminal.size();

// Center a 60x20 window
let x = (width as i16 - 60) / 2;
let y = (height as i16 - 20) / 2;
let bounds = Rect::new(x, y, x + 60, y + 20);

let window = WindowBuilder::new()
    .bounds(bounds)
    .title("Centered Window")
    .build_boxed();
app.desktop.add(window);
```

### Cascading Windows

To create a cascading effect (each window offset slightly from the previous), track how many windows have been created:

```rust
fn new_cascaded_window(app: &mut Application, window_count: usize) {
    let offset = (window_count * 2) as i16;
    let bounds = Rect::new(
        5 + offset,
        3 + offset,
        65 + offset,
        23 + offset
    );

    let window = WindowBuilder::new()
        .bounds(bounds)
        .title(&format!("Window {}", window_count + 1))
        .build_boxed();
    app.desktop.add(window);
}
```

### Checking for Window Closure

When handling `CM_CLOSE` commands, you might want to validate whether the window can be closed (for example, prompting to save unsaved changes):

```rust
if event.what == EventType::Command && event.command == CM_CLOSE {
    // Check if there are unsaved changes
    if has_unsaved_changes() {
        let result = message_box(
            app,
            "Save changes before closing?",
            MF_WARNING | MF_YES_BUTTON | MF_NO_BUTTON | MF_CANCEL_BUTTON
        );

        match result {
            CM_YES => {
                save_changes();
                // Allow close
            }
            CM_NO => {
                // Allow close without saving
            }
            CM_CANCEL => {
                // Cancel the close
                event.clear();
                return;
            }
            _ => {}
        }
    }
}
```

---

## Summary

In this chapter, you learned:

- How to create and add windows to the desktop
- The structure of windows (frame and interior group)
- How windows handle dragging, resizing, and z-order
- The difference between modeless and modal windows
- How events flow through the window hierarchy
- Coordinate systems in Turbo Vision
- Practical techniques for window positioning and management

### Key Types and Functions

| Type/Function | Purpose | Module |
|--------------|---------|---------|
| `Window` | Resizable, draggable window with frame | `views::window` |
| `Dialog` | Modal dialog window | `views::dialog` |
| `Desktop` | Container for windows | `views::desktop` |
| `Group` | Container for child views | `views::group` |
| `Frame` | Window border and title bar | `views::frame` |
| `window.add()` | Add a child view to a window | `views::window` |
| `dialog.execute()` | Run a modal dialog | `views::dialog` |
| `desktop.add()` | Add a window to the desktop | `views::desktop` |
| `set_initial_focus()` | Set focus to first focusable child | `views::window` |

### Next Steps

In the next chapter, you'll learn about **persistence and configuration**, including how to save and load the desktop state, work with streams, and use resources to bundle application data.

---

## Architecture Notes: From Pascal to Rust

The Rust implementation maintains the same conceptual model as Borland's Turbo Vision, but adapts it to Rust's ownership system:

### Window Construction

**Original Pascal:**
```pascal
TheWindow := New(PWindow, Init(R, 'A window', wnNoNumber));
Desktop^.Insert(TheWindow);
```

**Rust Implementation:**
```rust
let window = WindowBuilder::new()
    .bounds(bounds)
    .title("A window")
    .build_boxed();
app.desktop.add(window);
```

**Key differences:**
- Rust uses the builder pattern for type-safe, fluent construction
- No separate `Init` method — builder does all configuration
- No window numbers (simplified interaction model)
- Rust ownership prevents memory leaks automatically
- Builder pattern allows for future extensibility without breaking changes

### Group Management

**Original Pascal:** Groups used doubly-linked lists of pointers with manual memory management

**Rust Implementation:** Groups use `Vec<Box<dyn View>>` with automatic memory management
- Type-safe (trait objects replace void pointers)
- Automatic cleanup when group is dropped
- Same Z-order semantics (last in vector = topmost)

### Modal Execution

**Original Pascal:** Used `execView` with a local event loop and `endModal` to set end state

**Rust Implementation:** Same pattern, adapted to Rust
- `execute` method runs modal event loop (`src/views/group.rs:200-217`)
- `end_modal` sets the end state
- Automatic cleanup when dialog closes
- Same conceptual model, safer implementation

---

## See Also

- **Chapter 2**: Responding to Commands (event handling basics)
- **Chapter 8**: Views and Groups (detailed group architecture)
- **Chapter 9**: Event-Driven Programming (advanced event handling)
- **Chapter 11**: Window and Dialog Box Objects (advanced window features)
- **Example Code**: `examples/window_resize_demo.rs` — Window dragging and resizing
- **Example Code**: `examples/dialog_example.rs` — Modal dialog with buttons
- **Example Code**: `examples/window_modal_overlap_test.rs` — Modal window behavior
- **Source Code**: `src/views/window.rs` — Window implementation
- **Source Code**: `src/views/desktop.rs` — Desktop management
- **Source Code**: `src/views/frame.rs` — Window frame implementation
