# Turbo Vision - Current Status and TODO

**Last Updated**: 2025-11-03 (v0.2.6)
**Total Tests**: 171 tests (all passing)
**Total Lines**: 15,845 lines (12,134 code)

This document consolidates all TODO tracking files into a single comprehensive status report and roadmap.

---

## Table of Contents

1. [Current Implementation Status](#current-implementation-status)
2. [Completed Features by Version](#completed-features-by-version)
3. [Missing Features Inventory](#missing-features-inventory)
4. [Next Release Roadmap](#next-release-roadmap)
5. [Architecture Status](#architecture-status)
6. [Statistics and Progress](#statistics-and-progress)

---

## Current Implementation Status

### ✅ Fully Implemented Components (v0.2.6)

#### **Core Views & Controls**
- ✅ **TView** - Base view class (View trait)
- ✅ **TGroup** - Container for child views (Group)
- ✅ **TWindow** - Window with frame (Window)
- ✅ **TDialog** - Modal dialog (Dialog)
- ✅ **TFrame** - Window frame with title and close button
- ✅ **TButton** - Push button with mouse/keyboard support
- ✅ **TCheckBoxes** - Checkbox control (CheckBox)
- ✅ **TRadioButtons** - Radio button control (RadioButton)
- ✅ **TInputLine** - Single-line text input (InputLine)
- ✅ **TStaticText** - Static text label with centering support
- ✅ **TLabel** - Text label with keyboard shortcuts
- ✅ **TListBox** - List selection control (ListBox)
- ✅ **TSortedListBox** - Sorted list with binary search (v0.2.2, 415 lines, 8 tests)
- ✅ **TCluster** - Base class for check/radio groups (Cluster trait, v0.2.2)

#### **Editor Components**
- ✅ **TEditor** - Full text editor with search/replace/file I/O (v0.2.2, v0.2.3, v0.2.6, 1304 lines)
  - Search functionality (find, find_next)
  - Replace functionality (replace_selection, replace_next, replace_all)
  - File I/O operations (load_file, save_file, save_as)
  - Comprehensive undo/redo system
  - Block selection and clipboard operations (with OS clipboard)
  - ✅ **Syntax highlighting** (v0.2.6) - Token-based coloring with RustHighlighter
- ✅ **TMemo** - Multi-line text input (Memo, 911 lines)
- ✅ **TEditWindow** - Window wrapper for editor (v0.2.3, 169 lines, 3 tests)

#### **List & Menu Infrastructure**
- ✅ **TListViewer** - Base list viewer (ListViewer trait, v0.2.2)
- ✅ **TMenuView** - Base menu view (MenuViewer trait, v0.2.2)
- ✅ **TMenuBar** - Top menu bar with mouse support
- ✅ **TMenuBox** - Popup menu container (v0.2.2)
- ✅ **TMenuItem** - Menu item with keyboard shortcut display
- ✅ **TSubMenu** - Submenu container

#### **File System Components**
- ✅ **TFileDialog** - File open/save dialog (fully functional with mouse/keyboard)
- ✅ **TFileList** - File browser list (v0.2.2, 415 lines, 4 tests)
- ✅ **TDirListBox** - Directory tree view (v0.2.2, 398 lines, 4 tests)

#### **History System**
- ✅ **THistory** - History dropdown button (v0.2.2)
- ✅ **THistoryViewer** - History list viewer (v0.2.2)
- ✅ **THistoryWindow** - History popup window (v0.2.2)
- ✅ **HistoryManager** - Global history management (v0.2.2, 14 tests)

#### **Help System**
- ✅ **THelpFile** - Markdown help file manager (v0.2.5, 302 lines, 7 tests)
- ✅ **THelpViewer** - Help content viewer (v0.2.5, 286 lines, 4 tests)
- ✅ **THelpWindow** - Help display window (v0.2.5, 157 lines, 4 tests)
- ✅ **HelpContext** - Context-sensitive help mapping (v0.2.5, 122 lines, 7 tests)

#### **Validation System**
- ✅ **TValidator** - Base validator trait (v0.2.1)
- ✅ **TFilterValidator** - Character filter validation (v0.2.1)
- ✅ **TRangeValidator** - Range validation with hex/octal support (v0.2.1)
- ✅ **TPXPictureValidator** - Picture/mask validation (v0.2.6, phone numbers, dates)
- ✅ **TLookupValidator** - List validation (v0.2.3, 255 lines, 8 tests)

#### **Application Framework**
- ✅ **TApplication** - Main application class (Application, 310 lines)
- ✅ **TDesktop** - Desktop/workspace (Desktop)
- ✅ **TBackground** - Desktop background pattern
- ✅ **TStatusLine** - Bottom status line with hot spots and hints (v0.1.8)

#### **Utilities**
- ✅ **TScrollBar** - Scrollbar (vertical/horizontal)
- ✅ **TScroller** - Scrollable view base class
- ✅ **TTextDevice** - Text viewer (TextView)
- ✅ **TIndicator** - Position display widget
- ✅ **TParamText** - Parameterized text display
- ✅ **TMessageBox** - Message box utility (message_box())
- ✅ **TInputBox** - Simple input box (input_box())

#### **System Integration**
- ✅ **OSClipboard** - System clipboard integration (v0.2.3, arboard crate)
- ✅ **Terminal** - Terminal abstraction (458 lines, combines TScreen/TDisplay/TMouse/TEventQueue)

#### **Event System**
- ✅ **Three-Phase Event Processing** (v0.1.9) - PreProcess/Focused/PostProcess
- ✅ **Event Re-queuing** (v0.1.10) - put_event() capability
- ✅ **Owner-Aware Broadcasts** (v0.2.0) - Prevents echo back to sender

---

### ❌ Not Yet Implemented

#### **Window Management**
- ❌ **Window Minimize** - Iconify to desktop bottom
- ❌ **Window Maximize** - Full screen toggle
- ❌ **Window State Management** - Normal/minimized/maximized states

#### **File System Enhancements**
- ❌ **TFileInfoPane** - File info display (size, date, attributes)
- ❌ **TFileInputLine** - File path input with completion
- ❌ **TChDirDialog** - Change directory dialog

#### **Color System**
- ❌ **TColorDialog** - Color customization dialog
- ❌ **TColorSelector** - Color picker control
- ❌ **TMonoSelector** - Monochrome attribute selector
- ❌ **TColorDisplay** - Color preview display
- ❌ **Color Themes** - Runtime palette changes and theme loading

#### **Advanced Features**
- ❌ **TCalculator** - Calculator dialog
- ❌ **TCalcDisplay** - Calculator display
- ❌ **Application History** - Alt+0 history list for windows
- ❌ **Global Keyboard Shortcuts** - App-level key routing (shortcuts are display-only)

#### **Not Needed (Use Rust stdlib)**
- ~~TCollection~~ - Use `Vec<T>`
- ~~TSortedCollection~~ - Use `Vec<T>` + sort/binary_search
- ~~TStringCollection~~ - Use `Vec<String>`
- ~~TResourceFile~~ - Use JSON/TOML/serde
- ~~TStreamable~~ - Use serde Serialize/Deserialize

---

## Completed Features by Version

### v0.2.6 (2025-11-03) - Syntax Highlighting & Validation Completion
**Theme**: Editor enhancements and form validation polish

**Major Features**:
- ✅ **Syntax Highlighting System** - Token-based coloring with SyntaxHighlighter trait
  - RustHighlighter with keyword, string, comment, number, type highlighting
  - Extensible architecture for adding new languages
  - Integrated with Editor component
- ✅ **PictureValidator** (TPXPictureValidator) - Format mask validation
  - Phone numbers: `(###) ###-####`
  - Dates: `##/##/####`
  - Custom patterns with `#` (digit), `@` (letter), `!` (any char)
  - Automatic literal insertion (parentheses, slashes, etc.)

**Bug Fixes**:
- ✅ **Menu Dropdown Mouse Clicks** - Fixed bounds checking for dropdown items
  - Issue: Clicks on dropdown menu items weren't registering
  - Root cause: MenuBar bounds check only included row 0, but dropdowns at rows 2+
  - Solution: Custom dropdown bounds calculation in handle_event
- ✅ **Group Broadcast Event Routing** - Restructured for clarity
  - Nested broadcast check inside final else block
  - Prevents any interference with mouse event routing
- ✅ **Example Coordinate Systems** - Fixed absolute vs relative positioning
  - editor_demo.rs and validator_demo.rs now use relative dialog coordinates
- ✅ **Menu API Updates** - Updated all examples to current Menu API
  - menu.rs, status_line_demo.rs, window_resize_demo.rs
  - Changed from add_menu() to add_submenu()
  - Updated to declarative Menu::from_items() pattern

**New Examples**:
- ✅ **validator_demo.rs** - Comprehensive validation demonstrations
- ✅ **editor_demo.rs** - Editor features (basic editing, search, syntax highlighting, file I/O)
- ✅ **event_debug.rs** - Event capture diagnostic tool

**Total**: ~600 lines added, 171 tests (all passing)

### v0.2.5 (2025-11-03) - Help System
**Theme**: Context-sensitive help with modern markdown format

**Major Features**:
- ✅ **HelpFile** (302 lines, 7 tests) - Markdown parser with `# Title {#topic-id}` format
- ✅ **HelpViewer** (286 lines, 4 tests) - Scrollable help content display
- ✅ **HelpWindow** (157 lines, 4 tests) - Modal help window
- ✅ **HelpContext** (122 lines, 7 tests) - Context-sensitive help mapping

**Key Advantages**:
- Human-readable markdown instead of binary TPH files
- Version control friendly (plain text diffs)
- Cross-reference support via `[Text](#topic-id)`
- Keyboard navigation (arrows, PgUp/PgDn, Home/End)

**Total**: 867 lines, 22 tests

### v0.2.3 (2025-11-03) - Editor Polish & Clipboard
**Theme**: Professional text editing and OS integration

**Major Features**:
- ✅ **TEditWindow** (169 lines, 3 tests) - Ready-to-use editor window
- ✅ **TLookupValidator** (255 lines, 8 tests) - List validation
- ✅ **OS Clipboard Integration** - arboard crate for cross-platform clipboard

**Enhancements**:
- Editor file operations (load_file, save_file, save_as)
- Clipboard integration (Ctrl+C, Ctrl+X, Ctrl+V)
- Validator system completion

### v0.2.2 (2025-11-03) - Core Infrastructure
**Theme**: Foundation for professional applications

**Major Features** (Phase 2-6 completed):
- ✅ **List Infrastructure** (Phase 2) - ListViewer/MenuViewer traits, MenuBox
- ✅ **TCluster** (Phase 3) - Base for RadioButton/CheckBox
- ✅ **TSortedListBox** (Phase 4) - Binary search sorted lists
- ✅ **History System** (Phase 5) - History dropdowns for input fields
- ✅ **File System** (Phase 6) - FileList and DirListBox components

**Bug Fixes**:
- Editor UTF-8 support (character vs byte indices)
- Editor cursor rendering (two-cursor display bug)
- ScrollBar division by zero crash

**Total**: ~1,800 lines across 9 new components

### v0.2.1 (2025-11-03) - Input Validation
**Theme**: Form validation system

**Major Features**:
- ✅ **Validator trait** - Base validation interface
- ✅ **FilterValidator** - Character filtering (e.g., digits only)
- ✅ **RangeValidator** - Numeric range validation with hex/octal support

**Integration**:
- InputLine validator attachment (with_validator, set_validator)
- Real-time validation (is_valid_input)
- Final validation (is_valid)

### v0.2.0 (2025-11-03) - Event System Completion
**Theme**: Borland-compatible event architecture

**Major Features**:
- ✅ **Owner-Aware Broadcasts** - Group::broadcast() with owner parameter

**Bug Fixes**:
- Menu example OK button (CM_OK instead of 0)

### v0.1.10 (2025-11-03) - Event Re-queuing
**Theme**: Deferred event processing

**Major Features**:
- ✅ **put_event()** - Re-queue events for next iteration
- ✅ **pending_event** - FIFO event queue

**Enables**:
- Command generation patterns
- Deferred event processing
- Modal dialog patterns

### v0.1.9 (2025-11-03) - Three-Phase Event Processing
**Theme**: Borland event architecture

**Major Features**:
- ✅ **PreProcess Phase** - Views with OF_PRE_PROCESS flag
- ✅ **Focused Phase** - Currently focused view
- ✅ **PostProcess Phase** - Views with OF_POST_PROCESS flag

**Benefits**:
- Buttons intercept hotkeys when not focused
- StatusLine monitors all key presses
- Proper command routing patterns

### v0.1.8 (2025-11-03) - Status Line Enhancement
**Major Features**:
- ✅ **Status Line Hot Spots** - Mouse hover highlighting
- ✅ **Context Hints** - set_hint() for help text

**Colors**:
- STATUS_SELECTED: White on Green
- STATUS_SELECTED_SHORTCUT: Yellow on Green

### v0.1.7 (2025-11-03) - Menu Polish
**Major Features**:
- ✅ **Keyboard Shortcut Display** - Right-aligned shortcuts in menus
- ✅ **MenuItem::new_with_shortcut()** - Specify shortcut text

### v0.1.6 (2025-11-03) - Window Resize
**Major Features**:
- ✅ **Window Resize** - Drag bottom-right corner
- ✅ **Minimum Size Constraints** - 16x6 minimum

### v0.1.5 (2025-11-03) - Double-Click
**Major Features**:
- ✅ **Double-Click Detection** - 500ms window, same position
- ✅ **ListBox Integration** - Double-click to select

### v0.1.4 (2025-11-02) - Modal Architecture Refactor
**Theme**: Borland-compatible modal execution

**Major Changes**:
- ✅ **Group-based modal execution** - execute() in Group, not Dialog
- ✅ **Event loop in Group** - Matches Borland's TGroup::execute()
- ✅ **Efficient drawing** - Union rect pattern for window movement

**Bug Fixes**:
- Modal dialog hang bugs
- Window movement trails

### v0.1.3 (2025-11-02) - Mouse Enhancements
**Major Features**:
- ✅ **Scroll Wheel Support** - MouseWheelUp/MouseWheelDown events
- ✅ **Window Closing** - Non-modal windows close properly

### v0.1.2 (2025-11-02) - Window Management
**Major Features**:
- ✅ **Z-Order Management** - Click to bring to front
- ✅ **Modal Window Support** - Block background interaction
- ✅ **Menu Borders & Shadows** - Borland-accurate styling

### v0.1.1 (2025-11-02) - Bug Fixes
**Bug Fixes**:
- Window dragging trails (desktop redraw)

### v0.1.0 (2025-11-02) - Initial Release
**Theme**: Core TUI framework

**Major Components**:
- Event-driven architecture
- Drawing system with 16-color palette
- Focus management with Tab navigation
- Modal dialog execution
- Basic controls (Button, InputLine, StaticText, CheckBox, RadioButton)
- Menu system (MenuBar with dropdowns)
- StatusLine
- Desktop manager
- ScrollBar and Scroller
- TextView
- ListBox
- Memo
- FileDialog

---

## Missing Features Inventory

*Based on Borland Turbo Vision source analysis (105 .cc files, 130+ headers)*

### Summary Statistics

- **Total Missing Components**: 25 (down from 35)
- **Estimated Total Effort**: 592 hours (~15 weeks at 40 hrs/week)
- **HIGH Priority**: 0 items (0 hours) - Core functionality COMPLETE ✅
- **MEDIUM Priority**: 23 items (274 hours) - Extended features
- **LOW Priority**: 2 items (318 hours) - Nice to have

### Quick Reference by Category

| Category | Count | Priority | Effort |
|----------|-------|----------|--------|
| Core Views/Controls | 0 | - | 0h |
| Specialized Dialogs | 13 | LOW-MEDIUM | 126h |
| Editor Components | 0 | - | 0h |
| File System | 3 | MEDIUM | 16h |
| Application Framework | 0 | - | 0h |
| System Utilities | 0 | - | 0h |
| Helper Classes | 0 | - | 0h |
| Advanced Features | 9 | MEDIUM-LOW | 450h |

### HIGH Priority Components (Core Functionality)

**ALL COMPLETE!** ✅

- ✅ Collections (Phase 1) - Use Rust Vec<T> instead
- ✅ Menu & Status Infrastructure (Phase 1) - COMPLETE (v0.2.2)
- ✅ List Components (Phase 2) - COMPLETE (v0.2.2)
- ✅ Input Controls (Phase 3-5) - COMPLETE (v0.2.2)
- ✅ File System (Phase 6) - COMPLETE (v0.2.2)
- ✅ Editor (Phase 7) - COMPLETE (v0.2.2, v0.2.3)
- ✅ Application Framework (Phase 8) - Already implemented
- ✅ Help System (Phase 9) - COMPLETE (v0.2.5)

### MEDIUM Priority Components (Extended Features)

#### File Dialog Enhancements (16 hours)
- **TFileInputLine** - File path input (6h)
- **TFileInfoPane** - File info display (6h)
- **TChDirDialog** - Change directory dialog (4h)

#### Application Enhancements (50 hours)
- **TDeskTop** - Enhanced desktop features (10h)
  - Cascade windows
  - Tile windows
  - Window management commands
- **TEditorApp** - Editor application framework (20h)
- **TDrawBuffer** - Drawing utilities (8h)
- **CodePage** - Character encoding (12h)

**Total MEDIUM Priority**: 66 hours

### LOW Priority Components (Nice to Have)

#### Color Customization Suite (66 hours)
- TColorDialog, TColorSelector, TMonoSelector (40h)
- TColorDisplay, TColorGroup, TColorItem (14h)
- TColorGroupList, TColorItemList (12h)

#### Calculator (24 hours)
- TCalculator dialog (16h)
- TCalcDisplay component (8h)

#### Advanced Validators (12 hours)
- **TPXPictureValidator** - Mask validation (12h)

#### Text Output (40 hours)
- **TTextDevice** - Text output base (12h)
- **TTerminal** - Terminal emulator (20h)
- **otstream** - Output text stream (8h)

#### Configuration (10 hours)
- **ConfigFile** - Configuration manager (10h)

**Total LOW Priority**: 152 hours

---

## Next Release Roadmap

### v0.3.0 Candidates (High-Value Features)

Based on user impact and effort, recommended features for v0.3.0:

#### Option A: Window Management (10-14 hours)
**Why**: Completes window functionality, high visibility

1. **Window Min/Max Buttons** (4-6 hours)
   - Add minimize button (iconify window to desktop bottom)
   - Add maximize button (full screen toggle)
   - Window state management (normal, minimized, maximized)
   - Double-click title bar to toggle maximize

2. **Application History** (3-4 hours)
   - Alt+0 shows history list
   - Track viewed windows/dialogs
   - MRU (Most Recently Used) list
   - Quick switching between windows

3. **File Dialog Enhancements** (3-4 hours)
   - TFileInputLine with auto-completion
   - TFileInfoPane showing file details

**Total**: 10-14 hours, 3 features

#### Option B: Color Themes (6-10 hours)
**Why**: User customization, personalization

1. **Color Themes** (6-8 hours)
   - Runtime palette changes
   - Load/save color themes
   - Define theme file format (JSON/TOML)
   - Popular themes: Borland Blue, Monokai, Nord

2. **Theme Picker Dialog** (2-3 hours)
   - Simple dialog to select theme
   - Live preview

**Total**: 8-11 hours, 2 features

#### Option C: Editor Enhancements (12-15 hours)
**Why**: Improves text editing experience

1. **Syntax Highlighting** (12+ hours)
   - Hook system for syntax rules
   - Basic language support (Rust, C, Python, etc.)
   - Color-coded tokens

**Total**: 12+ hours, 1 major feature

### Recommended Order for v0.3.0

**Priority 1: Window Management** (10-14 hours)
- High visibility, completes core window functionality
- Builds on existing resize/drag/close functionality
- Classic Borland features users expect

**Priority 2: Color Themes** (6-10 hours)
- User-requested customization
- Not architecturally complex
- Quick win for personalization

**Priority 3: File Dialog Polish** (3-4 hours)
- Enhances already-strong file dialog
- Professional finishing touches

**Total v0.3.0**: 19-28 hours

**Deferred to v0.4.0**:
- Syntax Highlighting (major undertaking)
- Calculator (nice-to-have utility)
- Color Customization Suite (low demand)

---

## Architecture Status

### ✅ Completed Architecture Patterns

#### Event System (v0.1.9, v0.1.10, v0.2.0)
**Status**: Fully matches Borland's architecture ✅

1. **Three-Phase Event Processing** (v0.1.9)
   - Phase 1 (PreProcess): Views with OF_PRE_PROCESS flag
   - Phase 2 (Focused): Currently focused view
   - Phase 3 (PostProcess): Views with OF_POST_PROCESS flag
   - Enables proper command routing and interception

2. **Event Re-queuing** (v0.1.10)
   - put_event() method to re-queue events
   - pending_event field for FIFO queue
   - Enables command generation patterns

3. **Owner-Aware Broadcasts** (v0.2.0)
   - broadcast(event, owner_index) prevents echo
   - Sophisticated command distribution
   - Focus-list navigation support

**Reference**: Borland's TGroup::handleEvent() (tgroup.cc:342-369)

#### Modal Execution (v0.1.4)
**Status**: Borland-compatible ✅

- Event loop in Group (not Dialog)
- end_modal() pattern to exit loops
- Modal dialogs set SF_MODAL flag
- Proper drawing with desktop redraw

**Reference**: Borland's TGroup::execute() (tgroup.cc:182-195)

#### Drawing System (v0.1.4)
**Status**: Optimized ✅

- Event-driven redraws (no per-frame drawing)
- Union rect pattern for window movement
- DrawBuffer for efficient rendering
- Z-order management with bring_to_front()

**Reference**: Borland's drawUnderRect pattern (tview.cc)

#### Validation System (v0.2.1)
**Status**: Trait-based ✅

- Validator trait with is_valid() and is_valid_input()
- FilterValidator for character filtering
- RangeValidator for numeric ranges
- LookupValidator for list validation
- InputLine integration

**Reference**: Borland's TValidator architecture (validate.h, tvalidat.cc)

### ❌ Architecture Gaps

**None for core functionality!** ✅

All critical Borland patterns are now implemented. Remaining features are incremental enhancements, not architectural requirements.

---

## Statistics and Progress

### Code Statistics (v0.2.6)

```
===============================================================================
 Language            Files        Lines         Code     Comments       Blanks
===============================================================================
 Rust                   62        15845        12134         1352         2359
 |- Markdown            55         1365            5         1194          166
 (Total)                          17210        12139         2546         2525
===============================================================================
 Total                  62        15845        12134         1352         2359
===============================================================================
```

*Generated with [tokei](https://github.com/XAMPPRocky/tokei)*

### Implementation Progress

**Phases Completed**: 9 out of 11
- ✅ Phase 1: Menu & Status Infrastructure (20 hours)
- ✅ Phase 2: List Components (38 hours)
- ✅ Phase 3: TCluster (8 hours)
- ✅ Phase 4: TSortedListBox (8 hours)
- ✅ Phase 5: History System (24 hours)
- ✅ Phase 6: File System Components (26 hours)
- ✅ Phase 7: Editor Enhancements (8 hours)
- ✅ Phase 8: Application Framework (already implemented)
- ✅ Phase 9: Help System (56 hours)
- 🚧 Phase 10: Window Management (pending, 10-14 hours)
- 🚧 Phase 11: Polish & Utilities (pending, ~150 hours)

**Cumulative Effort Completed**: ~188 hours of planned implementation
**Remaining Core Features**: ~45-50 hours (mostly polish)

### Test Coverage

**Total Tests**: 171 tests
**Passing**: 171 tests (100%) ✅
**Failing**: 0 tests

**Component Breakdown**:
- Help System: 22 tests
- Validators: 8 tests (FilterValidator, RangeValidator, LookupValidator, PictureValidator)
- Editor: 5 tests
- History: 14 tests
- File System: 8 tests (FileList, DirListBox)
- Cluster: 7 tests
- SortedListBox: 8 tests
- Core: ~99 tests (views, controls, events, drawing)

### Version History Summary

| Version | Date | Theme | Features | Lines | Tests |
|---------|------|-------|----------|-------|-------|
| v0.1.0 | 2025-11-02 | Initial Release | Core TUI framework | ~8,000 | 70 |
| v0.1.1-v0.1.8 | 2025-11-02-03 | Polish & Fixes | Window management, events | ~1,000 | +10 |
| v0.1.9-v0.1.10 | 2025-11-03 | Event Architecture | Three-phase, re-queuing | ~500 | +5 |
| v0.2.0 | 2025-11-03 | Event Completion | Owner-aware broadcasts | ~200 | +2 |
| v0.2.1 | 2025-11-03 | Validation | Validator system | ~800 | +8 |
| v0.2.2 | 2025-11-03 | Core Infrastructure | Lists, history, file system | ~2,800 | +35 |
| v0.2.3 | 2025-11-03 | Editor & Clipboard | EditWindow, OS clipboard | ~600 | +14 |
| v0.2.5 | 2025-11-03 | Help System | Markdown-based help | ~870 | +22 |
| v0.2.6 | 2025-11-03 | Syntax & Validation | Highlighting, PictureValidator | ~600 | +17 |
| **Total** | - | - | **58+ features** | **15,845** | **171** |

### Milestone Markers

- ✅ **After Phase 2** (58 hours): List and menu infrastructure solid
- ✅ **After Phase 3** (66 hours): Button group controls unified
- ✅ **After Phase 4** (74 hours): Sorted lists with binary search
- ✅ **After Phase 5** (98 hours): History system complete
- ✅ **After Phase 6** (124 hours): File system navigation
- ✅ **After Phase 7** (132 hours): Professional text editing
- ✅ **After Phase 8** (190 hours): Application framework complete
- ✅ **After Phase 9** (246 hours): Context-sensitive help ← **WE ARE HERE**
- 🚧 **After Phase 10** (~260 hours): Window management complete
- 🚧 **After Phase 11** (~410+ hours): Complete framework with utilities

---

## Quick Reference: What's Available

### When to Use What

**Basic Controls**:
- Button, CheckBox, RadioButton - User input
- InputLine - Text input (use with_validator() for validation)
- Label, StaticText - Display text
- ListBox - Select from list (use SortedListBox for searching)

**Advanced Controls**:
- Editor - Full text editing with search/replace/file I/O
- EditWindow - Ready-to-use editor window wrapper
- Memo - Simple multi-line text input
- TextView - Read-only scrollable text display
- FileList - File browser with wildcard filtering
- DirListBox - Directory tree navigation

**Dialogs**:
- Dialog - Custom modal dialogs
- FileDialog - File open/save with full navigation
- message_box() - Quick alerts/confirmations
- input_box() - Simple text input prompts

**Layout**:
- Group - Container for child views
- Window - Framed container with title bar (draggable, resizable)
- Desktop - Window manager
- ScrollBar - Vertical/horizontal scrolling

**Menus & Status**:
- MenuBar - Top menu with dropdowns
- StatusLine - Bottom status with hot spots

**Validation**:
- FilterValidator - Character filtering ("0123456789" for digits only)
- RangeValidator - Numeric range (0-100, -50 to 50, hex 0x00-0xFF)
- LookupValidator - List of valid values (case-sensitive or insensitive)

**History**:
- History - Dropdown showing previous inputs (attach to InputLine)

**Help**:
- HelpFile - Load markdown help files
- HelpWindow - Display help topics
- HelpContext - Map context IDs to topics (for F1 key)

**System**:
- Application - Main event loop
- Terminal - Screen management (usually accessed via Application)
- Clipboard - OS clipboard integration (get_clipboard, set_clipboard)

---

## Contributing

When adding new features:

1. **Reference Borland source** - Check `local-only/borland-tvision/` for patterns
2. **Write tests** - Aim for comprehensive test coverage
3. **Update documentation** - Keep this file and README.md current
4. **Follow patterns** - Use existing trait/struct patterns
5. **Consider Rust idioms** - Don't blindly port C++, use Rust strengths

### Development Priorities

1. **Core functionality first** - Complete Phase 10 (Window Management)
2. **User-requested features** - Color themes, syntax highlighting
3. **Polish existing features** - Before adding new ones
4. **Test coverage** - Maintain high test coverage
5. **Documentation** - Keep examples and docs current

---

*This document consolidates:*
- `local-only/REAL-TO-DO.md`
- `local-only/TO_DO_NEXT_MINOR.md`
- `docs/TO-DO-LIST.md`
- `docs/MISSING_FEATURES.md`
