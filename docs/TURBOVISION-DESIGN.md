# Turbo Vision for Rust - Design Documentation

**Version:** 0.3.0
**Last Updated:** 2025-11-05
**Reference:** Borland Turbo Vision 2.0

---

## Table of Contents

1. [Class Hierarchy and Architecture](#class-hierarchy-and-architecture)
2. [Implementation Status](#implementation-status)
3. [Focus Architecture](#focus-architecture)
4. [Event System Architecture](#event-system-architecture)
5. [State Management](#state-management)
6. [Modal Dialog Execution](#modal-dialog-execution)
7. [Owner/Parent Communication](#ownerparent-communication)
8. [Syntax Highlighting System](#syntax-highlighting-system)
9. [Validation System](#validation-system)
10. [FileDialog Implementation](#filedialog-implementation)
11. [Screen Dump System](#screen-dump-system)
12. [Command Set System](#command-set-system)
13. [Architecture Comparisons](#architecture-comparisons)

---

# Class Hierarchy and Architecture

## Borland Turbo Vision Class Hierarchy

```
TObject (Base)
    │
    ├─> TView (Base view class)
    │    │
    │    ├─> TGroup (Container)
    │    │    │
    │    │    ├─> TWindow (Windowed container)
    │    │    │    │
    │    │    │    └─> TDialog (Modal dialog)
    │    │    │
    │    │    └─> TDeskTop (Desktop manager)
    │    │
    │    ├─> TFrame (Window border)
    │    │
    │    ├─> TScrollBar (Scrollbar widget)
    │    │
    │    ├─> TStaticText (Static label)
    │    │
    │    ├─> TButton (Push button)
    │    │
    │    ├─> TInputLine (Single-line input)
    │    │    │
    │    │    └─> TEditor (Multi-line editor - NOT inherited!)
    │    │
    │    ├─> TListViewer (Base for lists)
    │    │    │
    │    │    ├─> TListBox (Selection list)
    │    │    │
    │    │    └─> TFileList (File browser list)
    │    │
    │    ├─> TCluster (Button group base)
    │    │    │
    │    │    ├─> TCheckBoxes (Checkbox group)
    │    │    │
    │    │    └─> TRadioButtons (Radio button group)
    │    │
    │    ├─> TMenuBar (Menu bar)
    │    │
    │    ├─> TStatusLine (Status bar)
    │    │
    │    └─> TIndicator (Position indicator)
    │
    └─> TApplication (Main app class)
```

## Rust Turbo Vision Architecture

**Key Difference: Composition over Inheritance**

```
┌─────────────────────────────────────────────┐
│            View Trait                       │
│  (Base behavior - no data)                  │
│                                             │
│  fn bounds() -> Rect                        │
│  fn draw(&mut self, terminal)               │
│  fn handle_event(&mut self, event)          │
│  fn can_focus() -> bool                     │
│  fn set_focus(&mut self, focused)           │
│  fn state() -> StateFlags                   │
│  fn options() -> OptionsFlags               │
└─────────────────────────────────────────────┘
                    △
                    │ implements
        ┌───────────┴───────────┐
        │                       │
        │                       │
┌───────┴────────┐      ┌───────┴────────┐
│  Leaf Views    │      │  Container     │
│  (Components)  │      │  Views         │
│                │      │                │
│  • Button      │      │  • Group       │
│  • InputLine   │      │  • Window      │
│  • Label       │      │  • Dialog      │
│  • StaticText  │      │  • Desktop     │
│  • CheckBox    │      │  • Application │
│  • RadioButton │      └────────────────┘
│  • ScrollBar   │              │
│  • Indicator   │              │ contains
│  • Editor      │              │
│  • ListBox     │              ▼
│  • MenuBar     │      children: Vec<Box<dyn View>>
│  • StatusLine  │
└────────────────┘
```

## Borland vs Rust: Inheritance vs Composition

### Borland (C++ Inheritance)

```
TDialog (inherits TWindow)
   ├─> TWindow (inherits TGroup)
   │    ├─> TGroup (inherits TView)
   │    │    ├─> TView (base class)
   │    │    │    ├─ bounds: TRect
   │    │    │    ├─ state: ushort
   │    │    │    └─ owner: TGroup*
   │    │    │
   │    │    └─ children: TView*  (linked list)
   │    │
   │    └─ frame: TFrame*
   │
   └─ All inherited fields accessible directly
```

### Rust (Composition)

```
Dialog
   ├─> window: Window  (composed, not inherited!)
   │    ├─> group: Group
   │    │    ├─ bounds: Rect
   │    │    ├─ state: StateFlags
   │    │    └─ children: Vec<Box<dyn View>>
   │    │
   │    └─ frame: Frame
   │
   └─ Delegates View trait methods to window
```

## Key Architectural Patterns

### 1. Container Hierarchy (Borland → Rust)

```
Borland:                          Rust:
═══════                           ═════

TView                             View trait
  └─> TGroup                        └─> Group struct
        └─> TWindow                       └─> Window struct
              └─> TDialog                       └─> Dialog struct

Inheritance Chain                 Composition Chain
(is-a relationships)              (has-a relationships)
```

### 2. Event Flow (Both Systems)

```
           User Input
               │
               ▼
┌─────────────────────────────────────┐
│  Terminal                           │
│  (Captures keyboard/mouse)          │
└──────────────┬──────────────────────┘
               │ Event
               ▼
┌─────────────────────────────────────┐
│  Application                        │
│  (Main event loop)                  │
└──────────────┬──────────────────────┘
               │ Event
               ▼
┌─────────────────────────────────────┐
│  Desktop                            │
│  (Z-order, modal scope)             │
└──────────────┬──────────────────────┘
               │ Event
               ▼
┌─────────────────────────────────────┐
│  Dialog/Window                      │
│  (Container)                        │
└──────────────┬──────────────────────┘
               │ Event
               ▼
┌─────────────────────────────────────┐
│  Group (Three-Phase Processing)     │
│                                     │
│  Phase 1: PreProcess                │
│    └─> StatusLine (OF_PRE_PROCESS)  │
│                                     │
│  Phase 2: Focused View              │
│    └─> Button, InputLine, Editor    │
│                                     │
│  Phase 3: PostProcess               │
│    └─> Button (OF_POST_PROCESS)     │
└─────────────────────────────────────┘
```

### 3. Parent Communication Patterns

```
Borland (Raw Pointers):              Rust (Event Transformation):
═══════════════════════              ═══════════════════════════

Button                               Button
  ├─> owner: TDialog*                  ├─> command: CommandId
  │                                    │
  └─> press()                          └─> handle_event(&mut event)
        │                                    │
        └─> message(owner,                  └─> *event = Event::command(cmd)
                evBroadcast,                      │
                command,                          │ (bubbles up)
                this);                            │
              │                                   ▼
              │                              Dialog receives
              ▼                              transformed event
        Dialog receives
        via pointer call

Unsafe but flexible                  Safe and idiomatic
Circular references                  Call stack unwinding
```

## Syntax Highlighting Architecture

### Editor with Syntax Highlighting Integration

```
┌──────────────────────────────────────────────────────────────┐
│                         Editor                               │
│                                                              │
│  Fields:                                                     │
│  ├─ lines: Vec<String>         (text content)                │
│  ├─ cursor: Point              (cursor position)             │
│  ├─ selection: Option<Selection>  (text selection)           │
│  ├─ undo_stack: Vec<Action>    (undo/redo)                   │
│  ├─ highlighter: Option<Box<dyn SyntaxHighlighter>>  ◄──┐    │
│  ├─ scrollbar_v: Option<ScrollBar>                      │    │
│  ├─ scrollbar_h: Option<ScrollBar>                      │    │
│  └─ indicator: Option<Indicator>                        │    │
│                                                         │    │
│  Methods:                                               │    │
│  ├─ set_highlighter(h: Box<dyn SyntaxHighlighter>)   ◄──┤    │
│  ├─ clear_highlighter()                                 │    │
│  ├─ has_highlighter() -> bool                           │    │
│  └─ draw(&mut self, terminal)  ◄─────────────────────┐  │    │
│         │                                            │  │    │
│         └─> if let Some(ref highlighter) = self.highlighter  │   
│                 │                                    │  │    │
│                 └─> tokens = highlighter.highlight_line(line)│
│                         │                            │  │    │
│                         └─> for token in tokens      │  │    │
│                                  │                   │  │    │
│                                  └─> draw with color │  │    │
└──────────────────────────────────────────────────────┼──┼────┘
                                                       │  │
                                                       │  │
┌──────────────────────────────────────────────────────┼──┼─────┐
│         SyntaxHighlighter Trait                      │  │     │
│                                                      │  │     │
│  fn language(&self) -> &str                          │  │     │
│  fn highlight_line(&self, line: &str, line_num) -> Vec<Token> │
│  fn is_multiline_context(&self, line_num) -> bool    │  │     │
│  fn update_multiline_state(&mut self, line, ...)     │  │     │
└──────────────────────────────────────────────────────┼──┼─────┘
                        △                              │  │
                        │ implements                   │  │
        ┌───────────────┴───────────────┐              │  │
        │                               │              │  │
┌───────┴────────────┐        ┌─────────┴───────────┐  │  │
│  RustHighlighter   │        │ PlainTextHighlighter│  │  │
│                    │        │                     │  │  │
│  • Keywords        │        │  • No coloring      │  │  │
│  • Strings         │        │  • Default color    │  │  │
│  • Comments        │        │  • Minimal overhead │  │  │
│  • Numbers         │        └─────────────────────┘  │  │
│  • Types           │                                 │  │
│  • Functions       │                                 │  │
│  • Operators       │         ┌───────────────────────┘  │
└────────────────────┘         │                          │
                               │                          │
                    ┌──────────▼──────────────────────────▼───┐
                    │         Token Structure                 │
                    │                                         │
                    │  start: usize    (character position)   │
                    │  end: usize      (character position)   │
                    │  token_type: TokenType                  │
                    │      │                                  │
                    │      └─> default_color() -> Attr        │
                    │            │                            │
                    │            └─> Yellow    (Keyword)      │
                    │                LightRed  (String)       │
                    │                LightCyan (Comment)      │
                    │                Cyan      (Function)     │
                    │                ...                      │
                    └─────────────────────────────────────────┘
```

### Syntax Highlighting Flow

```
1. Editor::draw() called
       │
       ▼
2. For each visible line:
       │
       ▼
3. Check if highlighter is set?
       │
       ├─ YES ─> 4. Call highlighter.highlight_line(line, line_num)
       │             │
       │             ▼
       │         5. Highlighter returns Vec<Token>
       │             │
       │             ▼
       │         6. For each token:
       │             ├─> Extract token text from line
       │             ├─> Get token color: token.token_type.default_color()
       │             └─> buf.move_str(pos, text, color)
       │
       └─ NO ──> 7. Use default color for entire line
                     │
                     └─> buf.move_str(0, line, default_color)
```

### Example: RustHighlighter Processing

```
Input Line:
  "fn main() {"

RustHighlighter.highlight_line():
  │
  ▼
Tokens Generated:
  ┌─────────────────────────────────────┐
  │ Token 1:                            │
  │   start: 0,  end: 2                 │
  │   token_type: Keyword               │
  │   text: "fn"                        │
  │   color: Yellow                     │
  ├─────────────────────────────────────┤
  │ Token 2:                            │
  │   start: 3,  end: 7                 │
  │   token_type: Function              │
  │   text: "main"                      │
  │   color: Cyan                       │
  ├─────────────────────────────────────┤
  │ Token 3:                            │
  │   start: 7,  end: 8                 │
  │   token_type: Operator              │
  │   text: "("                         │
  │   color: White                      │
  ├─────────────────────────────────────┤
  │ Token 4:                            │
  │   start: 8,  end: 9                 │
  │   token_type: Operator              │
  │   text: ")"                         │
  │   color: White                      │
  ├─────────────────────────────────────┤
  │ Token 5:                            │
  │   start: 10, end: 11                │
  │   token_type: Operator              │
  │   text: "{"                         │
  │   color: White                      │
  └─────────────────────────────────────┘
       │
       ▼
  Rendered as:
  [Yellow]fn[White] [Cyan]main[White]()[White] {
```

## Component Ownership Model

### Borland (Manual Memory Management)

```
TDialog
   │
   │ owns via raw pointer
   ▼
TButton* button = new TButton(...);
dialog->insert(button);
   │
   │ dialog responsible for:
   │  - calling delete button
   │  - managing lifetime
   │  - handling dangling pointers
   │
   └─> delete this;  // Manual cleanup
```

### Rust (Automatic Memory Management)

```
Dialog
   │
   │ owns via Box<dyn View>
   ▼
let button = Box::new(Button::new(...));
dialog.add(button);
   │
   │ Dialog struct contains:
   │  window: Window {
   │    group: Group {
   │      children: Vec<Box<dyn View>>  ◄─── Ownership transfer
   │    }
   │  }
   │
   └─> } // Drop automatically cleans up entire tree
```

---

# Implementation Status

## Current Version: 0.2.6 (2025-11-03)

### Statistics
- **Total Tests**: 171 passing
- **Total Lines**: ~15,000
- **Components Implemented**: 55+
- **Phases Complete**: 9/11 (Phases 1-9)
- **Examples**: 16 (consolidated from 19)

### Major Features Complete

✅ **Core Architecture (Phase 1-3)**
- View trait system with event handling
- Group/Window/Dialog hierarchy
- Desktop and Application framework
- Terminal abstraction with crossterm backend

✅ **Event System (Phase 4)**
- Keyboard, Mouse, Command, Broadcast events
- Three-phase event processing (PreProcess, Focused, PostProcess)
- Event re-queuing via put_event()
- Owner-aware broadcast distribution

✅ **State Management (Phase 5)**
- Unified state flags (SF_VISIBLE, SF_FOCUSED, SF_DISABLED, SF_MODAL, etc.)
- Focus consolidation complete (all views use StateFlags)
- Command enable/disable system with global thread-local state

✅ **Basic Controls (Phase 6)**
- Button, Label, StaticText, InputLine, CheckBox, RadioButton
- Menu bar with dropdowns and keyboard navigation
- Status line with hot spots and hints

✅ **Advanced Controls (Phase 7)**
- Editor with undo/redo, search/replace, clipboard
- Syntax highlighting system (extensible, RustHighlighter built-in)
- Memo (multi-line text input)
- ScrollBar (horizontal and vertical)

✅ **List Infrastructure (Phase 8)**
- ListBox with keyboard/mouse navigation
- SortedListBox with binary search and type-ahead
- ListViewer base class
- Collection/StringCollection data management
- DirListBox and FileListBox for file browsing
- HistoryList with persistence

✅ **Validation System (Phase 8)**
- Validator trait for input validation
- FilterValidator (character filtering)
- RangeValidator (numeric ranges with hex/octal support)
- PictureValidator (Borland's TPXPictureValidator - format masks)
- LookupValidator (dropdown list validation)

✅ **Help System (Phase 9)**
- HelpFile with markdown support
- HelpWindow with topic navigation
- HelpIndex for keyword lookup
- Context-sensitive help framework

✅ **File System (Phase 9)**
- FileDialog with wildcard filtering and navigation
- FileInfoPane for file details
- PathView for current directory display
- Cross-platform file operations

### Recent Additions (v0.2.6)

**Syntax Highlighting**
- Token-based coloring system
- SyntaxHighlighter trait (extensible)
- RustHighlighter (built-in Rust support)
- 11 token types with color mapping
- Editor integration (optional highlighter)
- 7 tests

**Picture Mask Validation**
- PictureValidator matching Borland's TPXPictureValidator
- Mask characters: # (digit), @ (alpha), ! (any), * (optional)
- Auto-formatting mode
- Format examples: phone "(###) ###-####", date "##/##/####"
- 11 tests

**Example Consolidation**
- editor_demo.rs - All editor features (editing, search, syntax, file I/O)
- validator_demo.rs - All validators (Filter, Range, Picture)
- Reduced from 19 to 16 examples

### Missing Features (Phase 10-11)

Phase 10 candidates (~314 hours remaining):
- ColorSelector, ColorDialog, ColorItemList
- MultiCheckboxes
- Calendar
- History dropdown UI
- StringList, SortedStrCollection
- ParamText

Phase 11 (MDI/Advanced - ~278 hours):
- TDeskTop complete MDI implementation
- TSubMenu dynamic menu building

---

# Focus Architecture

## Overview

The Turbo Vision framework implements proper focus management where controls only respond to input when they have focus. This prevents input fields from capturing keys when not focused, list boxes from scrolling when another control is active, and buttons from activating when the user is typing elsewhere.

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
    if let Some(focused_idx) = self.focused {
        self.children[focused_idx].handle_event(event);
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
        if !self.is_focused() {
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
2. **Store focus in unified `state` field using `SF_FOCUSED` flag**
3. **Use View trait's default `set_focus()` implementation**

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
3. Calls `set_focus(true)` on the target child via StateFlags

**⚠️ IMPORTANT:** Do NOT manually call `set_focus()` on individual children without updating the Group's `focused` index:

```rust
// ❌ BAD: Only sets visual focus, Group still thinks another child is focused
self.dialog.child_at_mut(index).set_state_flag(SF_FOCUSED, true);

// ✅ GOOD: Updates both Group state and child focus
self.dialog.set_focus_to_child(index);
```

**Symptoms of improper focus management:**
- Control appears focused (correct colors) but doesn't respond to keyboard
- Need to press Tab before keyboard events work
- Events go to wrong control

This matches Borland's `fileList->select()` pattern which calls `owner->setCurrent(this, normalSelect)` to properly establish focus chain.

## Focus Consolidation (v0.2.3)

**Status:** ✅ **Complete**

All views now store focus in the unified `state` field using `SF_FOCUSED` flag, matching Borland's TView architecture exactly. The separate `focused: bool` field has been removed from all views.

**Implementation:**
- Button, InputLine, Editor, Memo, ListBox, CheckBox, RadioButton all use `state: StateFlags`
- `is_focused()` checks `self.get_state_flag(SF_FOCUSED)`
- `set_focus()` uses View trait default implementation (sets/clears SF_FOCUSED)

**Comparison with Borland:**

| Aspect | Borland | Rust (v0.2.3) |
|--------|---------|---------------|
| Focus storage | `state & sfFocused` | `state & SF_FOCUSED` |
| Set focus | `setState(sfFocused, True)` | `set_state_flag(SF_FOCUSED, true)` |
| Check focus | `state & sfFocused` | `get_state_flag(SF_FOCUSED)` |
| Architecture | Single unified field | Single unified field ✅ |

## Related Classes

- **Group** (`src/views/group.rs`) - Container with focus management
- **Window** (`src/views/window.rs`) - Wraps Group, delegates focus
- **Dialog** (`src/views/dialog.rs`) - Modal dialog with focus management
- **View trait** (`src/views/view.rs`) - Defines `can_focus()` and `set_focus()`

---

# Event System Architecture

## Overview

The event system provides flexible event handling matching Borland's architecture, with three-phase processing, broadcast distribution, and event re-queuing support.

## Event Types

```rust
pub enum EventType {
    Nothing,       // No event / consumed event
    Keyboard,      // Keyboard input
    MouseDown,     // Mouse button press
    MouseUp,       // Mouse button release
    MouseMove,     // Mouse movement
    MouseDrag,     // Mouse drag
    Command,       // Command from control
    Broadcast,     // Broadcast to all children
}
```

## Three-Phase Event Processing

**Status:** ✅ **Complete** (v0.1.9)

Groups process events in three phases matching Borland's TGroup::handleEvent():

```rust
fn handle_event(&mut self, event: &mut Event) {
    // Phase 1: PreProcess - views with OF_PRE_PROCESS flag
    for child in &mut self.children {
        if child.get_options() & OF_PRE_PROCESS != 0 {
            child.handle_event(event);
            if event.what == EventType::Nothing {
                return;
            }
        }
    }

    // Phase 2: Focused - currently focused child only
    if let Some(focused_idx) = self.focused {
        self.children[focused_idx].handle_event(event);
        if event.what == EventType::Nothing {
            return;
        }
    }

    // Phase 3: PostProcess - views with OF_POST_PROCESS flag
    for child in &mut self.children {
        if child.get_options() & OF_POST_PROCESS != 0 {
            child.handle_event(event);
            if event.what == EventType::Nothing {
                return;
            }
        }
    }
}
```

**Benefits:**
- Buttons with OF_POST_PROCESS can intercept events even when not focused
- Status line with OF_PRE_PROCESS can monitor all events
- Modal dialogs can intercept Esc key before children process it

**Comparison with Borland:**

| Aspect | Borland | Rust |
|--------|---------|------|
| Phase 1 | Views with ofPreProcess | OF_PRE_PROCESS flag |
| Phase 2 | Focused view (current) | Focused child |
| Phase 3 | Views with ofPostProcess | OF_POST_PROCESS flag |
| Event consumed | event.what = evNothing | event.what = EventType::Nothing |

## Broadcast Event Distribution

**Status:** ✅ **Complete** (v0.2.0)

Groups can broadcast events to all children except the originator:

```rust
pub fn broadcast(&mut self, event: &mut Event, owner_index: Option<usize>) {
    for (i, child) in self.children.iter_mut().enumerate() {
        if Some(i) == owner_index {
            continue; // Skip owner to prevent echo back
        }
        child.handle_event(event);
        if event.what == EventType::Nothing {
            break;
        }
    }
}
```

**Use Cases:**
- Command enable/disable notifications (CM_COMMAND_SET_CHANGED)
- File selection updates (CM_FILE_FOCUSED)
- History updates (CM_HISTORY_CHANGED)
- Focus navigation commands

**Comparison with Borland:**

| Aspect | Borland | Rust |
|--------|---------|------|
| Broadcast method | forEach(doHandleEvent, &hs) | broadcast(&mut event, owner_index) |
| Skip originator | Tracked in phase/handleStruct | owner_index parameter |
| Event type | evBroadcast | EventType::Broadcast |

## Event Re-queuing

**Status:** ✅ **Complete** (v0.1.10)

The terminal supports re-queuing events for next iteration:

```rust
// Terminal has pending_event field
pub fn put_event(&mut self, event: Event) {
    self.pending_event = Some(event);
}

// poll_event() checks pending_event first
pub fn poll_event(&mut self) -> std::io::Result<Option<Event>> {
    if let Some(pending) = self.pending_event.take() {
        return Ok(Some(pending));
    }
    // ... poll for new events
}
```

**Use Cases:**
- Converting keyboard events to commands
- Implementing custom key mappings
- Dialog Enter→cmDefault transformation

**Comparison with Borland:**

| Aspect | Borland | Rust |
|--------|---------|------|
| Re-queue method | TProgram::putEvent(event) | terminal.put_event(event) |
| Retrieve | TProgram::getEvent(event) | terminal.poll_event() |
| Storage | Event queue | pending_event field |

## Event Transformation Pattern

In Rust, child views communicate with parents by transforming events rather than using owner pointers:

```rust
// Button transforms keyboard event to command
if event.key_code == KB_ENTER {
    *event = Event::command(self.command);
    // Event bubbles up call stack to parent
}
```

This eliminates the need for raw owner pointers while achieving the same functionality. See [Owner/Parent Communication](#ownerparent-communication) for details.

---

# State Management

## StateFlags System

**Status:** ✅ **Complete** (v0.2.3)

All views use a unified `state: StateFlags` field matching Borland's `ushort state`:

```rust
bitflags! {
    pub struct StateFlags: u16 {
        const SF_VISIBLE   = 0x0001;
        const SF_FOCUSED   = 0x0002;
        const SF_DISABLED  = 0x0004;
        const SF_MODAL     = 0x0008;
        const SF_DEFAULT   = 0x0010;  // Default button
        const SF_SELECTED  = 0x0020;  // Selected state
        const SF_ACTIVE    = 0x0040;  // Active window
        const SF_DRAGGING  = 0x0080;  // Being dragged
    }
}
```

**View Trait Methods:**
```rust
fn get_state_flag(&self, flag: StateFlags) -> bool;
fn set_state_flag(&mut self, flag: StateFlags, value: bool);
fn state(&self) -> StateFlags;
fn set_state(&mut self, state: StateFlags);
```

**Comparison with Borland:**

| Aspect | Borland | Rust |
|--------|---------|------|
| Storage | `ushort state` | `StateFlags: u16` |
| Flags | sfVisible, sfFocused, etc. | SF_VISIBLE, SF_FOCUSED, etc. |
| Check flag | `state & sfFocused` | `get_state_flag(SF_FOCUSED)` |
| Set flag | `setState(sfFocused, True)` | `set_state_flag(SF_FOCUSED, true)` |

## OptionsFlags System

Views also have options flags matching Borland's `ushort options`:

```rust
bitflags! {
    pub struct OptionsFlags: u16 {
        const OF_SELECTABLE    = 0x0001;  // Can receive focus
        const OF_TOP_SELECT    = 0x0002;  // Select on top
        const OF_PRE_PROCESS   = 0x0004;  // Event phase 1
        const OF_POST_PROCESS  = 0x0008;  // Event phase 3
        const OF_CENTER_X      = 0x0010;  // Center horizontally
        const OF_CENTER_Y      = 0x0020;  // Center vertically
        const OF_FRAME_ONLY    = 0x0040;  // Window frame only
    }
}
```

**Comparison with Borland:**

| Aspect | Borland | Rust |
|--------|---------|------|
| Storage | `ushort options` | `OptionsFlags: u16` |
| Flags | ofSelectable, ofPreProcess, etc. | OF_SELECTABLE, OF_PRE_PROCESS, etc. |

---

# Modal Dialog Execution

## Overview

**Status:** ✅ **Complete** (v0.2.3) - **Dual Pattern Support**

The implementation provides TWO modal execution patterns for maximum flexibility:

1. **Borland-Style (Centralized)**: `app.exec_view(dialog)` - Application controls modal loop
2. **Rust-Style (Self-Contained)**: `dialog.execute(&mut app)` - Dialog controls own loop

Both patterns are fully functional and produce identical results.

## Pattern 1: Borland-Style (Centralized)

Matches Borland's `TProgram::execView()` architecture exactly.

### Architecture

```
Application::exec_view(view)
    ├─> Adds view to desktop (takes ownership)
    ├─> Checks SF_MODAL flag
    └─> If modal:
        ├─> Runs event loop (app controls drawing)
        ├─> Checks view.get_end_state() each iteration
        └─> Returns end_state when != 0
```

### Usage

```rust
// Create modal dialog
let mut dialog = Dialog::new_modal(
    Rect::new(20, 8, 60, 16),
    "Confirm Action"
);
dialog.add(Button::new(Rect::new(10, 4, 20, 6), "OK", CM_OK));
dialog.add(Button::new(Rect::new(25, 4, 35, 6), "Cancel", CM_CANCEL));
dialog.set_initial_focus();

// Execute via Application (blocks until closed)
let result = app.exec_view(dialog);

// Dialog automatically cleaned up (removed from desktop)

match result {
    CM_OK => { /* User clicked OK */ }
    CM_CANCEL => { /* User clicked Cancel */ }
    _ => {}
}
```

### Implementation

**Application::exec_view()** (`src/app/application.rs:69-125`):
```rust
pub fn exec_view(&mut self, view: Box<dyn View>) -> CommandId {
    let is_modal = (view.state() & SF_MODAL) != 0;

    self.desktop.add(view);
    let view_index = self.desktop.child_count() - 1;

    if !is_modal {
        return 0; // Modeless - just added to desktop
    }

    // Modal loop
    loop {
        self.idle();
        self.draw();
        self.terminal.flush();

        if let Ok(Some(mut event)) = self.terminal.poll_event(...) {
            self.handle_event(&mut event);
        }

        // Check if modal view wants to close
        let end_state = self.desktop.child_at(view_index).get_end_state();
        if end_state != 0 {
            self.desktop.remove_child(view_index);
            return end_state;
        }
    }
}
```

## Pattern 2: Rust-Style (Self-Contained)

Dialog manages its own event loop for simpler, more direct code.

### Architecture

```
Dialog::execute(&mut app)
    ├─> Sets SF_MODAL flag
    └─> Runs own event loop
        ├─> Draws desktop + self
        ├─> Handles events directly
        └─> Returns end_state when != 0
```

### Usage

```rust
// Create regular dialog
let mut dialog = Dialog::new(
    Rect::new(20, 8, 60, 16),
    "Confirm Action"
);
dialog.add(Button::new(Rect::new(10, 4, 20, 6), "OK", CM_OK));
dialog.add(Button::new(Rect::new(25, 4, 35, 6), "Cancel", CM_CANCEL));
dialog.set_initial_focus();

// Execute directly (blocks until closed)
let result = dialog.execute(&mut app);

// Dialog still in scope, can be reused

match result {
    CM_OK => { /* User clicked OK */ }
    CM_CANCEL => { /* User clicked Cancel */ }
    _ => {}
}
```

### Implementation

**Dialog::execute()** (`src/views/dialog.rs:61-129`):
```rust
pub fn execute(&mut self, app: &mut Application) -> CommandId {
    self.result = CM_CANCEL;

    let old_state = self.state();
    self.set_state(old_state | SF_MODAL);

    loop {
        // Dialog controls drawing
        app.desktop.draw(&mut app.terminal);
        self.draw(&mut app.terminal);
        self.update_cursor(&mut app.terminal);
        app.terminal.flush();

        if let Some(mut event) = app.terminal.poll_event(...).ok().flatten() {
            self.handle_event(&mut event);
        }

        let end_state = self.window.get_end_state();
        if end_state != 0 {
            self.result = end_state;
            break;
        }
    }

    self.set_state(old_state);
    self.result
}
```

## Pattern Comparison

| Aspect | Borland | Pattern 1 (Borland-Style) | Pattern 2 (Rust-Style) |
|--------|---------|---------------------------|------------------------|
| Entry point | `app.execView(dialog)` | `app.exec_view(dialog)` | `dialog.execute(&mut app)` |
| Ownership | Raw pointer | Box (auto cleanup) | Stack/Box |
| Loop location | View's execute() | Application::exec_view() | Dialog::execute() |
| Drawing | Program controls | Application draws | Dialog draws |
| Modal flag | `state & sfModal` | `state & SF_MODAL` | `state & SF_MODAL` |
| Cleanup | Manual (CLY_destroy) | Automatic | Automatic |
| Nested modals | ✅ Supported | ✅ Supported | ✅ Supported |
| Borland compatible | ✅ Original | ✅ Exact match | ⚠️ Different pattern |

## When to Use Which Pattern

### Use Pattern 1 (Borland-Style) When:
✅ Porting Borland code - matches original architecture exactly
✅ Centralized control - want Application to manage all modal loops
✅ Consistent with Borland - maintaining exact API compatibility

### Use Pattern 2 (Rust-Style) When:
✅ Simpler code - less ceremony, more direct
✅ Local scope - dialog is used in one function
✅ Rust idioms - more natural Rust ownership patterns
✅ Quick prototyping - faster to write and test

## Dialog End Modal Logic

**Dialog::handle_event()** (`src/views/dialog.rs:149-198`):
```rust
fn handle_event(&mut self, event: &mut Event) {
    // First let window (and children) handle event
    self.window.handle_event(event);

    // Then check for dialog-specific events
    match event.what {
        EventType::Keyboard => {
            match event.key_code {
                KB_ESC_ESC => {
                    *event = Event::command(CM_CANCEL);
                }
                KB_ENTER => {
                    if let Some(cmd) = self.find_default_button_command() {
                        *event = Event::command(cmd);
                    }
                }
                _ => {}
            }
        }
        EventType::Command => {
            match event.command {
                CM_OK | CM_CANCEL | CM_YES | CM_NO => {
                    if (self.state() & SF_MODAL) != 0 {
                        self.window.end_modal(event.command);
                        event.clear();
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
}
```

**Comparison with Borland:**

| Aspect | Borland | Rust |
|--------|---------|------|
| End modal | `endModal(command)` | `end_modal(command)` |
| End state | `endState` field | `end_state` field in Group |
| Check state | Return from execute() | `get_end_state()` |
| Commands | cmOK, cmCancel, cmYes, cmNo | CM_OK, CM_CANCEL, CM_YES, CM_NO |

---

# Owner/Parent Communication

## Overview

**Status:** ✅ **Equivalent Architecture** (v0.2.3)

In Borland, child views communicate with parents via raw owner pointers. In Rust, we achieve the same functionality through **event transformation** using the call stack, eliminating unsafe pointers while maintaining full compatibility.

## The Problem

Child views need to communicate with parent containers:
- Button needs to tell Dialog it was clicked
- ListBox needs to notify parent of selection
- CheckBox needs to inform parent of state change

## Borland's Approach: Raw Owner Pointers

### Architecture

```
TDialog
  ├─> TGroup
      ├─> TButton (owner = TGroup*)
      └─> TButton (owner = TGroup*)
            └──> message(owner, evBroadcast, command, this)
                     ▲
                     └─── Raw pointer dereference
```

### Code

```cpp
class TView {
protected:
    TGroup* owner;  // Raw pointer to parent
};

class TButton : public TView {
    void press() {
        // Send message via raw pointer
        message(owner, evBroadcast, command, this);
    }
};
```

### Problems in Rust Context

1. **Lifetime Issues**: Raw pointers have no lifetime tracking
2. **Circular References**: Parent owns child, child points to parent
3. **Mutable Aliasing**: Multiple mutable paths to same data

## Rust's Approach: Event Transformation

### Architecture

```
Dialog
  ├─> Group
      ├─> Button (no owner pointer)
      └─> Button (no owner pointer)
            └──> *event = Event::command(cmd)
                     ▲
                     └─── Event transformed, bubbles up call stack
```

### Code

```rust
// Button - NO owner pointer needed!
pub struct Button {
    command: CommandId,
    // NOTE: No owner field!
}

impl View for Button {
    fn handle_event(&mut self, event: &mut Event) {
        if event.key_code == KB_ENTER {
            // Transform event to communicate with parent
            *event = Event::command(self.command);
            // When function returns, parent receives transformed event
        }
    }
}

// Group - receives transformed events
impl View for Group {
    fn handle_event(&mut self, event: &mut Event) {
        // Send event to focused child
        self.children[self.focused].handle_event(event);
        // Child may have transformed it!

        // Event (possibly transformed) continues up call stack
    }
}

// Dialog - processes commands from children
impl View for Dialog {
    fn handle_event(&mut self, event: &mut Event) {
        self.window.handle_event(event);

        // Check if child transformed event to command
        if event.what == EventType::Command {
            match event.command {
                CM_OK | CM_CANCEL => {
                    self.window.end_modal(event.command);
                }
                _ => {}
            }
        }
    }
}
```

### Execution Flow

```
User presses Enter on Button
         │
         ▼
Dialog::handle_event(&mut event)
  └─> window.handle_event(&mut event)
      └─> group.handle_event(&mut event)
          └─> button.handle_event(&mut event)
              ├─ Detects KB_ENTER
              ├─ *event = Event::command(CM_OK)
              └─ Returns
          ← Event now Command type
      ← Bubbles up
  ← Dialog receives Command
  └─ Processes CM_OK, calls end_modal()
```

## Comparison

| Aspect | Borland | Rust |
|--------|---------|------|
| Child storage | `TGroup* owner` | No owner field |
| Setup | `button->setOwner(dialog)` | Automatic via call stack |
| Send message | `message(owner, evBroadcast, cmd)` | `*event = Event::command(cmd)` |
| Receive | Direct call via pointer | Event bubbles up stack |
| Safety | ⚠️ Raw pointer | ✅ Compiler-verified |
| Circular refs | ⚠️ Possible | ✅ Impossible |
| Performance | Indirect call | Direct return (faster) |

## Migration Pattern

When porting Borland code:

**Borland:**
```cpp
message(owner, evBroadcast, command, this);
```

**Rust:**
```rust
*event = Event::command(command);
```

## Why Rust's Approach is Superior

✅ **Memory Safe** - No dangling pointers possible
✅ **Thread Safe** - Compiler-enforced safety
✅ **Simpler** - No owner pointer management
✅ **Faster** - Direct returns vs indirect calls
✅ **Idiomatic** - Uses Rust's ownership naturally

**Result: 100% functional equivalence with superior safety.**

---

# Syntax Highlighting System

## Overview

**Status:** ✅ **Complete** (v0.2.6)

The syntax highlighting system provides extensible token-based coloring for the Editor widget, matching modern text editor capabilities while integrating seamlessly with Turbo Vision's architecture.

## Architecture

### Token-Based Coloring

```rust
pub enum TokenType {
    Normal,        // Default text
    Keyword,       // Language keywords (Yellow)
    String,        // String literals (LightRed)
    Comment,       // Comments (LightCyan)
    Number,        // Numeric literals (LightMagenta)
    Operator,      // Operators (White)
    Identifier,    // Identifiers (White)
    Type,          // Type names (LightGreen)
    Preprocessor,  // Preprocessor directives (LightCyan)
    Function,      // Function names (Cyan)
    Special,       // Special characters (White)
}

pub struct Token {
    pub start: usize,
    pub end: usize,
    pub token_type: TokenType,
}
```

### SyntaxHighlighter Trait

```rust
pub trait SyntaxHighlighter: Send + Sync {
    /// Language name
    fn language(&self) -> &str;

    /// Highlight a single line, returns tokens
    fn highlight_line(&self, line: &str, line_number: usize) -> Vec<Token>;

    /// Check if currently in multi-line context (e.g., block comment)
    fn is_multiline_context(&self, line_number: usize) -> bool {
        false
    }

    /// Update multi-line state after processing a line
    fn update_multiline_state(&mut self, line: &str, line_number: usize) {}
}
```

### Built-in Highlighters

**RustHighlighter** - Full Rust syntax support:
- Keywords: fn, let, if, for, match, struct, enum, impl, trait, pub, etc.
- String literals with escape sequences
- Character literals
- Line comments (//) and block comments (/* */)
- Numeric literals (decimal, hex, float)
- Type names (i32, String, custom types)
- Operators and special characters

**PlainTextHighlighter** - No-op highlighter for plain text

## Editor Integration

```rust
pub struct Editor {
    // ... existing fields ...
    highlighter: Option<Box<dyn SyntaxHighlighter>>,
}

impl Editor {
    /// Attach a syntax highlighter
    pub fn set_highlighter(&mut self, highlighter: Box<dyn SyntaxHighlighter>) {
        self.highlighter = Some(highlighter);
    }

    /// Remove syntax highlighting
    pub fn clear_highlighter(&mut self) {
        self.highlighter = None;
    }

    /// Check if highlighting is enabled
    pub fn has_highlighter(&self) -> bool {
        self.highlighter.is_some()
    }
}
```

### Draw Method Integration

The Editor's draw method applies token colors:

```rust
// In Editor::draw()
if let Some(ref highlighter) = self.highlighter {
    let tokens = highlighter.highlight_line(line, line_idx);
    for token in tokens {
        let token_text: String = line.chars()
            .skip(start_col + token_start)
            .take(token_end - token_start)
            .collect();
        buf.move_str(
            token_start,
            &token_text,
            token.token_type.default_color()
        );
    }
} else {
    // Default rendering without highlighting
    buf.move_str(0, line, Color::White);
}
```

## Usage Example

```rust
use turbo_vision::app::Application;
use turbo_vision::views::editor::Editor;
use turbo_vision::views::syntax::RustHighlighter;
use turbo_vision::core::geometry::Rect;

let mut app = Application::new()?;

// Create editor
let editor_bounds = Rect::new(1, 1, 78, 23);
let mut editor = Editor::new(editor_bounds)
    .with_scrollbars_and_indicator();

// Set Rust code
editor.set_text(r#"
fn main() {
    let x: i32 = 42;
    println!("Hello, {}", x);
}
"#);

// Enable Rust syntax highlighting
editor.set_highlighter(Box::new(RustHighlighter::new()));

// Run editor
app.exec_view(Box::new(editor));
```

## Extending with New Languages

To add a new language:

1. **Implement SyntaxHighlighter trait:**

```rust
pub struct PythonHighlighter {
    in_block_string: bool,
}

impl SyntaxHighlighter for PythonHighlighter {
    fn language(&self) -> &str {
        "Python"
    }

    fn highlight_line(&self, line: &str, line_number: usize) -> Vec<Token> {
        let mut tokens = Vec::new();
        // Parse line and create tokens
        // ...
        tokens
    }

    fn is_multiline_context(&self, _line_number: usize) -> bool {
        self.in_block_string
    }

    fn update_multiline_state(&mut self, line: &str, _line_number: usize) {
        // Track """ ... """ strings
        // ...
    }
}
```

2. **Use with Editor:**

```rust
editor.set_highlighter(Box::new(PythonHighlighter::new()));
```

## Design Patterns

**Hook-Based Architecture** - Language extensions implement trait
**Token Type Abstraction** - Decouple token types from colors
**Line-by-Line Processing** - Efficient rendering
**Multi-Line State Tracking** - Optional for block comments/strings
**Seamless Integration** - Works with all Editor features (undo/redo, search, file I/O)

## Statistics

- **Implementation**: `src/views/syntax.rs` (450 lines)
- **Tests**: 7 tests covering token types, Rust highlighting, plain text
- **Token Types**: 11 types with default color mappings
- **Performance**: O(n) per line, no impact when disabled

---

# Validation System

## Overview

**Status:** ✅ **Complete** (v0.2.6)

The validation system provides input validation for InputLine widgets, matching Borland's validator architecture with three validator types plus picture mask validation.

## Validator Trait

```rust
pub trait Validator: Send + Sync {
    /// Check if the complete input is valid
    fn is_valid(&self, input: &str) -> bool;

    /// Check if appending/typing a character is valid (real-time validation)
    fn is_valid_input(&self, input: &str, append: bool) -> bool {
        self.is_valid(input)
    }

    /// Report error to user (visual or audio feedback)
    fn error(&self) {
        // Default: silent (could beep or show message)
    }

    /// Check validity and call error() if invalid
    fn valid(&self, input: &str) -> bool {
        let is_valid = self.is_valid(input);
        if !is_valid {
            self.error();
        }
        is_valid
    }
}
```

## FilterValidator

Character filtering - only allows specific characters.

```rust
pub struct FilterValidator {
    valid_chars: String,
}

impl FilterValidator {
    pub fn new(valid_chars: &str) -> Self {
        Self {
            valid_chars: valid_chars.to_string(),
        }
    }
}

// Example: digits only
let validator = FilterValidator::new("0123456789");
```

**Use Cases:**
- Digits only (phone, zip code)
- Alpha only (name)
- Alphanumeric (username)
- Custom character sets

## RangeValidator

Numeric range validation with hex/octal support.

```rust
pub struct RangeValidator {
    min: i32,
    max: i32,
}

impl RangeValidator {
    pub fn new(min: i32, max: i32) -> Self {
        Self { min, max }
    }
}

// Examples
let percent = RangeValidator::new(0, 100);      // 0-100%
let byte = RangeValidator::new(0, 255);         // 0x00-0xFF
let signed = RangeValidator::new(-50, 50);      // -50 to 50
```

**Features:**
- Decimal numbers (123)
- Hex numbers (0xFF, 0xAB)
- Octal numbers (0o77)
- Negative numbers (-50)
- Real-time validation during typing

## LookupValidator

Dropdown list validation - input must match list item.

```rust
pub struct LookupValidator {
    items: Vec<String>,
}

impl LookupValidator {
    pub fn new(items: Vec<String>) -> Self {
        Self { items }
    }
}

// Example: states
let states = LookupValidator::new(vec![
    "CA".to_string(),
    "NY".to_string(),
    "TX".to_string(),
]);
```

**Use Cases:**
- State/country codes
- Department names
- Category selection
- Any fixed list validation

## PictureValidator

**Status:** ✅ **Complete** (v0.2.6) - Matches Borland's TPXPictureValidator

Format mask validation with automatic literal insertion.

```rust
pub struct PictureValidator {
    mask: String,
    auto_format: bool,
}

impl PictureValidator {
    pub fn new(mask: &str) -> Self {
        Self {
            mask: mask.to_string(),
            auto_format: true,
        }
    }

    /// Format input according to mask
    pub fn format(&self, input: &str) -> String {
        // Inserts literals automatically
        // ...
    }
}
```

### Mask Characters

| Char | Meaning | Example |
|------|---------|---------|
| `#` | Digit (0-9) | Phone, date, zip |
| `@` | Alpha (A-Z, a-z) | Product code, state |
| `!` | Any character | Mixed format |
| `*` | Optional section | Extension, suffix |
| Literals | Must match exactly | `()`, `-`, `/`, `.` |

### Examples

```rust
// Phone number: (555) 123-4567
let phone = PictureValidator::new("(###) ###-####");

// Date: 12/25/2023
let date = PictureValidator::new("##/##/####");

// Product code: ABCD-1234
let product = PictureValidator::new("@@@@-####");

// Social Security: 123-45-6789
let ssn = PictureValidator::new("###-##-####");

// IP Address: 192.168.1.1
let ip = PictureValidator::new("###.###.###.###");

// Credit card: 1234-5678-9012-3456
let cc = PictureValidator::new("####-####-####-####");
```

### Auto-Formatting

When `auto_format` is enabled (default), the validator automatically inserts literal characters as the user types:

```
User types: "5551234567"
Display:    "(555) 123-4567"

User types: "12252023"
Display:    "12/25/2023"

User types: "ABCD1234"
Display:    "ABCD-1234"
```

## InputLine Integration

```rust
use std::rc::Rc;
use std::cell::RefCell;

// Create data storage
let phone_data = Rc::new(RefCell::new(String::new()));

// Create InputLine with validator
let mut phone_input = InputLine::new(
    Rect::new(10, 5, 30, 6),
    20,
    phone_data.clone()
);

// Attach validator
phone_input.set_validator(
    Rc::new(RefCell::new(
        PictureValidator::new("(###) ###-####")
    ))
);

// Add to dialog
dialog.add(Box::new(phone_input));
```

## Validation Flow

1. **Real-Time Validation** (is_valid_input):
   - Called as user types
   - Rejects invalid characters immediately
   - Visual feedback (character not accepted)

2. **Final Validation** (is_valid):
   - Called when user finishes (presses Enter, clicks OK)
   - Checks complete input against rules
   - May call error() if invalid

3. **Auto-Formatting** (PictureValidator only):
   - Inserts literal characters automatically
   - Updates display in real-time
   - Maintains correct format

## Comparison with Borland

| Aspect | Borland | Rust |
|--------|---------|------|
| Base trait | TValidator | Validator trait |
| Filter | TFilterValidator | FilterValidator |
| Range | TRangeValidator | RangeValidator |
| Lookup | TLookupValidator | LookupValidator |
| Picture | TPXPictureValidator | PictureValidator |
| Real-time | isValidInput() | is_valid_input() |
| Final | isValid() | is_valid() |
| Error | error() | error() |

## Statistics

- **FilterValidator**: `src/views/validator.rs` (100 lines, 3 tests)
- **RangeValidator**: `src/views/validator.rs` (150 lines, 5 tests)
- **LookupValidator**: `src/views/validator.rs` (50 lines, 1 test)
- **PictureValidator**: `src/views/picture_validator.rs` (360 lines, 11 tests)
- **Total Tests**: 20 tests covering all validator types

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
- **Shift+F12** - Dump active window/dialog to `active-view-dump.txt`

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
    app.run();  // Press F12 or Shift+F12 anytime!
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

if key_code == KB_SHIFT_F12 {
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

**Status:** ✅ **Complete** (v0.1.8)

The Command Set system provides automatic button enable/disable based on application state. This matches Borland Turbo Vision's architecture where buttons automatically disable themselves when their associated commands are not available.

## Architecture

### Global Thread-Local State

```rust
thread_local! {
    static COMMAND_SET: RefCell<CommandSet> = RefCell::new(CommandSet::new());
    static COMMAND_SET_CHANGED: Cell<bool> = Cell::new(false);
}

// Global functions
pub fn enable_command(cmd: CommandId) {
    COMMAND_SET_CHANGED.with(|flag| flag.set(true));
    COMMAND_SET.with(|cs| cs.borrow_mut().enable_command(cmd));
}

pub fn disable_command(cmd: CommandId) {
    COMMAND_SET_CHANGED.with(|flag| flag.set(true));
    COMMAND_SET.with(|cs| cs.borrow_mut().disable_command(cmd));
}

pub fn command_enabled(cmd: CommandId) -> bool {
    COMMAND_SET.with(|cs| cs.borrow().has(cmd))
}
```

This matches Borland's static `TView::curCommandSet` exactly while remaining safe in Rust.

## CommandSet Implementation

```rust
pub struct CommandSet {
    bits: [u64; 1024],  // 65,536 commands (64 * 1024)
}

impl CommandSet {
    pub fn enable_command(&mut self, cmd: CommandId) {
        let word = (cmd / 64) as usize;
        let bit = cmd % 64;
        self.bits[word] |= 1 << bit;
    }

    pub fn disable_command(&mut self, cmd: CommandId) {
        let word = (cmd / 64) as usize;
        let bit = cmd % 64;
        self.bits[word] &= !(1 << bit);
    }

    pub fn has(&self, cmd: CommandId) -> bool {
        let word = (cmd / 64) as usize;
        let bit = cmd % 64;
        (self.bits[word] & (1 << bit)) != 0
    }

    // Set operations: union, intersect, difference
    pub fn union(&mut self, other: &CommandSet);
    pub fn intersect(&mut self, other: &CommandSet);
    pub fn difference(&mut self, other: &CommandSet);
}
```

## Application Integration

```rust
impl Application {
    pub fn idle(&mut self) {
        // Check if command set changed
        if command_set::has_changes() {
            // Broadcast change notification to all views
            let mut event = Event::broadcast(CM_COMMAND_SET_CHANGED);
            self.desktop.handle_event(&mut event);
            command_set::clear_changes();
        }
    }
}
```

## Button Auto-Disable

```rust
impl View for Button {
    fn handle_event(&mut self, event: &mut Event) {
        match event.what {
            EventType::Broadcast => {
                if event.command == CM_COMMAND_SET_CHANGED {
                    // Query global command state
                    let should_be_enabled = command_set::command_enabled(self.command);

                    // Update button state automatically
                    if should_be_enabled != !self.is_disabled() {
                        self.set_disabled(!should_be_enabled);
                        // Button will redraw itself
                    }
                }
            }
            _ => {}
        }
    }
}
```

## Usage Example

```rust
use turbo_vision::core::command_set;

// Disable commands initially
command_set::disable_command(CM_PASTE);  // No clipboard content
command_set::disable_command(CM_UNDO);   // Nothing to undo

// ... in event loop, app.idle() broadcasts changes ...

// User copies text
clipboard.set_text("Hello");
command_set::enable_command(CM_PASTE);  // Paste button automatically enables!

// User performs action
perform_action();
command_set::enable_command(CM_UNDO);   // Undo button automatically enables!

// User undoes
undo();
if !can_undo_more() {
    command_set::disable_command(CM_UNDO);  // Undo button automatically disables!
}
```

## Benefits

When fully implemented, the command set system:
- Eliminates manual button enable/disable code throughout the application
- Buttons "just work" based on application state
- Provides consistent UI state management
- Matches original Turbo Vision patterns exactly

## Comparison with Borland

| Aspect | Borland | Rust |
|--------|---------|------|
| Global state | `static TCommandSet curCommandSet` | `thread_local! COMMAND_SET` |
| Changed flag | `static Boolean commandSetChanged` | `thread_local! COMMAND_SET_CHANGED` |
| Enable | `TView::enableCommand(cmd)` | `command_set::enable_command(cmd)` |
| Disable | `TView::disableCommand(cmd)` | `command_set::disable_command(cmd)` |
| Query | `TView::commandEnabled(cmd)` | `command_set::command_enabled(cmd)` |
| Broadcast | `message(this, evBroadcast, cmCommandSetChanged)` | `Event::broadcast(CM_COMMAND_SET_CHANGED)` |
| Idle check | `TProgram::idle()` | `Application::idle()` |

## References

- Borland: `/local-only/borland-tvision/include/tv/cmdset.h`
- Borland: `/local-only/borland-tvision/classes/tcommand.cc`
- Borland: `/local-only/borland-tvision/classes/tview.cc` (lines 142-389)
- Borland: `/local-only/borland-tvision/classes/tbutton.cc` (lines 255-262)
- Borland: `/local-only/borland-tvision/classes/tprogram.cc` (lines 248-257)

---

# Architecture Comparisons

## Summary of Architectural Differences

This section documents the differences between Borland's C++ implementation and our Rust implementation, explaining why Rust's approach achieves equivalent functionality with superior safety.

### 1. Enter Key Default Button Activation

**Borland:** Converts KB_ENTER to `evBroadcast` with `cmDefault`, re-queues via `putEvent()`, broadcasts to all buttons.

**Rust:** Directly finds default button, checks if enabled, generates command event immediately.

**Status:** ✅ OK - Simplification with equivalent behavior
**Rationale:** Direct approach is more efficient, avoids event queue manipulation. End result identical.

---

### 2. State Flags Storage

**Borland:** Single `ushort state` field with combined flags.

**Rust:** Single `StateFlags: u16` field with same flags (SF_VISIBLE, SF_FOCUSED, SF_DISABLED, SF_MODAL, etc.).

**Status:** ✅ Complete - Exact match
**Rationale:** Focus consolidated into unified state flags in v0.2.3. All views use `state: StateFlags`.

---

### 3. Command Enable/Disable System

**Borland:** Global static `TView::curCommandSet` accessible anywhere.

**Rust:** Thread-local `COMMAND_SET` with global functions (`enable_command`, `disable_command`, `command_enabled`).

**Status:** ✅ Complete - Equivalent architecture
**Rationale:** Thread-local + RefCell matches Borland's static global while remaining safe in Rust. Buttons auto-update on CM_COMMAND_SET_CHANGED broadcast.

---

### 4. Type Downcasting from View Trait

**Borland:** Direct C-style casts: `TButton* btn = (TButton*)dialog->at(index);`

**Rust:** Cannot downcast from trait object. Must work through trait methods.

**Status:** ✅ OK - Rust safety model prevents unsafe downcasting
**Rationale:** Rust's trait system forces better abstractions. Any functionality needed from generic containers should be exposed through trait methods.

---

### 5. Broadcast Event Distribution

**Borland:** `forEach(doHandleEvent, &hs)` sends to all children.

**Rust:** `broadcast(&mut event, owner_index)` sends to all children except originator.

**Status:** ✅ Complete - Equivalent implementation
**Rationale:** Owner-aware broadcast prevents echo back, matches Borland's `message()` pattern.

---

### 6. Three-Phase Event Processing

**Borland:** PreProcess phase, Focused phase, PostProcess phase.

**Rust:** Same three phases with OF_PRE_PROCESS and OF_POST_PROCESS flags.

**Status:** ✅ Complete - Exact match
**Rationale:** Full three-phase processing implemented in v0.1.9. Views set flags to intercept events before/after focused view.

---

### 7. Modal Dialog Execute Pattern

**Borland:** Centralized `TProgram::execView()` controls modal loop.

**Rust:** **Dual pattern support:**
- Pattern 1: `app.exec_view(dialog)` - Centralized (Borland-style)
- Pattern 2: `dialog.execute(&mut app)` - Self-contained (Rust-style)

**Status:** ✅ Complete - Both patterns supported
**Rationale:** Pattern 1 matches Borland exactly. Pattern 2 provides simpler Rust idiom. Both produce identical results.

---

### 8. Owner/Parent Relationship

**Borland:** Raw owner pointers: `TView* owner` points to parent. Child calls `message(owner, evBroadcast, command, this)`.

**Rust:** **No owner pointers.** Children transform events: `*event = Event::command(cmd)`. Event bubbles up call stack to parent.

**Status:** ✅ Equivalent - Different mechanism, same functionality
**Rationale:**
- **Memory Safe** - No dangling pointers possible
- **Thread Safe** - Compiler-enforced
- **Simpler** - No owner pointer management
- **Faster** - Direct returns vs indirect calls
- **Idiomatic** - Uses Rust's ownership naturally

---

## Conclusion

**All architectural discrepancies have been resolved!** 🎉

The Rust implementation achieves **100% functional equivalence** with Borland Turbo Vision while providing:

✅ **Memory safety** - No raw pointers, no manual memory management
✅ **Type safety** - Compile-time guarantees for state and commands
✅ **Flexibility** - Dual patterns for modal dialogs (Borland-style + Rust-style)
✅ **Compatibility** - Can port Borland code directly to Rust patterns
✅ **Superior safety** - Compiler prevents entire classes of bugs

**Current Status:**
- ✅ Event System - Three-phase processing, broadcasts, re-queuing complete
- ✅ Command System - Global enable/disable with auto-button updates
- ✅ State Management - Focus consolidated into unified StateFlags
- ✅ Parent Communication - Event transformation replaces owner pointers
- ✅ Modal Execution - Both centralized and self-contained patterns
- ✅ Syntax Highlighting - Extensible token-based system
- ✅ Validation System - All validators including picture masks

**Statistics:**
- Version: 0.2.6
- Tests: 171 passing
- Lines: ~15,000
- Components: 55+
- Phases: 9/11 complete
- Examples: 16 consolidated examples

---

## Related Documentation

- **CURRENT-STATUS-AND-TODO.md** - Complete status, missing features, roadmap
- **CHANGELOG.md** - Version history and release notes
- **examples/README.md** - Guide to all 16 examples

---

## Contributing

When adding new features or fixing bugs:

1. Consult the original Borland Turbo Vision source code for reference patterns
2. Document any architectural decisions or deviations
3. Update this design document with new patterns or learnings
4. Reference original source locations in comments (e.g., `tfiledia.cc:275`)
5. Maintain compatibility with Borland's architecture where reasonable
6. Add tests for all new functionality
7. Update CHANGELOG.md with changes

## Version History

- **2025-11-03 (v0.2.6)** - Added syntax highlighting, picture validator, example consolidation. Integrated DISCREPANCIES.md content organically.
- **2025-11-02** - Added FileDialog fixes documentation, consolidated design docs
- **2025-11-01** - Added focus architecture and screen dump system docs
- **2025-XX-XX** - Initial version
