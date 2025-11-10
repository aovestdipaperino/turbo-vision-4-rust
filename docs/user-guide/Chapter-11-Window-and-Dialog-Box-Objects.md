# Chapter 11: Window and Dialog Box Objects

This chapter describes window and dialog box objects in Turbo Vision, covering their construction, management, and usage patterns.

## Window Objects

Window objects provide the primary container for user interface elements in Turbo Vision. A window consists of a frame (title bar and border) and an interior group where child views can be inserted.

### Window Construction and Frame

The `Window` struct is defined in `src/views/window.rs`. When you create a window, it automatically constructs and manages a `Frame` object that provides the title bar, borders, and window controls.

```rust
// Create a window with title and bounds using the builder pattern
let mut window = WindowBuilder::new()
    .bounds(Rect::new(5, 5, 50, 20))  // Position and size
    .title("My Window")                // Title
    .build();
```

The window's frame is automatically created and inserted as the first child. The frame handles:
- Drawing the double-line border (╔═╗║╚╝)
- Rendering the title text in the title bar
- Drawing the close button [■]
- Detecting drag operations on the title bar
- Detecting resize operations on the bottom-right corner

See `src/views/frame.rs` for the Frame implementation.

### Using Window Numbers

In the original Turbo Vision, windows could have numbers (1-9) displayed on their frames, allowing selection via Alt-1 through Alt-9 keystrokes. The current Rust implementation does not include window number support, but windows are managed by their index position in the Desktop's child list, which can be used for similar purposes.

Window identification is handled through z-order position in the Desktop:
- Windows later in the Desktop's children vector are "in front"
- The `focused` field in the Desktop's Group tracks which window has focus
- Windows can be brought to front using the `bring_to_front()` method

### Managing Window Size

Windows support minimum and maximum size constraints. The minimum window size is enforced during resize operations.

#### Limiting Window Size

The Window implementation enforces a minimum size of 16 columns wide and 6 lines tall (see `src/views/window.rs:143-147`). This ensures that the frame elements (close button, title, and resize corner) remain visible and functional.

```rust
// Minimum size check during resize
if self.bounds.width() >= 16 && self.bounds.height() >= 6 {
    self.bounds = new_bounds;
    self.update_frame_and_interior();
}
```

You can customize size limits by checking bounds before applying changes in your window's event handler.

#### Zooming Windows

The current Rust implementation does not include automatic zoom functionality (maximizing a window to fill the desktop). However, you can implement this behavior by:

1. Storing the window's original bounds before maximizing
2. Setting the window bounds to match the Desktop bounds
3. Restoring the original bounds when toggling back

The pattern would look like:

```rust
// Store original bounds
let zoom_rect = self.bounds;

// Maximize to desktop size
self.bounds = desktop_bounds;
self.update_frame_and_interior();

// Later: restore from zoom_rect
```

### Creating Window Scroll Bars

Scroll bars can be added to windows for scrolling content. The scroll bar construction pattern involves:

1. Creating the scroll bar with appropriate bounds
2. Adding it to the window's interior group
3. Connecting it to the scrolling content (like a Scroller view)

```rust
// Create vertical scroll bar on the right side of window interior
let interior_bounds = window.interior_group().bounds;
let scroll_bounds = Rect::new(
    interior_bounds.b.x - 1,
    interior_bounds.a.y,
    interior_bounds.b.x,
    interior_bounds.b.y
);

let vscroll = ScrollBar::new(scroll_bounds);
window.add(Box::new(vscroll));

// Create horizontal scroll bar at the bottom
let hscroll_bounds = Rect::new(
    interior_bounds.a.x,
    interior_bounds.b.y - 1,
    interior_bounds.b.x - 1,
    interior_bounds.b.y
);

let hscroll = ScrollBar::new(hscroll_bounds);
window.add(Box::new(hscroll));
```

Scroll bars respond to mouse clicks on arrows, page areas, and thumb dragging. See `src/views/scrollbar.rs` for the ScrollBar implementation.

### Window Dragging and Resizing

Windows support interactive dragging and resizing through the Frame:

**Dragging**: Click and hold on the title bar area (anywhere except the close button) to drag the window. The Frame detects this in `handle_event()` and sets the `SF_DRAGGING` state flag, recording a `drag_offset` in the Window.

**Resizing**: Click and hold on the bottom-right corner character (╝) to resize the window. The Frame sets the `SF_RESIZING` state flag, and the Window records the `resize_start_size`.

Movement tracking is handled through:
- `prev_bounds`: Stores the previous position for redraw calculations
- `get_redraw_union()`: Returns the union of current and previous bounds
- Desktop's `handle_moved_windows()`: Detects movement and redraws affected areas

See `src/views/window.rs:197-235` for drag handling and `src/views/window.rs:237-269` for resize handling.

### Window Examples

Several examples demonstrate window usage:

- **examples/window_resize_demo.rs**: Shows multiple windows with dragging and resizing
- **examples/window_modal_overlap_test.rs**: Demonstrates overlapping windows and z-order management
- **examples/dialog_example.rs**: Shows simple window usage with buttons

## Working with Dialog Boxes

Dialog boxes are specialized windows designed for modal operation and user input collection. The `Dialog` struct (in `src/views/dialog.rs`) wraps a Window and provides modal execution capabilities.

### Dialog Box Construction

Dialog boxes are simpler to construct than regular windows, as they default to fixed size and no window number:

```rust
// Create a modal dialog using the builder pattern
let mut dialog = DialogBuilder::new()
    .bounds(Rect::new(20, 8, 60, 18))  // Position and size
    .title("Settings")                  // Title
    .modal(true)                        // Make it modal
    .build();

// Add buttons
dialog.add(ButtonBuilder::new()
    .bounds(Rect::new(5, 8, 15, 10))
    .title("OK")
    .command(CommandId::CM_OK)
    .default(true)  // is_default
    .build_boxed()
);

dialog.add(ButtonBuilder::new()
    .bounds(Rect::new(20, 8, 30, 10))
    .title("Cancel")
    .command(CommandId::CM_CANCEL)
    .build_boxed()
);
```

### Dialog Box Default Attributes

Dialog boxes differ from regular windows in the following ways:

- **Fixed size**: Not resizable by default (no resize corner handling)
- **Modal focus**: Designed for modal operation via `execute()`
- **No window number**: Not part of Alt-1 through Alt-9 selection
- **Frame palette**: Uses dialog frame coloring (gray scheme)

See `src/views/dialog.rs:20-50` for Dialog construction.

### Modal Dialog Box Behavior

The key feature of dialog boxes is their modal execution through the `execute()` method:

```rust
// Execute dialog modally
match dialog.execute(&mut app) {
    CommandId::CM_OK => {
        println!("User clicked OK");
        // Process dialog data
    }
    CommandId::CM_CANCEL => {
        println!("User cancelled");
    }
    _ => {}
}
```

The `execute()` method:
1. Sets the `SF_MODAL` state flag on the dialog
2. Runs an event loop that processes events
3. Returns when any command is generated that sets `end_state`
4. Returns the CommandId that ended the modal state

See `src/views/dialog.rs:88-136` for the modal execution implementation.

#### Handling Dialog Box Events

Dialog boxes handle events with special modal behaviors:

**Enter key**: When Enter is pressed, the dialog searches for a default button (one with `is_default_button()` returning true) and activates it by broadcasting its command.

**ESC ESC sequence**: Pressing Escape twice generates a `CM_CANCEL` command, providing a keyboard way to cancel the dialog.

**Command-based closure**: Any command event that reaches the dialog ends the modal state. The standard commands are:
- `CM_OK`: User confirmed the dialog
- `CM_CANCEL`: User cancelled the dialog
- `CM_YES`: User answered yes
- `CM_NO`: User answered no

Custom commands can also be used to end modal state.

See `src/views/dialog.rs:138-201` for event handling.

### Using Controls in a Dialog Box

Dialog boxes are designed to contain control views like buttons, input fields, checkboxes, and list boxes.

#### Adding Controls to a Dialog Box

Controls are added during dialog construction using the `add()` method:

```rust
let mut dialog = DialogBuilder::new()
    .bounds(Rect::new(20, 8, 60, 18))
    .title("User Information")
    .modal(true)
    .build();

// Add label
dialog.add(Box::new(Label::new(
    Rect::new(2, 2, 15, 3),
    "Name:".to_string()
)));

// Add input field
dialog.add(Box::new(InputLine::new(
    Rect::new(16, 2, 36, 3),
    30  // max length
)));

// Add button
dialog.add(ButtonBuilder::new()
    .bounds(Rect::new(15, 8, 25, 10))
    .title("OK")
    .command(CommandId::CM_OK)
    .default(true)  // is_default
    .build_boxed()
);
```

Controls are added in the order they should appear in the tab order (focus order).

#### Tab Order and Focus Management

The order in which controls are inserted determines the tab order - the sequence in which controls receive focus when the user presses Tab.

**Focus progression**: Tab moves forward through controls in insertion order. Shift-Tab moves backward.

**Setting initial focus**: Call `set_initial_focus()` on the dialog to focus the first focusable control:

```rust
dialog.set_initial_focus();
```

**Manual focus control**: Use `set_focus_to_child()` to focus a specific child by index:

```rust
dialog.set_focus_to_child(2);  // Focus third control
```

Good practice: Insert controls in a logical order (top-to-bottom, left-to-right) to create an intuitive user experience.

#### Manipulating Controls

The current Rust implementation handles control data through direct access rather than the `SetData`/`GetData` pattern used in the original Turbo Vision. To set or read control values:

1. Store references to controls when creating them
2. Access control properties directly through the reference
3. Update control state by calling control methods

```rust
// Store reference when creating
let input = Rc::new(RefCell::new(InputLine::new(
    Rect::new(16, 2, 36, 3),
    30
)));
dialog.add(Box::new(input.clone()));

// Later: set value
input.borrow_mut().set_text("Default text");

// Later: get value
let text = input.borrow().get_text();
```

For more complex data transfer patterns, you would implement custom data record structures that match your dialog's control layout.

### Dialog Examples

See these examples for dialog usage patterns:

- **examples/dialog_example.rs**: Basic dialog with OK/Cancel buttons (src/views/dialog.rs:3)
- **examples/file_dialog.rs**: File selection dialog example
- **examples/window_resize_demo.rs**: Contains menu and status line demonstrations

## Using Standard Dialog Boxes

Turbo Vision provides specialized dialog boxes for common tasks.

### Using Message Boxes

Message boxes provide a quick way to display information or ask simple questions. The `MsgBox` builder pattern (in `src/views/msgbox.rs`) creates message dialogs:

```rust
// Information message
let msg = MsgBox::new("Operation complete!")
    .message_type(MessageType::Information)
    .button_ok()
    .centered();

let result = msg.execute(&mut app);
```

```rust
// Confirmation dialog
let msg = MsgBox::new("Delete this file?")
    .message_type(MessageType::Warning)
    .button_yes_no()
    .centered();

match msg.execute(&mut app) {
    CommandId::CM_YES => {
        // Delete the file
    }
    CommandId::CM_NO => {
        // Cancel
    }
    _ => {}
}
```

Message box types (from `src/views/msgbox.rs`):
- `MessageType::Information`: General information (╔ ℹ ╗)
- `MessageType::Warning`: Warning message (╔ ⚠ ╗)
- `MessageType::Error`: Error message (╔ ✖ ╗)
- `MessageType::Confirmation`: Confirmation request (╔ ? ╗)

Button configurations:
- `button_ok()`: Single OK button
- `button_ok_cancel()`: OK and Cancel buttons
- `button_yes_no()`: Yes and No buttons
- `button_yes_no_cancel()`: Yes, No, and Cancel buttons

### Using File Dialog Boxes

File dialog boxes allow users to browse and select files. The `FileDialog` struct provides file selection functionality:

```rust
let mut file_dialog = FileDialogBuilder::new()
    .bounds(Rect::new(10, 5, 70, 20))
    .title("Open File")
    .initial_dir(PathBuf::from("/home/user/documents"))  // starting directory
    .wildcard("*")
    .build();

match file_dialog.execute(&mut app) {
    CommandId::CM_OK => {
        if let Some(path) = file_dialog.get_selected_file() {
            println!("Selected: {}", path);
            // Open the file
        }
    }
    CommandId::CM_CANCEL => {
        println!("File selection cancelled");
    }
    _ => {}
}
```

See `examples/file_dialog.rs` for a complete example.

### Change Directory Dialog Boxes

The current Rust implementation does not yet include a specialized change directory dialog. However, you can implement this functionality using a custom dialog with a directory tree view or list box showing the directory hierarchy.

The pattern would involve:
1. Creating a dialog with a list or tree showing directories
2. Allowing navigation through directory levels
3. Returning the selected directory path when the user confirms

## Implementation Notes

### State Management

Window and dialog state is managed through the `StateFlags` enum (defined in `src/core/state.rs`):

- `SF_MODAL`: View is in modal execution
- `SF_ACTIVE`: Window is active (has focus)
- `SF_DRAGGING`: Window is being dragged
- `SF_RESIZING`: Window is being resized
- `SF_SHADOW`: Window should draw a shadow
- `SF_CLOSED`: Window is marked for removal
- `SF_VISIBLE`: View is visible
- `SF_FOCUSED`: View has focus
- `SF_DISABLED`: View is disabled

### Command System

Window operations use the command system (from `src/core/command.rs`):

- `CM_CLOSE`: Close button clicked on frame
- `CM_OK`: OK button pressed
- `CM_CANCEL`: Cancel button pressed or ESC ESC
- `CM_YES`: Yes button pressed
- `CM_NO`: No button pressed
- `CM_DEFAULT`: Enter key pressed (activates default button)

### Desktop Window Management

The `Desktop` struct (in `src/views/desktop.rs`) manages all windows:

- **Adding windows**: `desktop.add(window)` adds and focuses a window
- **Window count**: `desktop.child_count()` returns number of windows (excluding background)
- **Z-order**: Windows later in the children vector are in front
- **Bringing to front**: `desktop.bring_to_front(index)` reorders window to front
- **Movement tracking**: `desktop.handle_moved_windows()` detects and redraws moved windows
- **Cleanup**: `desktop.remove_closed_windows()` removes windows marked SF_CLOSED

### Coordinate Systems

Windows use absolute screen coordinates. When you add a child to a window, the window's interior group automatically converts the child's relative bounds to absolute coordinates.

```rust
// Child bounds are relative to window interior
let button_bounds = Rect::new(5, 2, 15, 4);  // Relative position

// Window's add() method converts to absolute coordinates
window.add(ButtonBuilder::new()
    .bounds(button_bounds)
    .title("OK")
    .command(CM_OK)
    .build_boxed());
```

The conversion happens in `Group::add()` (src/views/group.rs:59-78).

## Summary

Turbo Vision's window and dialog system provides:

- **Window objects**: Resizable, draggable containers with frames and interior groups
- **Dialog boxes**: Modal windows optimized for user interaction with controls
- **Frame decoration**: Automatic title bars, borders, and close buttons
- **Size management**: Minimum size constraints and manual zoom capability
- **Movement tracking**: Automatic redraw of moved windows and union rectangles
- **Control containers**: Built-in support for buttons, inputs, and other controls
- **Modal execution**: Event loop management for dialog-based workflows
- **Standard dialogs**: Message boxes and file dialogs for common tasks

For more information on specific topics:
- Controls: See Chapter 12 (to be written)
- Event handling: See Chapter 9
- Views and Groups: See Chapter 8
- Application structure: See Chapter 10

## File References

Key implementation files:
- `src/views/window.rs`: Window struct and management
- `src/views/dialog.rs`: Dialog struct and modal execution
- `src/views/frame.rs`: Frame decoration and window controls
- `src/views/desktop.rs`: Desktop window manager
- `src/views/group.rs`: Group container and focus management
- `src/views/msgbox.rs`: Message box builder
- `src/core/state.rs`: State flags
- `src/core/command.rs`: Command IDs

Example files:
- `examples/dialog_example.rs`: Basic dialog with buttons
- `examples/window_resize_demo.rs`: Multiple windows with dragging/resizing
- `examples/window_modal_overlap_test.rs`: Overlapping window management
- `examples/file_dialog.rs`: File selection dialog
