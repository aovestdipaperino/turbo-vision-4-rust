# Chapter 7 — Turbo Vision Architecture Overview (Rust Edition)

**Version:** 0.2.11
**Updated:** 2025-11-04

This chapter provides a comprehensive overview of Turbo Vision's architecture, explaining how the Rust implementation adapts the original Borland Pascal design to use modern Rust patterns. Understanding this architecture is essential for building sophisticated Turbo Vision applications.

**Prerequisites:** Basic Rust knowledge (traits, ownership, lifetimes) and completion of Chapters 1-6.

**See Also:** `docs/TURBOVISION-DESIGN.md` for detailed implementation notes.

---

## Table of Contents

1. [Pascal vs. Rust: Fundamental Differences](#pascal-vs-rust-fundamental-differences)
2. [The View Hierarchy](#the-view-hierarchy)
3. [Core Architectural Concepts](#core-architectural-concepts)
4. [View Types](#view-types)
5. [Group Views](#group-views)
6. [Non-View Objects](#non-view-objects)
7. [Coordinate Systems](#coordinate-systems)
8. [State Flags and Options](#state-flags-and-options)
9. [Event System](#event-system)

---

## Pascal vs. Rust: Fundamental Differences

### The Pascal Approach: Class Inheritance

Borland's Turbo Vision used Object Pascal's inheritance:

```pascal
type
  TView = object(TObject)
    Bounds: TRect;
    Owner: PGroup;
    procedure Draw; virtual;
    procedure HandleEvent(var Event: TEvent); virtual;
  end;

  TWindow = object(TGroup)    // Inherits from TGroup -> TView -> TObject
    Title: String;
    Frame: PFrame;
    procedure Draw; virtual;  // Overrides TView.Draw
  end;
```

**Key Pascal Features:**
- Single inheritance hierarchy
- Virtual methods for polymorphism
- Raw pointers for parent references (`Owner: PGroup`)
- Manual memory management

### The Rust Approach: Traits and Composition

Rust achieves the same goals differently:

```rust
// View trait - all UI components implement this
pub trait View {
    fn bounds(&self) -> Rect;
    fn draw(&mut self, terminal: &mut Terminal);
    fn handle_event(&mut self, event: &mut Event);
    fn state(&self) -> StateFlags;
    // ... more methods
}

// Window uses composition, not inheritance
pub struct Window {
    bounds: Rect,
    frame: Frame,           // Owns a Frame
    interior: Group,        // Owns a Group (for children)
    state: StateFlags,
    title: String,
}

impl View for Window {
    fn draw(&mut self, terminal: &mut Terminal) {
        // Window-specific drawing
        self.frame.draw(terminal);
        self.interior.draw(terminal);
    }
    // ... implement other View methods
}
```

**Key Rust Features:**
- Trait-based polymorphism (no inheritance)
- Composition over inheritance
- Ownership system (no raw pointers!)
- Automatic memory management with `Drop`
- `Box<dyn View>` for dynamic dispatch

### Comparison Table

| Concept | Pascal | Rust |
|---------|--------|------|
| **Polymorphism** | Virtual methods | Traits (`dyn View`) |
| **Code Reuse** | Inheritance | Composition + traits |
| **Parent Reference** | Raw pointer (`Owner`) | Event propagation |
| **Memory Management** | Manual (`New`, `Dispose`) | Automatic (`Box`, `Drop`) |
| **Null Safety** | Nil pointers | `Option<T>` |
| **Type Safety** | Runtime type checks | Compile-time type checks |

---

## The View Hierarchy

### Conceptual Hierarchy

While Rust doesn't have inheritance, the **conceptual** organization remains:

```
View (trait)
├── Frame
├── Button
├── Label
├── StaticText
├── InputLine
├── Checkbox
├── RadioButton
├── Cluster
├── ScrollBar
├── Scroller
├── Editor
├── Group (contains Box<dyn View>)
│   ├── Window
│   │   └── Dialog
│   ├── Desktop
│   └── MenuBar
├── StatusLine
└── ... more views
```

### Rust Implementation

In Rust, this is achieved through:

1. **View Trait** - Common interface for all UI components
2. **Composition** - Views contain other views
3. **Box<dyn View>** - Dynamic dispatch for collections

```rust
// All views implement the View trait
impl View for Button { /* ... */ }
impl View for InputLine { /* ... */ }
impl View for Window { /* ... */ }

// Groups contain any type of view
pub struct Group {
    children: Vec<Box<dyn View>>,
}

impl Group {
    pub fn add(&mut self, child: Box<dyn View>) {
        self.children.push(child);
    }
}
```

---

## Core Architectural Concepts

### 1. Everything is a View

In Turbo Vision, **everything you see is a View**:

- Windows are views
- Buttons are views
- Input fields are views
- Even the entire application desktop is a view

```rust
pub trait View {
    // Core methods all views must implement
    fn bounds(&self) -> Rect;           // Position and size
    fn set_bounds(&mut self, bounds: Rect);
    fn draw(&mut self, terminal: &mut Terminal);  // Render
    fn handle_event(&mut self, event: &mut Event); // Process input

    // Optional methods with default implementations
    fn can_focus(&self) -> bool { false }
    fn state(&self) -> StateFlags { 0 }
    fn options(&self) -> u16 { 0 }
}
```

### 2. Views Are Rectangular

Every view occupies a rectangular area defined by `Rect`:

```rust
pub struct Rect {
    pub a: Point,  // Top-left corner
    pub b: Point,  // Bottom-right corner (exclusive)
}

// Example: A button at position (10, 5) with size 20x3
let button_bounds = Rect::new(10, 5, 30, 8);
//                             x1  y1  x2  y2
```

### 3. Composition Over Inheritance

Instead of inheriting, views **compose** smaller views:

```rust
pub struct Window {
    frame: Frame,         // Border with title
    interior: Group,      // Children container
    // ... other fields
}

impl Window {
    pub fn add(&mut self, child: Box<dyn View>) {
        self.interior.add(child);  // Delegate to interior
    }
}
```

### 4. Ownership and Lifetimes

Rust's ownership prevents the pointer bugs common in Pascal:

```rust
// Pascal: Raw pointer to parent (can dangle!)
type
  TView = object
    Owner: PGroup;  // Might become invalid!
  end;

// Rust: No parent pointer! Events propagate up call stack
impl View for Button {
    fn handle_event(&mut self, event: &mut Event) {
        if clicked {
            // Transform event to notify parent
            *event = Event::command(self.command_id);
            // Event "bubbles up" through the call stack
        }
    }
}
```

---

## View Types

### Simple Views

Views that display content but contain no children:

#### StaticText

Displays read-only text:

```rust
let text = StaticText::new(
    Rect::new(2, 2, 40, 4),
    "Welcome to Turbo Vision!\nPress any key to continue..."
);
```

#### Label

Like StaticText, but with a shortcut and associated control:

```rust
let label = Label::new(
    Rect::new(2, 5, 15, 6),
    "~N~ame:"  // Alt+N activates linked control
);
```

#### Button

Clickable button that generates commands:

```rust
let button = Button::new(
    Rect::new(10, 10, 20, 12),
    "~O~K",
    CM_OK,
    true  // is default button
);
```

#### InputLine

Single-line text input:

```rust
let name = Rc::new(RefCell::new(String::new()));
let input = InputLine::new(
    Rect::new(2, 2, 30, 3),
    50,  // max length
    name.clone()
);
```

#### Checkbox

Boolean toggle:

```rust
let enabled = Rc::new(RefCell::new(false));
let checkbox = Checkbox::new(
    Rect::new(2, 5, 20, 6),
    "~E~nabled",
    enabled.clone()
);
```

#### RadioButton

Mutually exclusive selection:

```rust
let payment = Rc::new(RefCell::new(0u16));

let cash = RadioButton::new(bounds1, "~C~ash", 0, payment.clone());
let check = RadioButton::new(bounds2, "C~h~eck", 1, payment.clone());
let card = RadioButton::new(bounds3, "~C~ard", 2, payment.clone());
```

### Scrolling Views

Views with scrollable content:

#### Scroller

Base for scrollable views:

```rust
pub struct Scroller {
    bounds: Rect,
    delta: Point,      // Scroll position
    limit: Point,      // Maximum scroll
    // ...
}
```

#### Editor

Full-featured text editor (extends Scroller):

```rust
let editor = Editor::new(bounds)
    .with_scrollbars_and_indicator();
editor.set_text("Hello, world!");
```

### Specialized Views

#### Frame

Window border with title and controls:

```rust
pub struct Frame {
    bounds: Rect,
    title: String,
    // Automatically draws resize handles, close box, etc.
}
```

#### StatusLine

Bottom-of-screen status bar:

```rust
let status_line = StatusLine::new(
    Rect::new(0, height - 1, width, height),
    vec![
        StatusItem::new("~F1~ Help", KB_F1, CM_HELP),
        StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
    ]
);
```

---

## Group Views

Groups are views that **contain and manage other views**.

### Group

Base container for views:

```rust
pub struct Group {
    bounds: Rect,
    children: Vec<Box<dyn View>>,
    focused_index: Option<usize>,
    background: Background,
}

impl Group {
    pub fn add(&mut self, child: Box<dyn View>) {
        self.children.push(child);
    }

    pub fn handle_event(&mut self, event: &mut Event) {
        // Three-phase event processing

        // Phase 1: PreProcess views (like StatusLine)
        for child in &mut self.children {
            if child.options() & OF_PRE_PROCESS != 0 {
                child.handle_event(event);
            }
        }

        // Phase 2: Focused view
        if let Some(idx) = self.focused_index {
            self.children[idx].handle_event(event);
        }

        // Phase 3: PostProcess views (like Buttons with hotkeys)
        for child in &mut self.children {
            if child.options() & OF_POST_PROCESS != 0 {
                child.handle_event(event);
            }
        }
    }
}
```

### Window

Bordered, movable, resizable container:

```rust
pub struct Window {
    bounds: Rect,
    frame: Frame,           // Border with title
    interior: Group,        // Child views
    state: StateFlags,      // Visible, focused, etc.
    title: String,
    // ...
}

impl Window {
    pub fn new(bounds: Rect, title: &str) -> Self {
        let frame = Frame::new(bounds, title);

        // Interior is inset by 1 for the frame
        let mut interior_bounds = bounds;
        interior_bounds.grow(-1, -1);

        Self {
            bounds,
            frame,
            interior: Group::with_background(interior_bounds, colors::DIALOG_NORMAL),
            // ...
        }
    }

    pub fn add(&mut self, child: Box<dyn View>) {
        self.interior.add(child);
    }
}
```

### Dialog

Specialized window for data entry:

```rust
pub struct Dialog {
    window: Window,  // Composes a Window
}

impl Dialog {
    pub fn new(bounds: Rect, title: &str) -> Self {
        Self {
            window: Window::new(bounds, title),
        }
    }

    pub fn execute(&mut self, app: &mut Application) -> u16 {
        // Make modal and run until closed
        // Returns the command that closed the dialog
    }
}
```

### Desktop

The main application workspace:

```rust
pub struct Desktop {
    group: Group,
}

impl Desktop {
    pub fn add(&mut self, window: Box<dyn View>) {
        self.group.add(window);
    }

    pub fn tile(&mut self) {
        // Arrange windows in tiles
    }

    pub fn cascade(&mut self) {
        // Arrange windows in cascade
    }
}
```

### Application

The root container for everything:

```rust
pub struct Application {
    pub terminal: Terminal,
    pub desktop: Desktop,
    pub menu_bar: Option<MenuBar>,
    pub status_line: Option<StatusLine>,
    pub running: bool,
}
```

---

## Non-View Objects

Objects that provide functionality but aren't visible:

### Validators

Validate input line data:

```rust
pub trait Validator {
    fn valid(&self, input: &str) -> bool;
    fn error(&self) -> &str;
}

// Range validator: 1-100
let validator = RangeValidator::new(1, 100);

// Picture validator: phone number
let validator = PictureValidator::new("(###) ###-####");

// Filter validator: digits only
let validator = FilterValidator::new("0123456789");
```

### Primitives

Basic geometric types:

```rust
pub struct Point {
    pub x: i16,
    pub y: i16,
}

pub struct Rect {
    pub a: Point,
    pub b: Point,
}

impl Rect {
    pub fn width(&self) -> i16 { self.b.x - self.a.x }
    pub fn height(&self) -> i16 { self.b.y - self.a.y }
    pub fn contains(&self, p: Point) -> bool { /* ... */ }
    pub fn grow(&mut self, dx: i16, dy: i16) { /* ... */ }
}
```

---

## Coordinate Systems

### Grid-Based Coordinates

Turbo Vision uses a **grid-based coordinate system** where coordinates specify positions *between* characters, not character cells:

```
  0   1   2   3   4   5
0 +---+---+---+---+---+
  | A | B | C | D | E |
1 +---+---+---+---+---+
  | F | G | H | I | J |
2 +---+---+---+---+---+
```

- Point (0, 0) is the top-left corner
- Point (1, 0) is between 'A' and 'B'
- `Rect::new(0, 0, 1, 1)` is the smallest rectangle containing one character ('A')
- `Rect::new(1, 0, 4, 1)` contains 3 characters ('B', 'C', 'D')

### Local vs. Global Coordinates

**Global Coordinates:**
- Origin at screen top-left (0, 0)
- Used for mouse events
- Absolute screen positions

**Local Coordinates:**
- Origin at view's top-left
- Used for child view positioning
- Relative to parent

```rust
// Global: position on screen
let window_bounds = Rect::new(10, 5, 50, 20);  // Screen coords

// Local: position within window
let button_bounds = Rect::new(5, 2, 15, 4);   // Relative to window

// When window moves, button moves with it (local coords)
// When mouse clicks, convert global to local:
fn handle_mouse(&self, global_pos: Point) -> Point {
    Point {
        x: global_pos.x - self.bounds.a.x,
        y: global_pos.y - self.bounds.a.y,
    }
}
```

### Coordinate Examples

```rust
// Empty rectangle (just a point)
Rect::new(0, 0, 0, 0)

// Single character
Rect::new(0, 0, 1, 1)

// 10 characters wide, 1 high
Rect::new(0, 0, 10, 1)

// Button centered on screen (40 wide, 3 high)
let center_x = width / 2;
let center_y = height / 2;
Rect::new(center_x - 20, center_y - 1, center_x + 20, center_y + 2)
```

---

## State Flags and Options

Views use bit flags for state and configuration.

### StateFlags

Track view state (visible, focused, modal, etc.):

```rust
pub type StateFlags = u16;

pub const SF_VISIBLE: StateFlags    = 0x0001;  // View is visible
pub const SF_FOCUSED: StateFlags    = 0x0002;  // View has input focus
pub const SF_MODAL: StateFlags      = 0x0004;  // View is modal
pub const SF_DISABLED: StateFlags   = 0x0008;  // View is disabled
pub const SF_SHADOW: StateFlags     = 0x0010;  // View has shadow
pub const SF_DRAGGING: StateFlags   = 0x0020;  // Being dragged
// ...
```

### Option Flags

Configure view behavior:

```rust
pub const OF_SELECTABLE: u16    = 0x0001;  // Can receive focus
pub const OF_PRE_PROCESS: u16   = 0x0002;  // Handle events first
pub const OF_POST_PROCESS: u16  = 0x0004;  // Handle events last
pub const OF_CENTERED: u16      = 0x0008;  // Center on screen
pub const OF_TOP_SELECT: u16    = 0x0010;  // Select on mouse down
// ...
```

### Bitwise Operations in Rust

```rust
// Set a flag
view.state |= SF_VISIBLE;

// Clear a flag
view.state &= !SF_FOCUSED;

// Toggle a flag
view.state ^= SF_MODAL;

// Check if flag is set
if view.state & SF_VISIBLE != 0 {
    // View is visible
}

// Check multiple flags
if view.state & (SF_VISIBLE | SF_FOCUSED) == (SF_VISIBLE | SF_FOCUSED) {
    // View is both visible AND focused
}

// Set multiple flags
view.state |= SF_VISIBLE | SF_FOCUSED | SF_MODAL;
```

### Helper Methods

The View trait provides helpers:

```rust
pub trait View {
    fn set_state_flag(&mut self, flag: StateFlags, enable: bool) {
        let current = self.state();
        if enable {
            self.set_state(current | flag);
        } else {
            self.set_state(current & !flag);
        }
    }

    fn get_state_flag(&self, flag: StateFlags) -> bool {
        (self.state() & flag) == flag
    }
}

// Usage
view.set_state_flag(SF_VISIBLE, true);   // Show
view.set_state_flag(SF_VISIBLE, false);  // Hide

if view.get_state_flag(SF_FOCUSED) {
    // Handle focused state
}
```

---

## Event System

### Event Flow

Events flow through views in three phases (matching Borland's TGroup::handleEvent):

```
Terminal generates event
    ↓
Application receives event
    ↓
┌─────────────────────────────────────┐
│ Phase 1: PreProcess                 │
│ - StatusLine (OF_PRE_PROCESS)       │
│ - Intercepts shortcuts first        │
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│ Special: MenuBar                    │
│ - Handles F10, Alt keys             │
│ - Check cascading submenus          │
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│ Phase 2: Focused                    │
│ - Desktop and focused children      │
│ - Normal event processing           │
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│ Phase 3: PostProcess                │
│ - Buttons (OF_POST_PROCESS)         │
│ - Handle hotkeys even when unfocused│
└─────────────────────────────────────┘
    ↓
Application handles unprocessed events
```

### Event Types

```rust
pub enum EventType {
    Nothing,        // No event or consumed
    Keyboard,       // Key press
    MouseDown,      // Mouse button pressed
    MouseUp,        // Mouse button released
    MouseMove,      // Mouse moved
    MouseDrag,      // Mouse dragged with button down
    Command,        // High-level command
    Broadcast,      // Message to all views
}

pub struct Event {
    pub what: EventType,
    pub key_code: u16,
    pub command: u16,
    pub mouse: MouseEvent,
    // ...
}
```

### Event Handling Pattern

```rust
impl View for MyView {
    fn handle_event(&mut self, event: &mut Event) {
        match event.what {
            EventType::Keyboard => {
                if event.key_code == KB_ENTER {
                    // Handle Enter key
                    event.clear();  // Mark as consumed
                }
            }
            EventType::MouseDown => {
                if self.bounds().contains(event.mouse.position) {
                    // Handle click
                    *event = Event::command(self.command_id);
                }
            }
            EventType::Command => {
                if event.command == MY_COMMAND {
                    // Handle command
                    event.clear();
                }
            }
            _ => {}
        }
    }
}
```

### Parent Communication

**Pascal approach (raw pointers):**
```pascal
message(Owner, evBroadcast, cmMyCommand, @Self);
```

**Rust approach (event transformation):**
```rust
// Child transforms event to notify parent
*event = Event::command(CM_MY_COMMAND);
// Event "bubbles up" through call stack to parent
```

---

## Summary

### Key Architectural Principles

1. **Everything is a View** - Common interface via trait
2. **Composition > Inheritance** - Build complex views from simple ones
3. **Ownership, not Pointers** - Rust's safety guarantees
4. **Events flow up** - Through three-phase processing
5. **Rectangles everywhere** - Grid-based coordinates
6. **Flags for state** - Bitwise operations for configuration

### Pascal → Rust Translation Guide

| Pascal Concept | Rust Equivalent |
|----------------|-----------------|
| `TView` inheritance | `View` trait |
| `Owner: PGroup` pointer | Event propagation |
| `New(PWindow, Init(...))` | `Box::new(Window::new(...))` |
| `Desktop^.Insert(window)` | `desktop.add(Box::new(window))` |
| `with Button^ do ...` | `button.method()` |
| Virtual methods | Trait methods |
| `ofCentered` flag | `OF_CENTERED` const |
| `or` / `and not` | `\|` / `& !` |

### Next Steps

Now that you understand the architecture:

1. **Read `docs/TURBOVISION-DESIGN.md`** - Detailed implementation notes
2. **Study the examples** - See patterns in practice
3. **Build your own views** - Implement the View trait
4. **Explore the source** - `src/views/` directory

---

## See Also

- **docs/TURBOVISION-DESIGN.md** - Complete architecture documentation
- **Chapter 1** - Getting started with basic applications
- **Chapter 3** - Working with windows
- **Chapter 5** - Data collections
- **Chapter 6** - Data entry forms
- **src/views/view.rs** - View trait definition
- **src/views/group.rs** - Group implementation
- **examples/** - All 16+ working examples

---

## Conclusion

The Rust implementation of Turbo Vision successfully adapts the original Pascal design to modern Rust idioms. While the implementation techniques differ (traits vs. inheritance, ownership vs. pointers), the **conceptual model** remains the same:

- Views are rectangular UI components
- Groups contain and manage child views
- Events flow through a three-phase system
- Everything composes to build rich terminal UIs

Understanding this architecture enables you to build sophisticated, maintainable Turbo Vision applications in Rust.
