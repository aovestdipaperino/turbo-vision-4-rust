# Chapter 8 — Views and Groups (Rust Edition)

**Version:** 0.2.11
**Updated:** 2025-11-04

This chapter explores the fundamental building blocks of Turbo Vision: **Views** and **Groups**. Understanding these concepts is essential for creating sophisticated terminal applications.

**Prerequisites:** Chapters 1-7, especially Chapter 7 (Architecture Overview)

---

## Table of Contents

1. [What is a View?](#what-is-a-view)
2. [What is a Group?](#what-is-a-group)
3. [View Properties](#view-properties)
4. [Drawing Views](#drawing-views)
5. [Group Management](#group-management)
6. [Modal Execution](#modal-execution)
7. [Advanced Topics](#advanced-topics)

---

## What is a View?

A **view** is any object that:
1. Occupies a rectangular region of the screen
2. Can draw itself on demand
3. Handles events within its boundaries

### The View Trait

In Rust, all views implement the `View` trait:

```rust
pub trait View {
    // Required: Position and size
    fn bounds(&self) -> Rect;
    fn set_bounds(&mut self, bounds: Rect);

    // Required: Rendering
    fn draw(&mut self, terminal: &mut Terminal);

    // Required: Event handling
    fn handle_event(&mut self, event: &mut Event);

    // Optional: Can this view receive focus?
    fn can_focus(&self) -> bool { false }

    // Optional: State management
    fn state(&self) -> StateFlags { 0 }
    fn set_state(&mut self, state: StateFlags) {}

    // Optional: Configuration
    fn options(&self) -> u16 { 0 }
    fn set_options(&mut self, options: u16) {}
}
```

### Three Core Responsibilities

#### 1. Managing a Rectangular Region

Every view has boundaries defined by a `Rect`:

```rust
pub struct Rect {
    pub a: Point,  // Top-left corner (inclusive)
    pub b: Point,  // Bottom-right corner (exclusive)
}

// Example: A button at position (5, 3) with size 20×3
let button_bounds = Rect::new(5, 3, 25, 6);
//                             x1  y1  x2  y2
```

#### 2. Drawing on Demand

Views must be able to draw themselves at any time:

```rust
impl View for Button {
    fn draw(&mut self, terminal: &mut Terminal) {
        let color = if self.is_focused() {
            colors::BUTTON_FOCUSED
        } else {
            colors::BUTTON_NORMAL
        };

        // Draw button background
        let width = self.bounds.width() as usize;
        let mut buf = DrawBuffer::new(width);
        buf.move_char(0, ' ', color, width);

        // Draw button text centered
        let text = &self.text;
        let start = (width - text.len()) / 2;
        buf.move_str_with_shortcut(start, text, color, colors::BUTTON_SHORTCUT);

        // Write to terminal
        write_line_to_terminal(
            terminal,
            self.bounds.a.x,
            self.bounds.a.y + 1,  // Vertically centered
            &buf
        );
    }
}
```

**Key Point:** Views must always be ready to redraw. Other views may cover them, then uncover them. The view must know its current state at all times.

#### 3. Handling Events

Views respond to user input within their boundaries:

```rust
impl View for Button {
    fn handle_event(&mut self, event: &mut Event) {
        match event.what {
            EventType::MouseDown => {
                if self.bounds().contains(event.mouse.position) {
                    // Button was clicked!
                    *event = Event::command(self.command_id);
                    event.clear();
                }
            }
            EventType::Keyboard => {
                // Check for shortcut key
                if event.key_code == self.shortcut_key {
                    *event = Event::command(self.command_id);
                    event.clear();
                }
            }
            _ => {}
        }
    }
}
```

---

## What is a Group?

A **group** is a view that contains and manages other views (called **subviews**).

### Groups in the Hierarchy

```
Group (contains Vec<Box<dyn View>>)
├── Desktop    - Main workspace
├── Window     - Bordered container
│   └── Dialog - Data entry window
├── MenuBar    - Top menu strip
└── Application - Root container
```

### The Group Structure

```rust
pub struct Group {
    bounds: Rect,
    children: Vec<Box<dyn View>>,      // Subviews
    focused_index: Option<usize>,       // Which child has focus
    background: Background,             // Background view
    state: StateFlags,
}

impl Group {
    pub fn add(&mut self, child: Box<dyn View>) {
        self.children.push(child);
    }

    pub fn remove(&mut self, index: usize) -> Option<Box<dyn View>> {
        if index < self.children.len() {
            Some(self.children.remove(index))
        } else {
            None
        }
    }
}
```

### Why Groups?

Groups allow **delegation** of responsibilities:

```rust
// Desktop is a group that owns windows
let mut desktop = Desktop::new(Rect::new(0, 1, 80, 24));

// Add windows to desktop
desktop.add(Box::new(window1));
desktop.add(Box::new(window2));

// When desktop draws, it tells each window to draw itself
desktop.draw(&mut terminal);
// Internally: for child in &mut self.children { child.draw(terminal); }
```

---

## View Properties

### Bounds and Positioning

#### Getting Bounds

```rust
// Get view's current bounds
let bounds = view.bounds();
println!("Position: ({}, {})", bounds.a.x, bounds.a.y);
println!("Size: {}×{}", bounds.width(), bounds.height());
```

#### Moving a View

```rust
// Move view to new position (changes origin, keeps size)
let mut new_bounds = view.bounds();
new_bounds.a.x = 10;
new_bounds.a.y = 5;
new_bounds.b.x = 10 + original_width;
new_bounds.b.y = 5 + original_height;
view.set_bounds(new_bounds);
```

#### Resizing a View

```rust
// Resize view (keeps origin, changes size)
let mut new_bounds = view.bounds();
new_bounds.b.x = new_bounds.a.x + new_width;
new_bounds.b.y = new_bounds.a.y + new_height;
view.set_bounds(new_bounds);
```

#### Move and Resize

```rust
// Set both position and size
view.set_bounds(Rect::new(10, 5, 50, 20));
```

### Coordinate Systems

**Local Coordinates:** Origin at view's top-left

```rust
// Button inside dialog at dialog-local coordinates (5, 2)
let button = Button::new(
    Rect::new(5, 2, 15, 4),  // Local to dialog
    "OK",
    CM_OK,
    true
);
dialog.add(Box::new(button));
```

**Global Coordinates:** Origin at screen top-left

```rust
// Convert global mouse position to local coordinates
fn to_local(&self, global_pos: Point) -> Point {
    Point {
        x: global_pos.x - self.bounds.a.x,
        y: global_pos.y - self.bounds.a.y,
    }
}
```

### State Flags

Views track their state with bit flags:

```rust
pub const SF_VISIBLE: StateFlags    = 0x0001;  // View is visible
pub const SF_FOCUSED: StateFlags    = 0x0002;  // Has input focus
pub const SF_MODAL: StateFlags      = 0x0004;  // Is modal
pub const SF_DISABLED: StateFlags   = 0x0008;  // Is disabled
pub const SF_SHADOW: StateFlags     = 0x0010;  // Has shadow
```

#### Checking State

```rust
// Check if view is visible
if view.state() & SF_VISIBLE != 0 {
    view.draw(&mut terminal);
}

// Helper method
if view.get_state_flag(SF_FOCUSED) {
    // View has focus
}
```

#### Setting State

```rust
// Show view
view.set_state_flag(SF_VISIBLE, true);

// Hide view
view.set_state_flag(SF_VISIBLE, false);

// Toggle focus
let focused = view.get_state_flag(SF_FOCUSED);
view.set_state_flag(SF_FOCUSED, !focused);
```

### Option Flags

Configure view behavior with option flags:

```rust
pub const OF_SELECTABLE: u16    = 0x0001;  // Can receive focus
pub const OF_PRE_PROCESS: u16   = 0x0002;  // Handle events first
pub const OF_POST_PROCESS: u16  = 0x0004;  // Handle events last
pub const OF_CENTERED: u16      = 0x0008;  // Center on screen
pub const OF_TOP_SELECT: u16    = 0x0010;  // Move to top when selected
```

#### Using Options

```rust
// Make view selectable
view.set_options(view.options() | OF_SELECTABLE);

// Make view handle events first (like StatusLine)
view.set_options(view.options() | OF_PRE_PROCESS);

// Center view on screen
view.set_options(view.options() | OF_CENTERED);
```

---

## Drawing Views

### The Draw Contract

Every view must:
1. Fill its entire rectangle
2. Be able to draw at any time
3. Not assume previous state persists

### Direct Drawing

Simple views can draw directly:

```rust
impl View for SimpleLabel {
    fn draw(&mut self, terminal: &mut Terminal) {
        let bounds = self.bounds();
        let color = colors::LABEL_NORMAL;

        // Draw text at position
        terminal.write_str(
            bounds.a.x,
            bounds.a.y,
            &self.text,
            color
        );
    }
}
```

### Buffer Drawing (Recommended)

For complex or multiline views, use `DrawBuffer`:

```rust
use turbo_vision::core::draw::DrawBuffer;

impl View for MyView {
    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width() as usize;
        let height = self.bounds.height() as usize;

        for row in 0..height {
            let mut buf = DrawBuffer::new(width);

            // Fill with background
            buf.move_char(0, ' ', colors::NORMAL, width);

            // Add text
            if row < self.lines.len() {
                buf.move_str(0, &self.lines[row], colors::NORMAL);
            }

            // Write buffer to terminal
            write_line_to_terminal(
                terminal,
                self.bounds.a.x,
                self.bounds.a.y + row as i16,
                &buf
            );
        }
    }
}
```

### Drawing with Shortcuts

Handle ~X~ shortcut syntax:

```rust
// "~O~K" displays as "OK" with 'O' highlighted
buf.move_str_with_shortcut(
    x,
    "~O~K",
    colors::BUTTON_NORMAL,
    colors::BUTTON_SHORTCUT
);
```

### Colors and Palettes

Use predefined color constants:

```rust
use turbo_vision::core::palette::colors;

// Common colors
colors::BUTTON_NORMAL      // Normal button
colors::BUTTON_FOCUSED     // Focused button
colors::BUTTON_SHORTCUT    // Shortcut letter
colors::DIALOG_NORMAL      // Dialog background
colors::DIALOG_FRAME       // Dialog frame
colors::INPUT_NORMAL       // Input field
colors::INPUT_SELECTED     // Selected input
```

---

## Group Management

### Adding Subviews

```rust
// Create group
let mut window = Window::new(Rect::new(5, 3, 75, 20), "My Window");

// Add subviews (ownership transfers to window)
window.add(Box::new(label));
window.add(Box::new(input));
window.add(Box::new(button));
```

### Z-Order

**Z-order** determines which views are "in front":

```
Last added (front) → First added (back)
   Button              Input              Label
     ↑                   ↑                  ↑
   Third             Second              First
```

```rust
window.add(Box::new(label));   // Back
window.add(Box::new(input));   // Middle
window.add(Box::new(button));  // Front
```

When views overlap, views added later cover views added earlier.

### Delegation Pattern

Groups delegate to children:

```rust
impl View for Group {
    fn draw(&mut self, terminal: &mut Terminal) {
        // Draw background
        self.background.draw(terminal);

        // Draw children in reverse Z-order (back to front)
        for child in self.children.iter_mut().rev() {
            if child.state() & SF_VISIBLE != 0 {
                child.draw(terminal);
            }
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        // Three-phase event processing

        // Phase 1: PreProcess views
        for child in &mut self.children {
            if child.options() & OF_PRE_PROCESS != 0 {
                child.handle_event(event);
                if event.what == EventType::Nothing {
                    return;  // Event consumed
                }
            }
        }

        // Phase 2: Focused view
        if let Some(idx) = self.focused_index {
            self.children[idx].handle_event(event);
            if event.what == EventType::Nothing {
                return;
            }
        }

        // Phase 3: PostProcess views
        for child in &mut self.children {
            if child.options() & OF_POST_PROCESS != 0 {
                child.handle_event(event);
                if event.what == EventType::Nothing {
                    return;
                }
            }
        }
    }
}
```

### Focus Management

Only one subview in a group can be focused:

```rust
impl Group {
    pub fn focus_next(&mut self) {
        // Find next selectable view
        if let Some(current) = self.focused_index {
            for i in 1..self.children.len() {
                let idx = (current + i) % self.children.len();
                if self.children[idx].options() & OF_SELECTABLE != 0 {
                    self.focused_index = Some(idx);
                    self.children[idx].set_focus(true);
                    if current != idx {
                        self.children[current].set_focus(false);
                    }
                    break;
                }
            }
        }
    }
}
```

### Tab Order

Tab order = insertion order:

```rust
dialog.add(Box::new(name_input));      // Tab 1
dialog.add(Box::new(email_input));     // Tab 2
dialog.add(Box::new(phone_input));     // Tab 3
dialog.add(Box::new(save_button));     // Tab 4
```

Pressing Tab cycles through views in this order.

---

## Modal Execution

### What is Modal?

A **modal view** blocks interaction with all other views:

```
┌─────────────────────────────────┐
│ Application (modal by default) │
│  ┌─────────────────────────┐   │
│  │ Desktop                  │   │
│  │  ┌─────────────────┐    │   │
│  │  │ Dialog (modal)  │    │   │
│  │  │  [OK] [Cancel]  │    │   │
│  │  └─────────────────┘    │   │
│  │                          │   │
│  │  Window (inactive)       │   │
│  └─────────────────────────┘   │
└─────────────────────────────────┘
```

When Dialog is modal:
- Only Dialog and its children respond to events
- Desktop and Window are inactive
- StatusLine remains active (special case)

### Executing Modal Dialogs

```rust
// Create dialog
let mut dialog = Dialog::new(Rect::new(20, 8, 60, 16), "Confirm");

// Add controls
dialog.add(Box::new(message));
dialog.add(Box::new(ok_button));
dialog.add(Box::new(cancel_button));

// Execute modally (blocks until closed)
let result = dialog.execute(&mut app);

// result is the command that closed the dialog
if result == CM_OK {
    // User clicked OK
} else {
    // User clicked Cancel or closed dialog
}
```

### Modal Event Loop

Inside `dialog.execute()`:

```rust
pub fn execute(&mut self, app: &mut Application) -> u16 {
    self.set_state_flag(SF_MODAL, true);

    let mut result = 0;
    let mut running = true;

    while running {
        // Draw
        self.draw(&mut app.terminal);
        app.terminal.flush();

        // Handle events
        if let Ok(Some(mut event)) = app.terminal.poll_event(timeout) {
            self.handle_event(&mut event);

            // Check for closing commands
            if event.what == EventType::Command {
                match event.command {
                    CM_OK | CM_CANCEL | CM_YES | CM_NO => {
                        result = event.command;
                        running = false;
                    }
                    _ => {}
                }
            }
        }
    }

    self.set_state_flag(SF_MODAL, false);
    result
}
```

---

## Advanced Topics

### Custom View Example

Creating a custom counter view:

```rust
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::event::Event;
use turbo_vision::core::draw::DrawBuffer;
use turbo_vision::core::palette::colors;
use turbo_vision::core::state::StateFlags;
use turbo_vision::terminal::Terminal;
use turbo_vision::views::view::{View, write_line_to_terminal};

pub struct CounterView {
    bounds: Rect,
    current: usize,
    total: usize,
    state: StateFlags,
}

impl CounterView {
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            current: 1,
            total: 0,
            state: 0,
        }
    }

    pub fn update(&mut self, current: usize, total: usize) {
        self.current = current;
        self.total = total;
    }
}

impl View for CounterView {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width() as usize;
        let mut buf = DrawBuffer::new(width);

        // Background
        buf.move_char(0, ' ', colors::FRAME_PASSIVE, width);

        // Text
        let text = if self.total > 0 {
            format!(" {} of {} ", self.current, self.total)
        } else {
            " No records ".to_string()
        };

        // Center text
        let start = if width > text.len() {
            (width - text.len()) / 2
        } else {
            0
        };

        buf.move_str(start, &text, colors::FRAME_PASSIVE);

        // Write to screen
        write_line_to_terminal(
            terminal,
            self.bounds.a.x,
            self.bounds.a.y,
            &buf
        );
    }

    fn handle_event(&mut self, _event: &mut Event) {
        // Counter doesn't handle events
    }

    fn state(&self) -> StateFlags {
        self.state
    }

    fn set_state(&mut self, state: StateFlags) {
        self.state = state;
    }
}
```

### Using the Custom View

```rust
// In window constructor
let counter = CounterView::new(Rect::new(20, 0, 40, 1));
window.add(Box::new(counter));

// Later, update the counter
// (Would need reference to counter - see Chapter 5 for data sharing patterns)
```

### Grow Modes

Control how views resize when their owner resizes:

```rust
pub const GF_GROW_LO_X: u16 = 0x01;  // Left side follows owner's left
pub const GF_GROW_LO_Y: u16 = 0x02;  // Top follows owner's top
pub const GF_GROW_HI_X: u16 = 0x04;  // Right follows owner's right
pub const GF_GROW_HI_Y: u16 = 0x08;  // Bottom follows owner's bottom
pub const GF_GROW_ALL: u16  = 0x0F;  // All sides follow

// Example: Window interior that resizes with window
interior.set_grow_mode(GF_GROW_ALL);

// Example: Button that stays in bottom-right corner
button.set_grow_mode(GF_GROW_HI_X | GF_GROW_HI_Y);
```

### Clipping

Views are automatically clipped to their owner's bounds:

```rust
// Window at (10, 5) to (70, 20)
let window = Window::new(Rect::new(10, 5, 70, 20), "Clipped");

// Child extends beyond window bounds
let child = View::new(Rect::new(50, 0, 100, 10));  // Extends to x=100

// Only the part from x=50 to x=60 (window's right edge) will be visible
window.add(Box::new(child));
```

---

## Best Practices

### 1. Always Implement View Correctly

```rust
impl View for MyView {
    // Always implement all required methods
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, bounds: Rect) { self.bounds = bounds; }
    fn draw(&mut self, terminal: &mut Terminal) { /* fill entire bounds */ }
    fn handle_event(&mut self, event: &mut Event) { /* respond to events */ }
}
```

### 2. Use DrawBuffer for Complex Views

```rust
// Good - efficient, reduces flicker
let mut buf = DrawBuffer::new(width);
buf.move_char(0, ' ', color, width);
buf.move_str(pos, text, color);
write_line_to_terminal(terminal, x, y, &buf);

// Avoid - multiple writes cause flicker
terminal.write_char(x, y, ' ', color);
terminal.write_str(x+1, y, text, color);
```

### 3. Respect Ownership

```rust
// Good - transfer ownership to group
group.add(Box::new(view));

// After this, group owns view - don't try to use view
```

### 4. Handle Events Properly

```rust
fn handle_event(&mut self, event: &mut Event) {
    if /* this view handled the event */ {
        event.clear();  // Mark as consumed
    }
    // If not handled, leave event unchanged for parent
}
```

### 5. Maintain State Consistency

```rust
// Override set_state to respond to state changes
fn set_state_flag(&mut self, flag: StateFlags, enable: bool) {
    let old_state = self.state;

    // Update state
    if enable {
        self.state |= flag;
    } else {
        self.state &= !flag;
    }

    // Respond to changes
    if flag == SF_FOCUSED && old_state != self.state {
        // Redraw with new focus state
        self.needs_redraw = true;
    }
}
```

---

## Summary

### Key Concepts

- **View** - Rectangular screen region that draws itself and handles events
- **Group** - View that contains and manages other views
- **Subview** - View owned by a group
- **Z-order** - Front-to-back ordering of views
- **Modal** - View that blocks interaction with other views
- **Focus** - Currently active view that receives keyboard input

### The View Contract

Every view must:
1. Know its bounds
2. Draw its entire rectangle on demand
3. Handle events within its bounds
4. Not assume screen contents persist

### The Group Pattern

Groups:
1. Contain subviews in a Vec
2. Delegate drawing to children
3. Delegate events to children (three phases)
4. Manage focus among children
5. Handle Z-order automatically

---

## See Also

- **Chapter 7** - Architecture Overview
- **Chapter 9** - Event-Driven Programming (next)
- **docs/TURBOVISION-DESIGN.md** - Implementation details
- **src/views/view.rs** - View trait source
- **src/views/group.rs** - Group implementation
- **examples/window_resize_demo.rs** - Grow modes
- **examples/dialogs_demo.rs** - Modal execution

---

Understanding Views and Groups is fundamental to mastering Turbo Vision. These concepts apply throughout the framework, from simple buttons to complex applications.
