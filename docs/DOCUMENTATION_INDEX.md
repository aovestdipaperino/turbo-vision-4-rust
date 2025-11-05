# Turbo Vision for Rust - Documentation Index

This directory contains comprehensive documentation analyzing the Rust implementation of Turbo Vision, created to support converting Pascal-based documentation to Rust.

## Quick Navigation

### For Quick Answers
Start with **[QUICK_REFERENCE.txt](QUICK_REFERENCE.txt)** - contains code snippets and patterns for common tasks

### For Complete Understanding
Read **[RUST_IMPLEMENTATION_REFERENCE.md](RUST_IMPLEMENTATION_REFERENCE.md)** - comprehensive guide with all features explained

### For File Organization
Check **[KEY_FILES_SUMMARY.txt](KEY_FILES_SUMMARY.txt)** - find where each feature is implemented

### For Overview
See **[FINDINGS_SUMMARY.md](FINDINGS_SUMMARY.md)** - executive summary of all findings

---

## Documentation Files

### 1. FINDINGS_SUMMARY.md (11 KB)
**Overview of the entire analysis**

Contains:
- Executive summary
- 7 key components analyzed
- Architecture patterns
- Key differences from Borland
- Implementation summary by component
- How to use the documentation

**Use when**: You want a high-level overview or are new to Turbo Vision

---

### 2. RUST_IMPLEMENTATION_REFERENCE.md (23 KB)
**Comprehensive implementation guide with code examples**

Sections:
1. Event System - EventType, Event struct, key codes, event creation
2. Command System - CommandId, standard commands, custom commands
3. Handle Event Pattern - Event handling patterns, Group distribution, child-to-parent communication
4. Menu Bar Implementation - Creating menus, items, shortcuts, cascading submenus
5. Status Line Implementation - Items, keyboard shortcuts, mouse support, hint text
6. Message Boxes - All dialog types, flags, examples
7. Command Enable/Disable - Global command set, automatic button updates, broadcasting
8. Architecture Patterns - Modal loops, event flow, focus management, state flags
9. Complete Example - Full menu + status + dialogs example

**Use when**: You need to understand how to implement a feature or need code examples

---

### 3. KEY_FILES_SUMMARY.txt (8 KB)
**File organization and locations**

Contains:
- File-by-file breakdown by feature
- Source file paths with line counts
- Example programs organized by feature
- Architecture patterns
- Key differences from Pascal

**Use when**: You need to find where a feature is implemented or which example to study

---

### 4. QUICK_REFERENCE.txt (9 KB)
**Quick code snippets and patterns**

Contains ready-to-use code for:
- Creating basic applications
- Creating menu bars
- Creating status lines
- Showing message boxes
- Creating buttons
- Enabling/disabling commands
- Creating custom dialogs
- Custom event handling
- Event type checking
- Command constants reference
- Key code constants reference
- Rectangle and point operations
- Modal vs modeless dialogs

**Use when**: You need a code snippet to copy or want a quick lookup

---

## The 7 Core Features

### 1. Events and Commands
- **Files**: `src/core/event.rs`, `src/core/command.rs`
- **Learn from**: `examples/event_debug.rs`
- **Key insight**: Event structure unified for all input types

### 2. Handle Event Implementation
- **Files**: `src/views/view.rs`, `src/views/button.rs`, `src/views/group.rs`
- **Learn from**: `examples/menu.rs`
- **Key insight**: Event transformation replaces parent pointers

### 3. Menu Bar
- **Files**: `src/views/menu_bar.rs`, `src/core/menu_data.rs`
- **Learn from**: `examples/menu.rs` (BEST EXAMPLE)
- **Key insight**: Submenus and cascading support

### 4. Status Line
- **Files**: `src/views/status_line.rs`
- **Learn from**: `examples/menu.rs`, `examples/status_line_demo.rs`
- **Key insight**: Keyboard shortcuts and mouse support

### 5. Message Boxes
- **Files**: `src/views/msgbox.rs`
- **Learn from**: `examples/dialogs_demo.rs`
- **Key insight**: Functions, not classes; returns CommandId or Option<String>

### 6. Command Enable/Disable
- **Files**: `src/core/command_set.rs`
- **Learn from**: `examples/command_set_demo.rs`
- **Key insight**: Automatic button updates via broadcasting

### 7. Modal Loop
- **Files**: `src/views/group.rs`, `src/views/dialog.rs`, `src/app/application.rs`
- **Learn from**: `examples/menu.rs`
- **Key insight**: Two execution patterns (direct and centralized)

---

## Reading Guide

### For Complete Learning (Recommended)
1. **Day 1**: Read FINDINGS_SUMMARY.md (15 min)
2. **Day 2**: Study examples/menu.rs (30 min)
3. **Day 3**: Read relevant sections of RUST_IMPLEMENTATION_REFERENCE.md (1 hour)
4. **Day 4**: Keep QUICK_REFERENCE.txt handy and try examples (ongoing)

### For Quick Implementation
1. Find your task in QUICK_REFERENCE.txt
2. Copy the code snippet
3. Adapt to your needs
4. Reference RUST_IMPLEMENTATION_REFERENCE.md if you need more details

### For Converting Documentation
1. Read FINDINGS_SUMMARY.md (understand Rust vs Pascal differences)
2. Use KEY_FILES_SUMMARY.txt (find implementation details)
3. Reference RUST_IMPLEMENTATION_REFERENCE.md (check architecture and examples)
4. Use examples in QUICK_REFERENCE.txt for code samples

---

## Key Architectural Patterns

### Event Flow
```
Application::handle_event()
  -> MenuBar::handle_event()
  -> Desktop::handle_event()
       -> Window::handle_event()
            -> Group::handle_event()
                 -> Children (reverse order)
  -> StatusLine::handle_event()
```

### Event Handling Patterns
```rust
// Pattern 1: Consume event
event.clear();

// Pattern 2: Transform to command
*event = Event::command(self.command);

// Pattern 3: Bubble up
// (do nothing with event)
```

### Command Enable/Disable
```
1. disable_command(CM_X) 
2. Button constructor checks command_set::command_enabled()
3. Application::idle() broadcasts CM_COMMAND_SET_CHANGED
4. Button::handle_event() receives broadcast
5. Button updates disabled state automatically
```

---

## Rust vs Borland Differences

| Feature | Borland | Rust |
|---------|---------|------|
| Parent Communication | `message(owner, ...)` | Event modification |
| Event Handler | `virtual handleEvent()` | Trait `fn handle_event()` |
| Menu Initialization | `initMenuBar()` method | `app.set_menu_bar(...)` |
| Status Line Init | `initStatusLine()` method | `app.set_status_line(...)` |
| Message Boxes | Classes | Functions |
| Global State | Static `curCommandSet` | Thread-local `GLOBAL_COMMAND_SET` |
| Modal Loop | `TGroup::execute()` | `Group::execute()` |

---

## Essential Code Patterns

### Create Menu Bar
```rust
let mut menu_bar = MenuBar::new(Rect::new(0, 0, width as i16, 1));
let items = vec![MenuItem::with_shortcut("~N~ew", CM_NEW, 0, "Ctrl+N", 0)];
menu_bar.add_submenu(SubMenu::new("~F~ile", Menu::from_items(items)));
app.set_menu_bar(menu_bar);
```

### Create Status Line
```rust
let status_line = StatusLine::new(
    Rect::new(0, height as i16 - 1, width as i16, height as i16),
    vec![StatusItem::new("~F10~ Menu", KB_F10, CM_QUIT)],
);
app.set_status_line(status_line);
```

### Show Message Box
```rust
message_box_ok(&mut app, "File saved!");
let result = confirmation_box(&mut app, "Save changes?");
if let Some(name) = input_box(&mut app, "Name", "Enter:", "", 50) {
    // Use name
}
```

### Enable/Disable Commands
```rust
command_set::disable_command(CM_COPY);
command_set::enable_command(CM_COPY);
if command_set::command_enabled(CM_PASTE) { /* can paste */ }
```

### Handle Event
```rust
fn handle_event(&mut self, event: &mut Event) {
    match event.what {
        EventType::Keyboard if event.key_code == KB_ENTER => {
            event.clear();
        }
        EventType::Broadcast if event.command == CM_COMMAND_SET_CHANGED => {
            self.update();
        }
        _ => {}
    }
}
```

---

## File Locations

All source files are in the `/Users/enzo/Code/turbo-vision/` directory:

**Core Event & Command System**:
- `src/core/event.rs` - Events
- `src/core/command.rs` - Command IDs
- `src/core/command_set.rs` - Enable/disable system

**View Implementations**:
- `src/views/view.rs` - View trait with handle_event()
- `src/views/button.rs` - Button event handling
- `src/views/group.rs` - Group event distribution
- `src/views/dialog.rs` - Dialog modal execution
- `src/views/menu_bar.rs` - Menu bar
- `src/views/status_line.rs` - Status line
- `src/views/msgbox.rs` - Message boxes

**Application**:
- `src/app/application.rs` - Application event routing

**Best Examples to Study**:
- `examples/menu.rs` - BEST EXAMPLE (all features)
- `examples/command_set_demo.rs` - Command enable/disable
- `examples/dialogs_demo.rs` - Message boxes
- `examples/event_debug.rs` - Event inspection

---

## Questions and Answers

**Q: How do children communicate with parents?**
A: By transforming events: `*event = Event::command(cmd)`. The parent receives this command event from its `handle_event()` call.

**Q: Where is the command enable/disable implemented?**
A: Global thread-local `CommandSet` in `src/core/command_set.rs`. Buttons check this on creation and respond to `CM_COMMAND_SET_CHANGED` broadcast.

**Q: What are the two dialog execution patterns?**
A: 1) Direct: `dialog.execute(&mut app)` - simpler, dialog runs its own loop. 2) Centralized: `app.exec_view(dialog)` - app manages loop (Borland-style).

**Q: How do I add keyboard shortcuts to menu items?**
A: Use tilde syntax: `"~N~ew"` creates Alt+N shortcut with N highlighted.

**Q: How do message boxes work?**
A: Functions that create dialogs and run modal loops: `message_box_ok()`, `confirmation_box()`, `input_box()`, etc. All return `CommandId` or `Option<String>`.

---

## Getting Started

1. **First Time?** Read FINDINGS_SUMMARY.md
2. **Need Code?** Check QUICK_REFERENCE.txt
3. **Full Details?** Read RUST_IMPLEMENTATION_REFERENCE.md
4. **Find Files?** Use KEY_FILES_SUMMARY.txt
5. **Study Code?** Look at examples/menu.rs

---

## Version Information

- **Analysis Date**: November 5, 2025
- **Turbo Vision Version**: Current Rust implementation
- **Borland Reference**: Matched against tgroup.cc, tview.cc, tmenubar.cc, etc.
- **Documentation Quality**: Comprehensive with code examples
- **Coverage**: All 7 core features analyzed

---

## Support Resources

All code examples are from actual source files in the repository:
- `/Users/enzo/Code/turbo-vision/src/` - Source implementation
- `/Users/enzo/Code/turbo-vision/examples/` - Example programs

You can compile and run examples to see features in action.
