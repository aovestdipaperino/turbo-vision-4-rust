# Turbo Vision Rust API Catalog - Index

**Generated:** 2025-11-06  
**Exploration Level:** Very Thorough  
**Status:** Complete

---

## Overview

This comprehensive API catalog documents all public structs, traits, and methods in the Turbo Vision Rust codebase (68 Rust files, 90+ public structs, 5 major traits, 500+ public methods).

---

## Documents Included

### 1. **RUST_API_CATALOG.md** (Primary Document)
Complete detailed reference with:
- All public structs with field descriptions
- All public traits with method signatures
- All public methods with parameters and return types
- Builder patterns and design patterns
- Trait inheritance and implementation information
- Color constants and command definitions
- Type aliases and special types

**Sections:**
- Core Module (geometry, colors, drawing, events, validators, menus, history, syntax highlighting)
- Terminal Module (Terminal abstraction layer)
- Views Module (50+ UI component structs)
- Application Module (Application structure)

### 2. **API_EXPLORATION_SUMMARY.txt** (Quick Reference)
Executive summary with:
- High-level API structure overview
- Component categorization (6 categories)
- Trait hierarchy and patterns
- Design principles
- File statistics
- Key findings

---

## API Structure Quick Reference

### Core Modules

#### Geometry (`core/geometry.rs`)
- **Point** - 2D coordinates
- **Rect** - Rectangle with geometric operations

#### Colors (`core/palette.rs`)
- **TvColor** - 16-color enum (0-15)
- **Attr** - Foreground/background color pairs
- **colors module** - 30+ color constants

#### Drawing (`core/draw.rs`)
- **Cell** - Character + attributes
- **DrawBuffer** - Line-based drawing

#### Events (`core/event.rs`)
- **Event** - Unified event structure
- **EventType** - Event classification
- **KeyCode** - Keyboard codes (u16)
- **MouseEvent** - Mouse coordinates & buttons
- **EscSequenceTracker** - Alt key handling

#### Validation (`views/validator.rs`)
- **Validator** trait - Base validation interface
- **FilterValidator** - Character set validation
- **RangeValidator** - Numeric range validation
- **LookupValidator** - Value lookup validation
- **PictureValidator** - Format mask validation

#### Commands (`core/command.rs`)
- **CommandId** - u16 command identifiers
- 50+ predefined commands

#### Menus (`core/menu_data.rs`)
- **MenuItem** - Regular/SubMenu/Separator
- **Menu** - Menu container
- **MenuBuilder** - Builder pattern

#### Status Line (`core/status_data.rs`)
- **StatusItem** - Item with key + command
- **StatusLine** - Status line container
- **StatusLineBuilder** - Builder pattern

#### History (`core/history.rs`)
- **HistoryList** - Per-ID history storage
- **HistoryManager** - Global history manager

#### Syntax Highlighting (`views/syntax.rs`)
- **SyntaxHighlighter** trait - Highlighting interface
- **TokenType** - 12 token classifications
- **Token** - Token span (start, end, type)
- **PlainTextHighlighter** - No highlighting
- **RustHighlighter** - Rust syntax support

### Terminal Module

#### Terminal (`terminal/mod.rs`)
17 major public methods:
- **init()**, **shutdown()** - Lifecycle
- **size()** - Query terminal size
- **write_cell()**, **write_line()** - Rendering
- **clear()**, **flush()** - Buffer management
- **show_cursor()**, **hide_cursor()** - Cursor control
- **push_clip()**, **pop_clip()** - Clipping regions
- **poll_event()**, **read_event()**, **put_event()** - Event handling
- **dump_screen()**, **dump_region()**, **flash()** - Debugging

### View System (Core UI Framework)

#### View Trait (`views/view.rs`)
All UI components implement this trait:
- **Required:** bounds(), set_bounds(), draw(), handle_event()
- **Optional:** 16+ methods with defaults
- **Categories:** Focus, State, Shadow, Cursor, Special, Modal, ListBox, Movement, Debugging

#### ListViewer Trait (`views/list_viewer.rs`)
For list-based components:
- **ListViewerState** - Shared state struct
- **Methods:** set_range(), focus_item(), focus_next(), focus_page_down(), etc.

### UI Components (50+ Structs)

**Simple Widgets (6):**
- Button, Label, StaticText, CheckBox, RadioButton, Indicator

**Containers (6):**
- Group, Window, Dialog, Desktop, Frame, Background

**List Views (6):**
- ListBox, SortedListBox, FileList, HistoryViewer, DirListBox, ListViewer trait

**Text Editing (5):**
- InputLine, Memo, Editor, FileEditor, EditWindow

**Scroll Controls (2):**
- ScrollBar, Scroller

**Menus (3):**
- MenuBar, MenuBox, MenuViewer

**Dialogs & Specialized (8):**
- FileDialog, HelpViewer, HelpWindow, TextViewer, StatusLine, ParamText, Cluster, History

**Validators (4):**
- FilterValidator, RangeValidator, LookupValidator, PictureValidator

**Help System (2):**
- HelpFile, HelpTopic

**File System (1):**
- FileEntry

### Application Module

#### Application (`app/application.rs`)
- **Fields:** terminal, menu_bar, status_line, desktop, running
- **Methods:** new(), set_menu_bar(), set_status_line(), run(), quit(), shutdown()

---

## Key Design Patterns

### 1. Builder Pattern
- **ButtonBuilder** → Button
- **WindowBuilder** → Window
- **MenuBuilder** → Menu
- **StatusLineBuilder** → StatusLine

### 2. Trait-Based Polymorphism
- **View trait** - Base for all UI components
- **Validator trait** - Input validation
- **SyntaxHighlighter trait** - Code highlighting
- **Cluster trait** - Radio button groups
- **ListViewer trait** - List-based views

### 3. Composition Over Inheritance
- Group contains View children
- ListBox embeds ListViewerState
- Window is Group with title/frame
- Desktop manages Windows

### 4. Event Propagation
- Child handles → transforms to command → bubbles up
- Group processes → Application dispatches

### 5. Reference Counting
- **ValidatorRef = Rc<RefCell<dyn Validator>>>**
- Multiple owners of shared validator state

---

## Type Aliases & Constants

### Type Aliases
- `CommandId = u16` - Command identifiers
- `KeyCode = u16` - Keyboard codes (high byte: scan, low byte: char)
- `StateFlags = u16` - State flag bits
- `ValidatorRef = Rc<RefCell<dyn Validator>>` - Shared validator

### Major Constants Groups

**Key Codes (40+ constants):**
- KB_ESC, KB_ENTER, KB_BACKSPACE, KB_TAB, KB_F1-F12, KB_UP/DOWN/LEFT/RIGHT, KB_HOME/END, etc.

**Event Masks (8 constants):**
- EV_MOUSE_DOWN, EV_MOUSE_UP, EV_KEYBOARD, EV_COMMAND, EV_BROADCAST, etc.

**Mouse Button Masks (3 constants):**
- MB_LEFT_BUTTON, MB_MIDDLE_BUTTON, MB_RIGHT_BUTTON

**Color Constants (30+ constants):**
- NORMAL, BUTTON_NORMAL, MENU_NORMAL, LISTBOX_SELECTED, etc.

**Validator Option Flags (3 constants):**
- VO_FILL, VO_TRANSFER, VO_ON_APPEND

**Commands (50+ constants):**
- Standard: CM_QUIT, CM_CLOSE, CM_OK, CM_CANCEL
- File: CM_NEW, CM_OPEN, CM_SAVE, CM_SAVE_AS
- Edit: CM_UNDO, CM_REDO, CM_CUT, CM_COPY
- etc.

---

## Architecture Overview

```
Terminal (crossterm backend)
    ↓
Application (event loop coordinator)
    ├→ MenuBar (View trait)
    ├→ Desktop (View trait - manages windows)
    │   └→ Window (View trait - top-level window)
    │       └→ Group (View trait - child container)
    │           ├→ Button, Label, CheckBox... (simple widgets)
    │           ├→ ListBox, FileList... (list views)
    │           ├→ Editor, Memo... (text editing)
    │           ├→ Dialog, Frame... (containers)
    │           └→ ... (50+ UI components)
    └→ StatusLine (View trait)

Event System:
  Keyboard/Mouse Events → Terminal.poll_event()
    → Application.run() processes
    → Delegates to View.handle_event()
    → Event bubbles up through container hierarchy
    → Application.run() processes command

Validation System:
  InputLine uses Validator trait
    ├→ FilterValidator (allowed characters)
    ├→ RangeValidator (numeric range)
    ├→ LookupValidator (value in list)
    └→ PictureValidator (format mask)

Highlighting System:
  Editor uses SyntaxHighlighter trait
    ├→ PlainTextHighlighter (no highlighting)
    └→ RustHighlighter (Rust syntax)
```

---

## File Location & Access

All generated files are in the project root:

```
/Users/enzolombardi/Code/tv/
├── RUST_API_CATALOG.md           (44 KB - Full detailed reference)
├── API_EXPLORATION_SUMMARY.txt   (12 KB - Quick summary)
└── API_CATALOG_INDEX.md          (This file - Navigation guide)
```

---

## Usage Examples

### Finding Information

**Looking for Button API?**
→ See RUST_API_CATALOG.md → Views Module → Button section

**Quick overview of validators?**
→ See API_EXPLORATION_SUMMARY.txt → VALIDATION & INPUT section

**Need to understand View trait?**
→ See RUST_API_CATALOG.md → Views Module → View Trait section

**Looking for all commands?**
→ See RUST_API_CATALOG.md → Core Module → Command System

**Want builder pattern examples?**
→ See RUST_API_CATALOG.md → Design Patterns → Builder Pattern

---

## Statistics

| Metric | Count |
|--------|-------|
| Rust Files Analyzed | 68 |
| Public Structs | 90+ |
| Public Traits | 5 major |
| Public Methods | 500+ |
| Enums | 15+ |
| Type Aliases | 4+ |
| Command Constants | 50+ |
| Color Constants | 30+ |
| Key Code Constants | 40+ |

---

## Document Versions

- **RUST_API_CATALOG.md** - Full detailed reference (primary document)
  - 1,302 lines
  - Complete method signatures
  - All field descriptions
  - Design patterns explained
  - Examples provided

- **API_EXPLORATION_SUMMARY.txt** - Executive summary
  - Text format
  - High-level overview
  - Statistics and findings
  - Quick reference

- **API_CATALOG_INDEX.md** - This document
  - Navigation guide
  - Quick reference tables
  - Architecture diagram
  - Document cross-references

---

## How to Navigate

1. **For Complete Details:** Start with RUST_API_CATALOG.md
   - Use Ctrl+F to search for component names
   - Look for section headers to find modules
   - Check Design Patterns section for patterns

2. **For Quick Overview:** Start with API_EXPLORATION_SUMMARY.txt
   - Read findings overview
   - Check component categories
   - Review trait hierarchy

3. **For Navigation:** Use this document (API_CATALOG_INDEX.md)
   - Check Quick Reference section
   - Look for component names
   - Use cross-references to main document

---

## Generated Information

**Exploration Date:** 2025-11-06  
**Exploration Type:** Very Thorough (comprehensive)  
**Codebase:** Turbo Vision Rust (src/ directory)  
**Generator:** Claude Code with extensive grep/read analysis

---

