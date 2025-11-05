# Turbo Vision for Rust - Implementation Reference

Complete guide to understanding how Turbo Vision for Rust implements events, commands, menus, status lines, and message boxes. This document is designed to help convert Pascal-based Turbo Vision documentation to Rust.

---

## Table of Contents

1. [Event System](#event-system)
2. [Command System](#command-system)
3. [Handle Event Pattern](#handle-event-pattern)
4. [Menu Bar Implementation](#menu-bar-implementation)
5. [Status Line Implementation](#status-line-implementation)
6. [Message Boxes](#message-boxes)
7. [Command Enable/Disable](#command-enabledisable)
8. [Key Architecture Patterns](#key-architecture-patterns)

---

## Event System

### Core Event Types

**Location**: `src/core/event.rs`

```rust
// Event type enumeration
pub enum EventType {
    Nothing,          // No event
    Keyboard,         // Keyboard input
    MouseDown,        // Mouse button pressed
    MouseUp,          // Mouse button released
    MouseMove,        // Mouse moved
    MouseAuto,        // Auto-repeat mouse event
    MouseWheelUp,     // Mouse wheel scrolled up
    MouseWheelDown,   // Mouse wheel scrolled down
    Command,          // Command event (from buttons, menus, etc.)
    Broadcast,        // Broadcast message to all views
}

// Unified event structure
pub struct Event {
    pub what: EventType,           // Event type
    pub key_code: KeyCode,         // Keyboard code (for EventType::Keyboard)
    pub key_modifiers: KeyModifiers, // Ctrl, Shift, Alt modifiers
    pub mouse: MouseEvent,         // Mouse data (for mouse events)
    pub command: CommandId,        // Command ID (for EventType::Command/Broadcast)
}
```

### Creating Events

```rust
// Keyboard event
let event = Event::keyboard(KB_F1);

// Command event
let event = Event::command(CM_OK);

// Broadcast event
let event = Event::broadcast(CM_COMMAND_SET_CHANGED);

// Mouse event
let event = Event::mouse(
    EventType::MouseDown,
    Point::new(10, 5),
    MB_LEFT_BUTTON,
    false  // double_click
);
```

### Standard Key Codes

**Key Constants** (from `src/core/event.rs`):

```rust
// Special keys
pub const KB_ESC: KeyCode = 0x011B;
pub const KB_ENTER: KeyCode = 0x1C0D;
pub const KB_BACKSPACE: KeyCode = 0x0E08;
pub const KB_TAB: KeyCode = 0x0F09;
pub const KB_SHIFT_TAB: KeyCode = 0x0F00;

// Function keys F1-F12
pub const KB_F1: KeyCode = 0x3B00;
pub const KB_F10: KeyCode = 0x4400;
pub const KB_F12: KeyCode = 0x8600;

// Arrow keys
pub const KB_UP: KeyCode = 0x4800;
pub const KB_DOWN: KeyCode = 0x5000;
pub const KB_LEFT: KeyCode = 0x4B00;
pub const KB_RIGHT: KeyCode = 0x4D00;

// Alt key combinations
pub const KB_ALT_X: KeyCode = 0x2D00;
pub const KB_ALT_F: KeyCode = 0x2100;
```

### Event Masks

For filtering which event types a view accepts:

```rust
pub const EV_MOUSE: u16 = 0x003F;      // All mouse events
pub const EV_KEYBOARD: u16 = 0x0040;   // Keyboard events
pub const EV_COMMAND: u16 = 0x0100;    // Command events
pub const EV_BROADCAST: u16 = 0x0200;  // Broadcast messages
```

---

## Command System

### Standard Commands

**Location**: `src/core/command.rs`

```rust
pub type CommandId = u16;

// Standard dialog button commands
pub const CM_OK: CommandId = 10;
pub const CM_CANCEL: CommandId = 11;
pub const CM_YES: CommandId = 12;
pub const CM_NO: CommandId = 13;

// Window/Application commands
pub const CM_QUIT: CommandId = 24;
pub const CM_CLOSE: CommandId = 25;

// System broadcast commands
pub const CM_COMMAND_SET_CHANGED: CommandId = 52;  // Command availability changed
pub const CM_RECEIVED_FOCUS: CommandId = 50;       // View received focus
pub const CM_RELEASED_FOCUS: CommandId = 51;       // View lost focus

// Standard menu commands
pub const CM_NEW: CommandId = 102;
pub const CM_OPEN: CommandId = 103;
pub const CM_SAVE: CommandId = 104;
pub const CM_CUT: CommandId = 112;
pub const CM_COPY: CommandId = 113;
pub const CM_PASTE: CommandId = 114;
pub const CM_UNDO: CommandId = 110;
pub const CM_REDO: CommandId = 111;

// Custom application commands start at 100+
pub const CM_ABOUT: CommandId = 100;
```

### Creating Custom Commands

Simply define constants:

```rust
// In your application
const CMD_ENABLE_EDITS: CommandId = 200;
const CMD_DISABLE_EDITS: CommandId = 201;
const CMD_GENERAL_PREFS: CommandId = 204;

// Send command from a button
let button = Button::new(bounds, "~E~nable", CMD_ENABLE_EDITS, false);
```

---

## Handle Event Pattern

### The View Trait

**Location**: `src/views/view.rs`

```rust
pub trait View {
    fn bounds(&self) -> Rect;
    fn set_bounds(&mut self, bounds: Rect);
    fn draw(&mut self, terminal: &mut Terminal);
    
    /// Main event handler - ALL event processing happens here
    /// CRITICAL: Modify event IN PLACE to communicate with parent/siblings
    fn handle_event(&mut self, event: &mut Event);
    
    fn can_focus(&self) -> bool { false }
    fn set_focus(&mut self, focused: bool) { }
    fn is_focused(&self) -> bool { false }
    // ... other methods ...
}
```

### Handle Event Patterns

#### Pattern 1: Consume an event

When a view handles an event and doesn't want others to see it:

```rust
impl View for MyView {
    fn handle_event(&mut self, event: &mut Event) {
        match event.what {
            EventType::Keyboard => {
                if event.key_code == KB_ENTER {
                    // Handle the event
                    do_something();
                    // Clear the event so parent doesn't process it
                    event.clear();
                    return;
                }
            }
            _ => {}
        }
    }
}
```

#### Pattern 2: Transform event into command (child-to-parent)

When a child view wants to signal its parent:

```rust
impl View for Button {
    fn handle_event(&mut self, event: &mut Event) {
        if event.what == EventType::Keyboard && event.key_code == KB_ENTER && self.is_focused() {
            // Transform keyboard event into command
            // This bubbles up through parent's handle_event() call stack
            *event = Event::command(self.command);
            return;
        }
        
        if event.what == EventType::MouseDown && /* clicked */ {
            // Send command to parent via event transformation
            *event = Event::command(self.command);
            return;
        }
    }
}
```

#### Pattern 3: Let event bubble up

When a view doesn't handle an event:

```rust
fn handle_event(&mut self, event: &mut Event) {
    // Don't clear event, don't modify it
    // Just return - parent will process it
}
```

### Event Flow (Rust vs Borland)

**Borland Pattern** (uses pointer to parent):
```cpp
void TButton::press() {
    message(owner, evBroadcast, command, this);  // Send to owner
}
```

**Rust Pattern** (uses event modification):
```rust
fn handle_event(&mut self, event: &mut Event) {
    *event = Event::command(self.command);  // Transform and bubble up
}
```

Both achieve the same result but Rust uses ownership and the call stack instead of raw pointers.

### Group Event Distribution

**Location**: `src/views/group.rs`

Groups distribute events to children in reverse order (top-to-bottom in z-order):

```rust
impl View for Group {
    fn handle_event(&mut self, event: &mut Event) {
        // Distribute to children in reverse order (top child first)
        // This matches Borland's event distribution
        for i in (0..self.children.len()).rev() {
            if event.what == EventType::Nothing {
                break;  // Event was handled
            }
            self.children[i].handle_event(event);
        }
    }
}
```

---

## Menu Bar Implementation

### Creating a Menu Bar

**Location**: `src/views/menu_bar.rs`, example in `examples/menu.rs`

```rust
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::core::geometry::Rect;

// Create menu bar at top of screen
let mut menu_bar = MenuBar::new(Rect::new(0, 0, width as i16, 1));

// Create File menu items
let file_menu_items = vec![
    MenuItem::with_shortcut("~N~ew", CM_NEW, 0, "Ctrl+N", 0),
    MenuItem::with_shortcut("~O~pen...", CM_OPEN, 0, "Ctrl+O", 0),
    MenuItem::separator(),
    MenuItem::with_shortcut("~S~ave", CM_SAVE, 0, "Ctrl+S", 0),
    MenuItem::with_shortcut("E~x~it", CM_QUIT, 0, "Alt+X", 0),
];

// Create submenu
let file_menu = SubMenu::new("~F~ile", Menu::from_items(file_menu_items));

// Add to menu bar
menu_bar.add_submenu(file_menu);

// Add to application
app.set_menu_bar(menu_bar);
```

### Menu Item Types

```rust
pub enum MenuItem {
    Regular {
        text: String,           // "~N~ew" (tilde marks shortcut)
        command: CommandId,     // CM_NEW
        disabled: bool,         // Disabled state
        shortcut: Option<String>, // "Ctrl+N" for display
    },
    SubMenu {
        text: String,           // "~R~ecent Files"
        command: CommandId,     // Unused for submenus
        menu: Menu,             // Submenu items
    },
    Separator,                  // Visual separator line
}

impl MenuItem {
    pub fn with_shortcut(text: &str, cmd: CommandId, disabled: bool, 
                         shortcut: &str, flags: u16) -> Self {
        // Create item with display shortcut text
    }
    
    pub fn submenu(text: &str, cmd: CommandId, menu: Menu, flags: u16) -> Self {
        // Create cascading submenu
    }
    
    pub fn separator() -> Self {
        // Create separator line
    }
}
```

### Keyboard Shortcuts in Menu Items

The tilde (~) marks keyboard shortcuts:

```rust
// First letter after tilde is highlighted and used as shortcut key
MenuItem::with_shortcut("~N~ew", CM_NEW, 0, "Ctrl+N", 0),
// Alt+N will trigger this menu item
```

### Handling Menu Events

Menu events are generated when user selects a menu item:

```rust
// In application event loop
if event.what == EventType::Command {
    match event.command {
        CM_NEW => {
            // Handle New command
        }
        CM_OPEN => {
            // Handle Open command
        }
        _ => {}
    }
}
```

### Menu Position and Cascading Submenus

```rust
// After menu bar handles event, check for cascading submenus
if let Some(ref mut menu_bar) = app.menu_bar {
    menu_bar.handle_event(&mut event);
    
    if let Some(command) = menu_bar.check_cascading_submenu(&mut app.terminal) {
        if command != 0 {
            event = Event::command(command);
        }
    }
}
```

---

## Status Line Implementation

### Creating a Status Line

**Location**: `src/views/status_line.rs`

```rust
use turbo_vision::views::status_line::{StatusLine, StatusItem};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::event::KB_F10;

let status_line = StatusLine::new(
    Rect::new(0, height as i16 - 1, width as i16, height as i16),
    vec![
        StatusItem::new("~F1~ Help", KB_F1, CM_HELP),
        StatusItem::new("~F10~ Menu", KB_F10, CM_QUIT),
    ],
);

app.set_status_line(status_line);
```

### StatusItem Structure

```rust
pub struct StatusItem {
    pub text: String,              // "~F10~ Menu" (tilde marks shortcut)
    pub key_code: KeyCode,         // KB_F10 (keyboard trigger)
    pub command: CommandId,        // CM_QUIT (command to send)
}

impl StatusItem {
    pub fn new(text: &str, key_code: KeyCode, command: CommandId) -> Self {
        // text: display text with ~X~ for highlights
        // key_code: keyboard shortcut code
        // command: command to execute when clicked or key pressed
    }
}
```

### Status Line Features

- **Visual Feedback**: Items are highlighted differently when mouse hovers over them
- **Keyboard Shortcuts**: Each item has a key code that triggers its command
- **Mouse Clicks**: Clicking a status item generates its command
- **Hint Text**: Optional context-sensitive help text on the right side

```rust
// Set hint text
if let Some(ref mut status_line) = app.status_line {
    status_line.set_hint(Some("Ready".to_string()));
}
```

---

## Message Boxes

### Location

`src/views/msgbox.rs`

### Simple Message Box (OK button only)

```rust
use turbo_vision::views::msgbox::message_box_ok;

// Just show message and wait for OK
message_box_ok(&mut app, "File saved successfully!");
```

### Information/Warning/Error Boxes

```rust
use turbo_vision::views::msgbox::{
    message_box_ok,
    message_box_warning,
    message_box_error,
};

message_box_ok(&mut app, "This is information");
message_box_warning(&mut app, "This is a warning");
message_box_error(&mut app, "An error occurred");
```

### Confirmation Dialogs

```rust
use turbo_vision::views::msgbox::{confirmation_box, confirmation_box_yes_no};
use turbo_vision::core::command::{CM_YES, CM_NO, CM_CANCEL};

// Yes/No/Cancel
let result = confirmation_box(&mut app, "Save changes?");
match result {
    r if r == CM_YES => { /* save */ }
    r if r == CM_NO => { /* don't save */ }
    _ => { /* cancel */ }
}

// Yes/No only
let result = confirmation_box_yes_no(&mut app, "Continue?");
match result {
    r if r == CM_YES => { /* yes */ }
    _ => { /* no */ }
}
```

### Input Dialog

```rust
use turbo_vision::views::msgbox::input_box;

if let Some(name) = input_box(&mut app, "Name", "Enter name:", "", 50) {
    println!("You entered: {}", name);
} else {
    println!("Cancelled");
}
```

### Search Dialog

```rust
use turbo_vision::views::msgbox::search_box;

if let Some(search_text) = search_box(&mut app, "Search") {
    // Perform search
}
```

### Search and Replace Dialog

```rust
use turbo_vision::views::msgbox::search_replace_box;

if let Some((find, replace)) = search_replace_box(&mut app, "Replace") {
    // Perform find and replace
}
```

### Goto Line Dialog

```rust
use turbo_vision::views::msgbox::goto_line_box;

if let Some(line_number) = goto_line_box(&mut app, "Go to Line") {
    // Jump to line
}
```

### Message Box Flags

```rust
pub const MF_WARNING: u16 = 0x0000;          // Warning icon
pub const MF_ERROR: u16 = 0x0001;            // Error icon
pub const MF_INFORMATION: u16 = 0x0002;      // Information icon
pub const MF_CONFIRMATION: u16 = 0x0003;     // Confirmation icon

pub const MF_YES_BUTTON: u16 = 0x0100;       // Yes button
pub const MF_NO_BUTTON: u16 = 0x0200;        // No button
pub const MF_OK_BUTTON: u16 = 0x0400;        // OK button
pub const MF_CANCEL_BUTTON: u16 = 0x0800;    // Cancel button

pub const MF_YES_NO_CANCEL: u16 = 
    MF_YES_BUTTON | MF_NO_BUTTON | MF_CANCEL_BUTTON;
pub const MF_OK_CANCEL: u16 = 
    MF_OK_BUTTON | MF_CANCEL_BUTTON;
```

### Advanced: Custom Message Box

```rust
use turbo_vision::views::msgbox::message_box_rect;

// Create custom-sized message box at specific location
let result = message_box_rect(
    &mut app,
    Rect::new(10, 5, 60, 15),  // Custom bounds
    "Custom message box",
    MF_INFORMATION | MF_OK_BUTTON
);
```

---

## Command Enable/Disable

### Location

`src/core/command_set.rs`

### Basic Enable/Disable

```rust
use turbo_vision::core::command_set;
use turbo_vision::core::command::CM_COPY;

// Disable a command (e.g., when clipboard is empty)
command_set::disable_command(CM_COPY);
command_set::disable_command(CM_CUT);
command_set::disable_command(CM_PASTE);

// Enable when clipboard has data
command_set::enable_command(CM_COPY);
```

### Check if Command is Enabled

```rust
if command_set::command_enabled(CM_SAVE) {
    // Can save
}
```

### Range Operations

```rust
// Enable all file commands (100-110)
let mut cmd_set = CommandSet::new();
cmd_set.enable_range(100, 110);

// Disable range
cmd_set.disable_range(100, 110);
```

### How Buttons Respond Automatically

When you create a button, it checks if its command is enabled:

```rust
impl Button {
    pub fn new(bounds: Rect, title: &str, command: CommandId, is_default: bool) -> Self {
        // Check if command is initially enabled
        let mut state = 0;
        if !command_set::command_enabled(command) {
            state |= SF_DISABLED;  // Start disabled
        }
        // ...
    }
}
```

Buttons automatically update when `CM_COMMAND_SET_CHANGED` is broadcast:

```rust
impl View for Button {
    fn handle_event(&mut self, event: &mut Event) {
        // Handle broadcast message
        if event.what == EventType::Broadcast {
            if event.command == CM_COMMAND_SET_CHANGED {
                // Update button's disabled state
                let disabled = !command_set::command_enabled(self.command);
                self.set_disabled(disabled);
            }
        }
    }
}
```

### Idle Loop Broadcasting

The Application automatically broadcasts when commands change:

```rust
// In Application::idle()
if command_set::command_set_changed() {
    let mut event = Event::broadcast(CM_COMMAND_SET_CHANGED);
    
    // Broadcast to all views
    self.desktop.handle_event(&mut event);
    if let Some(ref mut menu_bar) = self.menu_bar {
        menu_bar.handle_event(&mut event);
    }
    
    command_set::clear_command_set_changed();
}
```

---

## Key Architecture Patterns

### The Modal Loop Pattern (Borland-style)

**Matches Borland's TGroup::execute()** from `tgroup.cc:182-195`

```rust
// Location: src/views/group.rs, src/views/dialog.rs
impl Group {
    pub fn execute(&mut self, app: &mut Application) -> CommandId {
        self.end_state = 0;
        
        loop {
            // Get event from Application (which handles drawing)
            if let Some(mut event) = app.get_event() {
                self.handle_event(&mut event);
            }
            
            // Check if we should end the modal loop
            if self.end_state != 0 {
                break;
            }
        }
        
        self.end_state
    }
}
```

### Two Execution Patterns

#### Pattern 1: Direct (Self-Contained)

```rust
let mut dialog = Dialog::new(bounds, "Title");
dialog.add(Box::new(Button::new(...)));
let result = dialog.execute(&mut app);  // Runs own event loop
```

#### Pattern 2: Centralized (Borland-style)

```rust
let dialog = Dialog::new_modal(bounds, "Title");
// ... add children ...
let result = app.exec_view(dialog);  // App runs the modal loop
```

### Event Flow Architecture

**Complete event flow** (from Application down):

```
Application::handle_event(&mut event)
  -> Menu Bar handles event
  -> Desktop handles event
       -> Top Window handles event
            -> Group handles event
                 -> Child controls handle event (in reverse order)
  -> Status Line handles event
```

Each level can:
1. **Consume** the event: `event.clear()` so next level doesn't see it
2. **Transform** the event: change `event.what` to pass different event to parent
3. **Let bubble**: leave event unchanged so parent processes it

### State Flags

**Location**: `src/core/state.rs`

```rust
pub const SF_FOCUSED: u16 = 0x0001;        // View has input focus
pub const SF_DISABLED: u16 = 0x0002;       // View is disabled
pub const SF_SHADOW: u16 = 0x0004;         // Draw shadow
pub const SF_MODAL: u16 = 0x0008;          // Modal window
pub const SF_HIDDEN: u16 = 0x0010;         // Hidden (don't draw)

pub const OF_PRE_PROCESS: u16 = 0x0100;    // Process events first
pub const OF_POST_PROCESS: u16 = 0x0200;   // Process events last
```

### Focus Management

```rust
// Focus is distributed through the view hierarchy
impl View for Group {
    fn set_focus_to(&mut self, index: usize) {
        if index < self.children.len() {
            self.focused = index;
            self.children[index].set_focus(true);  // Notify child
        }
    }
}

// Tab key moves focus
if event.what == EventType::Keyboard {
    if event.key_code == KB_TAB {
        self.move_focus_forward();
    } else if event.key_code == KB_SHIFT_TAB {
        self.move_focus_backward();
    }
}
```

---

## Complete Example: Menu + Status + Dialogs

From `examples/menu.rs`:

```rust
use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_QUIT, CM_NEW, CM_OK};
use turbo_vision::core::geometry::{Rect, Point};
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
use turbo_vision::views::status_line::{StatusLine, StatusItem};
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::button::Button;
use turbo_vision::views::static_text::StaticText;

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();
    
    // Create menu bar
    let mut menu_bar = MenuBar::new(Rect::new(0, 0, width as i16, 1));
    let file_menu_items = vec![
        MenuItem::with_shortcut("~N~ew", CM_NEW, 0, "Ctrl+N", 0),
        MenuItem::with_shortcut("E~x~it", CM_QUIT, 0, "Alt+X", 0),
    ];
    menu_bar.add_submenu(SubMenu::new("~F~ile", Menu::from_items(file_menu_items)));
    app.set_menu_bar(menu_bar);
    
    // Create status line
    let status_line = StatusLine::new(
        Rect::new(0, height as i16 - 1, width as i16, height as i16),
        vec![StatusItem::new("~F10~ Menu", KB_F10, CM_QUIT)],
    );
    app.set_status_line(status_line);
    
    // Main event loop
    app.running = true;
    while app.running {
        app.desktop.draw(&mut app.terminal);
        if let Some(ref mut menu_bar) = app.menu_bar {
            menu_bar.draw(&mut app.terminal);
        }
        if let Some(ref mut status_line) = app.status_line {
            status_line.draw(&mut app.terminal);
        }
        let _ = app.terminal.flush();
        
        if let Ok(Some(mut event)) = app.terminal.poll_event(
            std::time::Duration::from_millis(50)
        ) {
            app.handle_event(&mut event);
            
            if event.what == EventType::Command {
                match event.command {
                    CM_QUIT => {
                        app.running = false;
                    }
                    CM_NEW => {
                        show_message(&mut app, "New");
                    }
                    _ => {}
                }
            }
        }
    }
    
    Ok(())
}

fn show_message(app: &mut Application, title: &str) {
    let (term_width, term_height) = app.terminal.size();
    let dialog_width = 40;
    let dialog_height = 7;
    
    let dialog_x = (term_width as i16 - dialog_width) / 2;
    let dialog_y = (term_height as i16 - dialog_height) / 2;
    
    let mut dialog = Dialog::new(
        Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height),
        title
    );
    
    let text = StaticText::new_centered(
        Rect::new(2, 1, dialog_width - 4, 2),
        "Command selected!"
    );
    dialog.add(Box::new(text));
    
    let button = Button::new(
        Rect::new(15, 3, 25, 5),
        "  ~O~K  ",
        CM_OK,
        true
    );
    dialog.add(Box::new(button));
    dialog.set_initial_focus();
    
    dialog.execute(app);
}
```

---

## Summary of Key Differences from Pascal

| Aspect | Borland | Rust |
|--------|---------|------|
| **Parent Pointer** | Direct `owner` pointer | Event modification + call stack |
| **Event Handling** | `handleEvent(event)` method | `handle_event(&mut event)` method |
| **Init Methods** | `initMenuBar()`, `initStatusLine()` | Set via `app.set_menu_bar()`, `app.set_status_line()` |
| **Dialog Result** | Return value from `execute()` | Dialog returns `CommandId` |
| **Command Set** | Global `TView::curCommandSet` | Thread-local `GLOBAL_COMMAND_SET` |
| **Button Auto-update** | Receives `cmCommandSetChanged` broadcast | Same pattern, auto-broadcasts in idle() |
| **Modal Loop** | `Group::execute()` with `endState` | Same pattern implemented in Rust |

