# Turbo Vision for Rust - Design Documentation

**Version:** 0.1.0
**Last Updated:** 2025-11-02
**Reference:** Borland Turbo Vision 2.0

---

## Table of Contents

1. [Focus Architecture](#focus-architecture)
2. [FileDialog Implementation](#filedialog-implementation)
3. [Screen Dump System](#screen-dump-system)
4. [Command Set System](#command-set-system)

---

# Focus Architecture

## Overview

The Turbo Vision framework implements a proper focus management system where controls only respond to input when they have focus. This prevents input fields from capturing keys when not focused, list boxes from scrolling when another control is active, and buttons from activating when the user is typing elsewhere.

## Core Principles

### 1. Only Focused Controls Handle Keyboard Input

Controls should only process keyboard events when they have focus. This is enforced at multiple levels:
- Group-level event routing
- Control-level focus checks
- Proper focus state management

### 2. Mouse Events Go to the Control Under the Mouse

Unlike keyboard events, mouse events are sent to the control at the mouse position, regardless of focus state. However, clicking a focusable control automatically gives it focus.

### 3. Tab Key Cycles Through Focusable Controls

The Tab key is handled at the Group level to cycle focus between focusable children. Shift+Tab cycles backward.

## Implementation Details

### Group-Level Event Routing

The `Group` class implements the core focus management logic in its `handle_event` method:

```rust
fn handle_event(&mut self, event: &mut Event) {
    // Tab key cycles focus
    if event.what == EventType::Keyboard && event.key_code == KB_TAB {
        self.select_next();
        event.clear();
        return;
    }

    // Mouse events: send to child under mouse
    if event.what == EventType::MouseDown || ... {
        // Find child at mouse position
        // If clicked on focusable child, give it focus
        // Send event to that child
    }

    // Keyboard events: only send to focused child
    if self.focused < self.children.len() {
        self.children[self.focused].handle_event(event);
    }
}
```

**Key Points:**
- Tab is handled at Group level
- Mouse events find the child at mouse position
- Keyboard events only go to the focused child
- Clicking a focusable control gives it focus

### Control-Level Focus Checks

Each focusable control must check its focus state before handling keyboard input:

```rust
fn handle_event(&mut self, event: &mut Event) {
    if event.what == EventType::Keyboard {
        // Check focus before processing keyboard input
        if !self.focused {
            return;
        }
        // Process keyboard events...
    }

    // Mouse events don't need focus check
    if event.what == EventType::MouseDown {
        // Process click...
    }
}
```

### Controls That Check Focus

The following controls properly check focus before handling keyboard events:
- ✅ InputLine - Text input field
- ✅ ListBox - Scrollable list
- ✅ Button - Push button
- ✅ CheckBox - Checkbox control
- ✅ RadioButton - Radio button control
- ✅ Editor - Text editor
- ✅ Memo - Multi-line text input

### Focus State Management

Controls that can receive focus must:

1. **Implement `can_focus()` to return `true`**
2. **Implement `set_focus()` to track focus state**
3. **Have a `focused: bool` field** in their struct

## Programmatic Focus Control

### Setting Focus to Specific Child

When a dialog needs to set focus to a specific child (e.g., after refreshing contents), use the `set_focus_to_child()` method:

```rust
// FileDialog after directory navigation
self.dialog.set_focus_to_child(CHILD_LISTBOX);
```

This method properly:
1. Clears focus from all children
2. Updates the Group's internal `focused` index
3. Calls `set_focus(true)` on the target child

**⚠️ IMPORTANT:** Do NOT manually call `set_focus()` on individual children without updating the Group's `focused` index:

```rust
// ❌ BAD: Only sets visual focus, Group still thinks another child is focused
self.dialog.child_at_mut(index).set_focus(true);

// ✅ GOOD: Updates both Group state and child focus
self.dialog.set_focus_to_child(index);
```

**Symptoms of improper focus management:**
- Control appears focused (correct colors) but doesn't respond to keyboard
- Need to press Tab before keyboard events work
- Events go to wrong control

This matches Borland's `fileList->select()` pattern which calls `owner->setCurrent(this, normalSelect)` to properly establish focus chain.

## Common Mistakes

❌ **DON'T** pass events to all children and let them decide:
```rust
// BAD: All children get events
for child in &mut self.children {
    child.handle_event(event);
}
```

✅ **DO** only send keyboard events to focused child:
```rust
// GOOD: Only focused child gets keyboard events
if self.focused < self.children.len() {
    self.children[self.focused].handle_event(event);
}
```

❌ **DON'T** forget to check focus in control's handle_event:
```rust
// BAD: Control processes all keyboard input
fn handle_event(&mut self, event: &mut Event) {
    if event.what == EventType::Keyboard {
        // Process keys...
    }
}
```

✅ **DO** check focus before processing keyboard input:
```rust
// GOOD: Only process keys when focused
fn handle_event(&mut self, event: &mut Event) {
    if event.what == EventType::Keyboard {
        if !self.focused {
            return;
        }
        // Process keys...
    }
}
```

## Related Classes

- **Group** (`src/views/group.rs`) - Container with focus management
- **Window** (`src/views/window.rs`) - Wraps Group, delegates focus
- **Dialog** (`src/views/dialog.rs`) - Modal dialog with focus management
- **View trait** (`src/views/view.rs`) - Defines `can_focus()` and `set_focus()`

---

# FileDialog Implementation

## Overview

The FileDialog provides a fully functional file selection interface matching the original Borland Turbo Vision implementation. It supports directory navigation, wildcard filtering, and both mouse and keyboard interaction.

## Features

- Directory listing with wildcard filtering (*.ext patterns)
- **Mouse support**: Click to select files, double-click to open
- **Keyboard navigation**: Arrow keys, PgUp/PgDn, Home/End, Enter
- Directory navigation (click/Enter on `..` for parent, `[dirname]` for subdirectories)
- Visual file browser with ListBox
- Input field auto-populates when files are selected
- Open/Cancel buttons
- **Focus restoration after directory navigation**

## Architecture

### Event Processing Flow

The FileDialog uses a clean separation between event handling and state synchronization:

```rust
// Let the dialog (and its children) handle the event first
self.dialog.handle_event(&mut event);

// After event is processed, check if ListBox selection changed
self.sync_inputline_with_listbox();
```

This eliminates double-processing by allowing the ListBox to handle its own navigation events, then reading the result.

### Focus Management After Navigation

When navigating directories, proper focus restoration is critical:

```rust
// Matches Borland: fileList->select() calls owner->setCurrent(this, normalSelect)
self.dialog.set_focus_to_child(CHILD_LISTBOX);
```

This properly updates both the Group's `focused` index and the child's visual focus state.

## Major Bug Fixes (2025-11-02)

### 1. Double Event Processing

**Problem**: Events were processed twice - once by FileDialog manually, then by ListBox.

**Solution**: Removed pre-event interception. Let ListBox handle events, then sync InputLine with the result.

**Files Modified**:
- `src/views/file_dialog.rs` - Event processing order
- `src/views/view.rs` - Added `get_list_selection()` trait method
- `src/views/listbox.rs` - Implemented `get_list_selection()`

### 2. InputLine Not Updating

**Problem**: Initial selection after directory change wasn't broadcast to InputLine.

**Solution**: Added broadcast of first item selection after `rebuild_and_redraw()`.

**Reference**: Borland's `TFileList::readDirectory()` (tfilelis.cc:588-595) broadcasts `cmFileFocused` after `newList()`.

### 3. Focus "Limbo" State

**Problem**: ListBox appeared focused (correct colors) but didn't respond to keyboard until Tab was pressed.

**Root Cause**: Manual `set_focus()` calls only updated the child's visual state, not the Group's internal `focused` index.

**Solution**: Added `set_focus_to_child()` method hierarchy that updates both visual and logical focus.

**Files Modified**:
- `src/views/window.rs` - Added `set_focus_to_child()`
- `src/views/dialog.rs` - Exposed `set_focus_to_child()`
- `src/views/file_dialog.rs` - Used proper focus method

## Borland Reference Code

Key files from original implementation:
- `tfiledia.cc:251-302` - TFileDialog::valid() navigation logic
- `tfiledia.cc:275,287` - fileList->select() calls
- `tfilelis.cc:73-76` - TFileList::focusItem() broadcasts
- `tfilelis.cc:588-595` - readDirectory() initial broadcast
- `tview.cc:658-664` - TView::select() calls owner->setCurrent()
- `tgroup.cc` - TGroup::setCurrent() and focusView()

## Testing Checklist

After fixes, the FileDialog should:
- ✅ Navigate up/down by exactly 1 position per keypress
- ✅ Show correct file in InputLine at all times
- ✅ Respond to ENTER key on folders by navigating into them
- ✅ Keep focus on ListBox after directory navigation
- ✅ Respond to keyboard immediately (no Tab needed)
- ✅ Handle mouse clicks and double-clicks correctly
- ✅ Support PgUp/PgDn, Home/End navigation

---

# Screen Dump System

## Overview

The screen dump system provides global keyboard shortcuts to capture terminal output for debugging, documentation, and testing. It works at the Terminal level, ensuring universal availability without requiring integration code.

## Keyboard Shortcuts

Two shortcuts are available at any time during application execution:

- **F12** - Dump entire screen to `screen-dump.txt`
- **F11** - Dump active window/dialog to `active-view-dump.txt`

Both shortcuts provide:
- **Visual Feedback**: Brief screen flash (color inversion) to confirm capture
- **Silent Operation**: Errors don't crash the app
- **Instant Capture**: Screen is captured immediately in its current state

## Usage

### Basic Usage

Simply press the shortcuts while your application is running:

```rust
use turbo_vision::app::Application;

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;
    // ... set up your UI ...
    app.run();  // Press F12 or F11 anytime!
    Ok(())
}
```

### Viewing Dumps

```bash
cat screen-dump.txt           # View full screen dump
cat active-view-dump.txt      # View active window/dialog dump
less -R screen-dump.txt       # For scrollable viewing
```

## Architecture: Terminal-Level Implementation

The shortcuts are implemented at the **Terminal level** in `poll_event()` and `read_event()`, providing significant architectural benefits:

### Benefits

1. **Universal Availability** - Works everywhere without any integration:
   - ✅ `Application::run()` event loop
   - ✅ `Dialog::execute()` event loop
   - ✅ Custom event loops
   - ✅ Any code that calls `terminal.poll_event()` or `terminal.read_event()`

2. **Zero Configuration** - No need to:
   - Add hooks in Application
   - Add hooks in Dialog
   - Add hooks in every custom event loop
   - Remember to call special handler functions

3. **Cannot Be Blocked** - Since shortcuts are handled before events are returned:
   - Event handlers can't accidentally consume the shortcut
   - Always works, regardless of application state

4. **Clean Separation of Concerns**:
   - Terminal layer: Handles low-level I/O and global system shortcuts
   - Application layer: Handles business logic and UI events
   - View layer: Handles widget-specific behavior

### Implementation

```rust
// In Terminal::poll_event()
if key_code == KB_F12 {
    let _ = self.flash();
    let _ = self.dump_screen("screen-dump.txt");
    return Ok(None);  // Event consumed, not propagated
}

if key_code == KB_F11 {
    let _ = self.flash();
    if let Some(bounds) = self.active_view_bounds {
        let _ = self.dump_region(..., "active-view-dump.txt");
    }
    return Ok(None);  // Event consumed, not propagated
}
```

### Visual Flash Effect

The flash effect provides clear visual feedback:

1. Saves the current buffer
2. Inverts all foreground and background colors
3. Flushes the inverted screen
4. Waits 50ms
5. Restores the original buffer
6. Flushes the restored screen

This provides immediate confirmation that the capture succeeded.

## Programmatic API

### High-Level API

```rust
// Dump entire screen
terminal.dump_screen("screen.ans")?;

// Dump a specific view (works with any View implementor)
dialog.dump_to_file(&terminal, "dialog.ans")?;

// Dump a specific region
terminal.dump_region(10, 5, 40, 20, "region.ans")?;

// Flash the screen manually
terminal.flash()?;
```

### Low-Level API

```rust
use turbo_vision::core::ansi_dump;

// Get buffer and dump it manually
let buffer = terminal.buffer();
ansi_dump::dump_buffer_to_file(buffer, width, height, "custom.ans")?;

// Dump to any writer
let mut writer = std::io::stdout();
ansi_dump::dump_buffer(&mut writer, buffer, width, height)?;
```

## File Format

The generated files use standard ANSI escape sequences:
- `\x1b[XXm` for foreground colors (30-37, 90-97)
- `\x1b[XXm` for background colors (40-47, 100-107)
- `\x1b[0m` to reset colors at end of each line

Files can be viewed on any system with ANSI support using `cat`, `less -R`, or text editors.

## Use Cases

1. **Debugging** - Visualize exactly what's in the terminal buffer
2. **Bug Reports** - Users can press F12 and send you the output file
3. **Documentation** - Capture screenshots of terminal UI
4. **Testing** - Create visual regression tests
5. **Development** - Quickly inspect layout issues

## Implementation Files

- `src/core/ansi_dump.rs` - ANSI dump functionality
- `src/terminal/mod.rs` - Terminal methods and shortcut handlers
- `src/views/view.rs` - View trait `dump_to_file()` method
- `examples/dump_demo.rs` - Complete working example

---

# Command Set System

## Overview

The Command Set system provides automatic button enable/disable based on application state. This matches Borland Turbo Vision's architecture where buttons automatically disable themselves when their associated commands are not available.

## Implementation Status

### ✅ Completed

1. **CommandSet Struct** (`src/core/command_set.rs`)
   - Bitfield-based storage for 65,536 commands
   - Individual command enable/disable
   - Range-based operations
   - Set operations (union, intersect)
   - Matches Borland's `TCommandSet` implementation

2. **Command Constants** (`src/core/command.rs`)
   - Added `CM_COMMAND_SET_CHANGED = 52`
   - Added related broadcast commands

3. **Event System Updates** (`src/core/event.rs`)
   - Added `Event::broadcast(cmd)` constructor
   - EventType::Broadcast support

### 🚧 Future Work

4. **Global Command Set Storage**
   - Need thread-local storage or Application reference passing
   - Borland uses `TView::curCommandSet` (static)

5. **Command Set API Methods**
   - Methods on Application/View:
     - `enable_command(cmd)`
     - `disable_command(cmd)`
     - `command_enabled(cmd) -> bool`

6. **Idle Processing & Broadcast**
   - Application::idle() method
   - Check if command_set_changed flag is set
   - Broadcast CM_COMMAND_SET_CHANGED to all views

7. **Button Auto-Disable**
   - Update Button constructor to check `command_enabled()`
   - Add broadcast handler for CM_COMMAND_SET_CHANGED

## Architecture Design

### Borland's Approach

```cpp
// Global static in TView
static TCommandSet curCommandSet;
static Boolean commandSetChanged;

// Any view can modify
void TView::enableCommand(ushort cmd) {
    commandSetChanged = True;
    curCommandSet += cmd;
}

// Program broadcasts changes
void TProgram::idle() {
    if (commandSetChanged) {
        message(this, evBroadcast, cmCommandSetChanged, 0);
        commandSetChanged = False;
    }
}

// Buttons respond
case cmCommandSetChanged:
    if (command enabled changed) {
        setState(sfDisabled, !commandEnabled(command));
        drawView();
    }
```

### Rust Approach (Target)

```rust
// Store in Application
pub struct Application {
    command_set: CommandSet,
    command_set_changed: bool,
    // ...
}

// Views access via Application reference
impl Application {
    pub fn enable_command(&mut self, cmd: CommandId) {
        self.command_set_changed = true;
        self.command_set.enable_command(cmd);
    }

    pub fn command_enabled(&self, cmd: CommandId) -> bool {
        self.command_set.has(cmd)
    }

    fn idle(&mut self) {
        if self.command_set_changed {
            let event = Event::broadcast(CM_COMMAND_SET_CHANGED);
            self.desktop.handle_event(&mut event);
            self.command_set_changed = false;
        }
    }
}
```

## Target Usage Example

```rust
// In application setup
app.disable_command(CM_PASTE);  // No clipboard content yet
app.disable_command(CM_UNDO);   // Nothing to undo

// User copies text
clipboard.set_text("Hello");
app.enable_command(CM_PASTE);  // Button automatically enables!

// User performs action
perform_action();
app.enable_command(CM_UNDO);   // Undo button lights up!

// User undoes
undo();
if !can_undo_more() {
    app.disable_command(CM_UNDO);  // Button automatically disables!
}
```

## Benefits

When fully implemented, the command set system:
- Eliminates manual button enable/disable code throughout the application
- Buttons "just work" based on application state
- Provides consistent UI state management
- Matches original Turbo Vision patterns exactly

## References

- Borland: `/local-only/borland-tvision/include/tv/cmdset.h`
- Borland: `/local-only/borland-tvision/classes/tcommand.cc`
- Borland: `/local-only/borland-tvision/classes/tview.cc` (lines 142-389)
- Borland: `/local-only/borland-tvision/classes/tbutton.cc` (lines 255-262)
- Borland: `/local-only/borland-tvision/classes/tprogram.cc` (lines 248-257)

---

## Related Documentation

- **TO-DO-LIST.md** - Missing features and implementation roadmap
- **DISCREPANCIES.md** - Differences from original Borland Turbo Vision

---

## Contributing

When adding new features or fixing bugs:

1. Consult the original Borland Turbo Vision source code for reference patterns
2. Document any architectural decisions or deviations
3. Update this design document with new patterns or learnings
4. Reference original source locations in comments (e.g., `tfiledia.cc:275`)
5. Maintain compatibility with Borland's architecture where reasonable

## Version History

- **2025-11-02** - Added FileDialog fixes documentation, consolidated design docs
- **2025-11-01** - Added focus architecture and screen dump system docs
- **2025-XX-XX** - Initial version
