# Turbo Vision for Rust - Implementation Analysis

## Executive Summary

I have thoroughly analyzed the Rust implementation of Turbo Vision and created comprehensive documentation for converting Pascal-based Turbo Vision documentation to Rust. Three reference documents have been created in the repository:

1. **RUST_IMPLEMENTATION_REFERENCE.md** (23 KB) - Comprehensive implementation guide
2. **KEY_FILES_SUMMARY.txt** (8 KB) - File organization and locations
3. **QUICK_REFERENCE.txt** (9 KB) - Quick code patterns and examples

---

## Key Findings

### 1. Events and Commands

**Event System** (`src/core/event.rs`):
- Unified `Event` struct with `what: EventType`, `key_code`, `key_modifiers`, `mouse`, `command`
- 9 event types: Nothing, Keyboard, MouseDown, MouseUp, MouseMove, MouseAuto, MouseWheelUp/Down, Command, Broadcast
- Standard key codes defined as constants (KB_F1, KB_ESC, KB_ENTER, etc.)
- Helper constructors: `Event::keyboard()`, `Event::command()`, `Event::broadcast()`, `Event::mouse()`

**Command System** (`src/core/command.rs`):
- Commands are u16 IDs
- Standard commands: CM_OK, CM_CANCEL, CM_YES, CM_NO, CM_QUIT, CM_CLOSE
- System broadcasts: CM_COMMAND_SET_CHANGED, CM_RECEIVED_FOCUS, CM_RELEASED_FOCUS
- Menu commands: CM_NEW, CM_OPEN, CM_SAVE, CM_CUT, CM_COPY, CM_PASTE, CM_UNDO, CM_REDO
- Custom commands use 100+ range

### 2. Handle Event Implementation

**View Trait** (`src/views/view.rs`):
- Core method: `fn handle_event(&mut self, event: &mut Event)`
- Event modification pattern: children transform events to communicate with parents
- Three patterns:
  1. **Consume**: `event.clear()` - prevent parent from seeing it
  2. **Transform**: `*event = Event::command(cmd)` - change event type for parent
  3. **Bubble**: Leave unchanged - parent processes it

**Key Insight - Event Transformation vs Parent Pointers**:
```
Borland (Pascal):  message(owner, evBroadcast, command, this);  // Direct parent pointer
Rust:              *event = Event::command(command);           // Event modification + call stack

Same result achieved without unsafe pointers via event mutation and call stack
```

**Group Event Distribution** (`src/views/group.rs`):
- Distributes to children in reverse order (topmost window first)
- Matches Borland's TGroup event distribution
- Children can consume, transform, or pass through events

### 3. Menu Bar Implementation

**MenuBar** (`src/views/menu_bar.rs`):
- Horizontal menu bar at top (created with 1 row height)
- Submenus can have cascading menus
- MenuItem types: Regular, SubMenu, Separator
- Keyboard shortcuts marked with tildes: "~N~ew" → Alt+N
- Mouse support: click items, hover highlighting
- Integration: `app.set_menu_bar(menu_bar)`

**Complete Example**:
```rust
let mut menu_bar = MenuBar::new(Rect::new(0, 0, width as i16, 1));
let file_items = vec![
    MenuItem::with_shortcut("~N~ew", CM_NEW, 0, "Ctrl+N", 0),
    MenuItem::separator(),
    MenuItem::with_shortcut("E~x~it", CM_QUIT, 0, "Alt+X", 0),
];
menu_bar.add_submenu(SubMenu::new("~F~ile", Menu::from_items(file_items)));
app.set_menu_bar(menu_bar);
```

### 4. Status Line Implementation

**StatusLine** (`src/views/status_line.rs`):
- Single row at bottom showing keyboard shortcuts and commands
- StatusItem: text, key_code, command
- Mouse hover support: items highlight when hovered
- Mouse clicks: clicking item sends its command
- Keyboard shortcuts: pressing key_code sends command
- Optional hint text on right side

**Complete Example**:
```rust
let status_line = StatusLine::new(
    Rect::new(0, height as i16 - 1, width as i16, height as i16),
    vec![
        StatusItem::new("~F1~ Help", KB_F1, CM_HELP),
        StatusItem::new("~F10~ Menu", KB_F10, CM_QUIT),
    ],
);
app.set_status_line(status_line);
```

### 5. Message Boxes

**msgbox Module** (`src/views/msgbox.rs`):
Library functions, not classes:
- `message_box_ok()` - simple message with OK
- `message_box_warning()`, `message_box_error()` - with icon
- `confirmation_box()` - Yes/No/Cancel
- `confirmation_box_yes_no()` - Yes/No only
- `confirmation_box_ok_cancel()` - OK/Cancel
- `input_box()` - text input with label
- `search_box()` - find text
- `search_replace_box()` - find and replace
- `goto_line_box()` - line number input

All return CommandId (CM_YES, CM_NO, CM_OK, CM_CANCEL) or Option<String> for input

### 6. Command Enable/Disable

**CommandSet** (`src/core/command_set.rs`):
- Global thread-local command set (matches Borland's static)
- Bitfield with 2048 words (supports 65,536 commands)
- Functions:
  - `command_enabled(cmd)` - check if enabled
  - `enable_command(cmd)` - enable single
  - `disable_command(cmd)` - disable single
  - `enable_range(start, end)` - enable range
  - `disable_range(start, end)` - disable range
  - `command_set_changed()` - check if changed
  - `clear_command_set_changed()` - clear flag

**Automatic Button Updates**:
1. Button constructor checks `command_set::command_enabled(command)`
2. Sets SF_DISABLED flag if disabled
3. Application::idle() broadcasts CM_COMMAND_SET_CHANGED
4. Buttons' handle_event() receives broadcast
5. Button updates its disabled state automatically

### 7. Modal Loop Pattern

**Group::execute()** (`src/views/group.rs`):
Matches Borland's TGroup::execute() from tgroup.cc:182-195
```rust
pub fn execute(&mut self, app: &mut Application) -> CommandId {
    self.end_state = 0;
    
    loop {
        if let Some(mut event) = app.get_event() {
            self.handle_event(&mut event);
        }
        
        if self.end_state != 0 {
            break;
        }
    }
    
    self.end_state
}
```

**Two Execution Patterns**:
1. **Direct (Self-contained)**: `dialog.execute(&mut app)` - simpler, runs its own loop
2. **Centralized (Borland-style)**: `app.exec_view(dialog)` - app manages loop

Both work identically; Pattern 1 is simpler for direct use.

---

## Architecture Patterns

### Event Flow

```
Application::handle_event(&mut event)
  ├→ MenuBar::handle_event(&mut event)
  ├→ Desktop::handle_event(&mut event)
  │   └→ Window::handle_event(&mut event)
  │       └→ Group::handle_event(&mut event)
  │           └→ Children (reverse order, top first)
  └→ StatusLine::handle_event(&mut event)
```

Each level can:
1. **Consume**: `event.clear()` - mark as handled
2. **Transform**: change `event.what` or `event.command`
3. **Bubble**: leave unchanged

### Focus Management

- Focus distributed through view hierarchy
- Tab key moves forward, Shift+Tab backward
- Group tracks focused child index
- Views with `can_focus() == true` can receive focus

### State Flags

```rust
SF_FOCUSED = 0x0001      // Has input focus
SF_DISABLED = 0x0002     // Disabled (grayed out)
SF_SHADOW = 0x0004       // Draw shadow
SF_MODAL = 0x0008        // Modal window
SF_HIDDEN = 0x0010       // Hidden, don't draw

OF_PRE_PROCESS = 0x0100  // Process events first
OF_POST_PROCESS = 0x0200 // Process events last
```

---

## Example Files in Repository

Best examples to study for implementation:

1. **examples/menu.rs** (334 lines) - BEST EXAMPLE
   - Menu bar with File/Edit/Help menus
   - Status line with keyboard shortcuts
   - Message boxes (OK, Yes/No/Cancel, Input)
   - Cascading submenus
   - Right-click popup menus
   - All features integrated

2. **examples/command_set_demo.rs** (171 lines)
   - Shows button enable/disable
   - Command broadcast system
   - Automatic UI updates

3. **examples/dialogs_demo.rs** (69 lines)
   - All message box types
   - Input/confirmation dialogs
   - Clean, simple examples

4. **examples/event_debug.rs** (79 lines)
   - Event inspection and debugging
   - Shows how to identify event types

---

## Key Differences from Pascal Borland

| Feature | Borland | Rust |
|---------|---------|------|
| Parent Communication | `message(owner, ...)` | Event modification + call stack |
| Event Handler | `virtual handleEvent()` | `fn handle_event(&mut event)` |
| Menu Initialization | `initMenuBar()` method | `app.set_menu_bar(MenuBar::new(...))` |
| Status Line Init | `initStatusLine()` method | `app.set_status_line(StatusLine::new(...))` |
| Command State | Global `TView::curCommandSet` | Thread-local `GLOBAL_COMMAND_SET` |
| Modal Loop | `TGroup::execute()` with `endState` | Same pattern, `Group::execute()` |
| Message Boxes | Classes (TMessageBox, etc.) | Functions (message_box_ok, etc.) |

---

## Implementation Summary by Component

### Events (src/core/event.rs)
- 339 lines
- 9 event types
- Unified event structure
- Standard key codes
- Helper constructors

### Commands (src/core/command.rs)
- 65 lines
- CommandId type
- 30+ standard commands
- Custom command range (100+)

### Handle Event (src/views/view.rs)
- 200 lines
- View trait
- handle_event() method
- State flag management
- Focus management

### Button (src/views/button.rs)
- 200+ lines
- Event handling
- Command sending
- Enable/disable support
- Broadcast handling

### Group/Dialog (src/views/group.rs, dialog.rs)
- 600+ lines total
- Modal loop
- Child management
- Focus distribution
- Event distribution

### Menu Bar (src/views/menu_bar.rs)
- 500+ lines
- Dropdown menus
- Cascading submenus
- Keyboard navigation
- Mouse support

### Status Line (src/views/status_line.rs)
- 232 lines
- Keyboard shortcut display
- Mouse hover highlighting
- Click handling
- Hint text

### Message Boxes (src/views/msgbox.rs)
- 411 lines
- 10+ dialog types
- Simple to advanced
- Input/confirmation dialogs
- Search dialogs

### Command Set (src/core/command_set.rs)
- 352 lines
- Global command state
- Bitfield (2048 words)
- Enable/disable operations
- Range operations

### Application (src/app/application.rs)
- 311 lines
- Event routing
- Drawing coordination
- Component integration
- Modal loop management

---

## Documentation Created

All documentation files are in the repository root:

1. **RUST_IMPLEMENTATION_REFERENCE.md** (23 KB)
   - Comprehensive implementation guide
   - All 7 core features
   - Complete code examples
   - Architecture patterns
   - Comparison with Borland

2. **KEY_FILES_SUMMARY.txt** (8 KB)
   - File-by-file breakdown
   - Feature organization
   - Example programs
   - Architecture patterns
   - Key differences

3. **QUICK_REFERENCE.txt** (9 KB)
   - Code snippets
   - Common patterns
   - Command constants
   - Key codes
   - Quick lookup

---

## How to Use This Information

For converting Turbo Vision documentation from Pascal to Rust:

1. **Start with examples**: Study `examples/menu.rs` - it demonstrates all features
2. **Use QUICK_REFERENCE.txt**: Copy code patterns for common tasks
3. **Reference RUST_IMPLEMENTATION_REFERENCE.md**: Deep dive into architecture
4. **Check KEY_FILES_SUMMARY.txt**: Find where features are implemented

Key transformations:
- Virtual methods → trait implementations
- Parent pointers → event modification
- Object methods → function calls
- Global state → thread-local statics

---

## Files Located

All relevant source files have been identified and documented:
- 8 core modules covering all features
- 20+ examples demonstrating usage
- Complete file paths with line counts
- Code organization by feature

The Rust implementation closely follows Borland's architecture but adapts it to Rust's ownership model, using event transformation instead of parent pointers and achieving the same functionality without unsafe code.
