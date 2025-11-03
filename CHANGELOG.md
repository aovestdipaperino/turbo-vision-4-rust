# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.2] - 2025-11-03

### Fixed
- **Editor UTF-8 Support**: Critical bug fixes for proper UTF-8 character handling
  - Fixed crash when pressing DELETE/BACKSPACE on multi-byte UTF-8 characters
  - Added `char_to_byte_idx()` helper to convert character positions to byte indices
  - Fixed `delete_char()`, `backspace()`, `insert_char()` to use byte indices for string operations
  - Fixed `apply_action()` undo/redo to handle UTF-8 correctly
  - Fixed `clamp_cursor()` to use character count instead of byte length
  - Fixed `get_selection_text()` to convert character positions to byte indices
  - Fixed `delete_selection_internal()` string slicing for UTF-8
  - Fixed `insert_text_internal()` to use byte indices for `insert_str()`
  - Fixed `insert_newline()` string slicing to use byte indices
  - Fixed `select_all()` to count characters not bytes
  - Fixed `max_line_length()` to count characters for scrollbar calculations
  - Fixed find operations to count characters for cursor positioning
  - Fixed KB_END key handler to use character count
  - Added safety checks to delete operations

- **Editor Cursor Rendering**: Fixed two-cursor display bug
  - Fixed `update_cursor()` to use `get_content_area()` instead of `bounds`
  - Cursor now correctly positioned when editor has scrollbars and indicator
  - Previously showed two cursors: one at correct position, one offset by indicator height

- **ScrollBar**: Fixed division by zero crash
  - Added validation in `set_params()` to ensure `max_val >= min_val`
  - Added safety check in `get_pos()` to handle `range <= 0` or `size <= 0`
  - Prevents crash when content becomes smaller than viewport
  - Prevents crash from invalid scrollbar parameters

### Added
- **full_editor example**: Comprehensive editor demonstration with search/replace
  - Shows editor with scrollbars, indicator, and sample text for testing
  - Sample text includes patterns for testing case-sensitive/whole-word search
  - Added panic logging to capture crashes with full backtrace to debug log

- **editor_test example**: Minimal editor test for debugging

### Technical Details
The Editor was incorrectly mixing character indices (used for cursor position tracking) with byte indices (required by Rust's `String::remove()` and `String::insert()` methods). In UTF-8 encoding:
- ASCII characters are 1 byte each
- Many Unicode characters (accented letters, emojis, CJK) are 2-4 bytes each

When the editor tried to delete or insert at position `cursor.x` (a character index) using `String::remove(cursor.x)` (which expects a byte index), it would panic with "byte index is not a char boundary" on any multi-byte character.

The fix adds proper character-to-byte index conversion throughout the editor, ensuring all string manipulation uses byte indices while cursor tracking continues to use character positions.

The two-cursor bug occurred because `update_cursor()` used `self.bounds` while `draw()` used `get_content_area()`. When an indicator is added (via `with_scrollbars_and_indicator()`), the content area starts 1 row below the bounds, causing the terminal cursor to be positioned incorrectly.

The scrollbar division by zero occurred when `max_val < min_val`, making `(max_val - min_val + 1) <= 0`. This could happen when content shrinks below viewport size or parameters are set incorrectly.

## [0.2.1] - 2025-11-03

### Added
- **Input Validators**: Comprehensive input validation system matching Borland's TValidator architecture
  - New `Validator` trait with `is_valid()`, `is_valid_input()`, `error()`, and `valid()` methods
  - `FilterValidator`: Validates input against allowed character set (e.g., digits only)
  - `RangeValidator`: Validates numeric input within min/max range
  - Support for decimal, hexadecimal (0x prefix), and octal (0 prefix) number formats
  - Real-time validation: invalid characters rejected as user types
  - Final validation: check complete input before accepting
  - `ValidatorRef` type alias: `Rc<RefCell<dyn Validator>>` for shared validator references

### Changed
- **InputLine**: Enhanced with validator support
  - Added `with_validator()` constructor to create InputLine with validator
  - Added `set_validator()` method to attach validator after construction
  - Added `validate()` method to check current input validity
  - Character insertion now checks `is_valid_input()` before accepting
  - Matches Borland's `TInputLine` with `TValidator` attachment pattern

### Examples
- **validator_demo.rs**: New example demonstrating input validation
  - Field 1: Digits only (FilterValidator with "0123456789")
  - Field 2: Number 0-100 (RangeValidator for positive range)
  - Field 3: Number -50 to 50 (RangeValidator for mixed range)
  - Field 4: Hex 0x00-0xFF (RangeValidator with hex support)
  - Shows real-time rejection of invalid characters
  - Displays validation results when OK is clicked

### Technical Details
This implements Borland Turbo Vision's validator architecture from validate.h and tvalidat.cc. The `Validator` trait provides the base validation interface, with `FilterValidator` implementing character filtering (matching `TFilterValidator` from tfilterv.cc) and `RangeValidator` implementing numeric range validation (matching `TRangeValidator` from trangeva.cc).

The `InputLine` checks validators in two contexts:
1. **During typing** (`is_valid_input()`): Rejects invalid characters immediately
2. **Final validation** (`is_valid()`): Checks complete input when accepting

RangeValidator supports multiple number formats:
- Decimal: "123", "-45"
- Hexadecimal: "0xFF", "0xAB"
- Octal: "077" (63 decimal), "0100" (64 decimal)

This matches Borland's `get_val()` and `get_uval()` functions from trangeva.cc:59-69.

Reference: Borland's TValidator architecture in validate.h, tvalidat.cc, tfilterv.cc, and trangeva.cc.

## [0.2.0] - 2025-11-03

### Added
- **Broadcast Enhancement**: Added owner-aware broadcast method to Group
  - New `broadcast()` method takes optional `owner_index` parameter
  - Prevents broadcast echo back to the originating view
  - Matches Borland's `message()` function pattern from tvutil.h
  - Enables focus-list navigation and sophisticated command routing patterns
  - Foundation for future inter-view communication features

### Fixed
- **Menu Example**: Fixed OK button command to use CM_OK instead of 0
  - Buttons in menu.rs dialogs now properly close when clicked
  - Added CM_OK to imports
  - Dialog's handle_event now correctly recognizes CM_OK command

### Technical Details
The `Group::broadcast()` method implements Borland's message pattern where broadcasts can skip the originator. This prevents circular event loops and enables proper implementation of focus navigation commands (like Ctrl+Tab to cycle through siblings without the current view receiving its own broadcast).

The method signature is `broadcast(&mut self, event: &mut Event, owner_index: Option<usize>)` where owner_index identifies the child that originated the broadcast. This child will be skipped when distributing the event to all children.

Reference: Borland's `void *message(TView *receiver, ...)` in tvutil.h and TGroup::forEach pattern in tgroup.cc:675-689.

## [0.1.10] - 2025-11-03

### Added
- **Event Re-queuing System**: Implemented Borland's putEvent() pattern for deferred event processing
  - Added `put_event()` method to Terminal
  - Added `pending_event` field to Terminal struct
  - Events can now be re-queued for processing in next iteration
  - Matches Borland's `TProgram::putEvent()` and `TProgram::pending` from tprogram.cc
  - Enables complex event transformation chains and command generation patterns

### Changed
- **Terminal**: Enhanced `poll_event()` to check pending events first
  - Pending events are processed before polling for new input
  - Matches Borland's `TProgram::getEvent()` behavior (tprogram.cc:154-194)
  - Event queue is FIFO - pending event delivered on next poll
  - Supports Borland-style event flow patterns

### Technical Details
This completes the event architecture trilogy started in v0.1.9. While three-phase processing handles HOW events flow through views, event re-queuing handles WHEN events are processed. The `put_event()` method allows views to:
- Generate new events for next iteration (e.g., converting mouse clicks to commands)
- Defer complex event processing
- Implement modal dialog patterns where unhandled events bubble up
- Match Borland's event generation patterns from status line and buttons

The pending event is checked first in `poll_event()`, ensuring re-queued events take priority over new input. This matches the exact behavior of `TProgram::getEvent()` which checks `pending.what != evNothing` before reading new events.

## [0.1.9] - 2025-11-03

### Added
- **Three-Phase Event Processing**: Implemented Borland's three-phase event handling architecture
  - Phase 1 (PreProcess): Views with `OF_PRE_PROCESS` flag get first chance at events
  - Phase 2 (Focused): Currently focused view processes event
  - Phase 3 (PostProcess): Views with `OF_POST_PROCESS` flag get last chance
  - Enables proper event interception patterns matching Borland's TGroup::handleEvent()
  - `Button` now uses `OF_POST_PROCESS` to intercept Space/Enter when not focused
  - `StatusLine` now uses `OF_POST_PROCESS` to monitor all key presses
  - Added `options()` and `set_options()` methods to View trait

### Changed
- **Group**: Enhanced `handle_event()` with three-phase processing for keyboard/command events
  - Mouse events continue to use positional routing (no three-phase)
  - Keyboard and Command events now flow through PreProcess → Focused → PostProcess
  - Matches Borland's `focusedEvents` vs `positionalEvents` distinction
  - Each phase checks if event was handled (EventType::Nothing) before continuing

- **Button**: Now implements `options()` with `OF_POST_PROCESS` flag
  - Buttons can intercept their hotkeys even when not focused
  - Matches Borland's button behavior from tbutton.cc

- **StatusLine**: Now implements `options()` with `OF_POST_PROCESS` flag
  - Status line monitors all key presses in post-process phase
  - Enables status line to handle function keys globally
  - Matches Borland's TStatusLine architecture from tstatusl.cc

- **View trait**: Added `options()` and `set_options()` methods
  - Default implementation returns 0 (no special processing)
  - Views can set `OF_PRE_PROCESS` or `OF_POST_PROCESS` flags
  - Foundation for advanced event routing patterns

### Technical Details
This implements the critical architectural pattern from Borland's TGroup::handleEvent() (tgroup.cc:342-369). The three-phase system allows views to intercept events before or after the focused view processes them. This is essential for:
- Buttons responding to Space/Enter even when another control is focused
- Status line handling function keys globally
- Modal dialogs intercepting Esc/F10 regardless of focus

The implementation distinguishes between `focusedEvents` (keyboard/command) which use three-phase processing, and `positionalEvents` (mouse) which route directly to the view under the cursor.

## [0.1.8] - 2025-11-03

### Added
- **Status Line Hot Spots**: Status line items now have visual feedback and improved interaction
  - Mouse hover highlighting: items change color when mouse hovers over them
  - Hover color: White on Green (matching button style) for better visibility
  - Dedicated `draw_select()` method to render items with selection state
  - Context-sensitive hint display: `set_hint()` method to show help text on status line
  - Improved mouse tracking during clicks for better user feedback
  - New `StatusLine::item_mouse_is_in()` helper to detect which item mouse is over
  - New example: `status_line_demo.rs` showcasing all status line improvements

### Changed
- **StatusLine**: Enhanced with hover state tracking and hint system
  - Added `selected_item: Option<usize>` field to track hovered item
  - Added `hint_text: Option<String>` field for context-sensitive help
  - Improved `handle_event()` with mouse move detection for hover effects
  - Hint text displayed on right side when available and space permits
  - Matches Borland's `TStatusLine::drawSelect()` pattern from tstatusl.cc

- **Color Palette**: Added new status line selection colors
  - `STATUS_SELECTED`: White on Green for selected status items
  - `STATUS_SELECTED_SHORTCUT`: Yellow on Green for shortcuts in selected items
  - Provides clear visual feedback matching button color scheme

### Technical Details
This implements Borland Turbo Vision's status line hot spot pattern. The status line now provides visual feedback when the user hovers over items, matching the behavior of `TStatusLine::drawSelect()` in the original implementation. The hint system allows displaying context-sensitive help text on the status line, which can be updated based on the focused control or current application state. This is a step toward full context-sensitive help support planned for v0.3.0.

## [0.1.7] - 2025-11-03

### Added
- **Keyboard Shortcuts in Menus**: Menu items now display keyboard shortcuts right-aligned
  - New `MenuItem::new_with_shortcut()` constructor to specify shortcut text
  - Shortcuts displayed right-aligned in dropdown menus (e.g., "Ctrl+O", "F3", "Alt+X")
  - Menu width automatically adjusts to accommodate shortcuts
  - Matches Borland's `TMenuItem::keyCode` display pattern

### Changed
- **MenuItem**: Enhanced with optional `shortcut` field for display purposes
  - Shortcut text is purely visual - shows users what keys to press
  - Improves menu polish and user experience
  - Follows desktop UI conventions for shortcut display

### Technical Details
This implements Borland Turbo Vision's menu shortcut display pattern. Menu items can now show keyboard shortcuts right-aligned, similar to modern desktop applications. The implementation calculates menu width based on both item text and shortcut length, ensuring proper alignment and visual polish. Shortcuts are currently display-only - actual global shortcut handling would require application-level key routing.

## [0.1.6] - 2025-11-03

### Added
- **Window Resize Support**: Windows can now be resized by dragging the bottom-right corner
  - Click and drag the bottom-right corner (last 2 columns, last row) to resize
  - Minimum size constraints prevent windows from becoming too small (16x6 minimum)
  - All child views automatically update during resize
  - Efficient redrawing using union rect pattern (same as window movement)
  - Matches Borland's `TWindow` resize behavior from `twindow.cc` and `tframe.cc`

### Changed
- **Frame**: Enhanced mouse event handling to detect resize corner clicks
  - Bottom-right corner detection: `mouse.x >= size.x - 2 && mouse.y >= size.y - 1`
  - New `SF_RESIZING` state flag to track resize operations
  - Matches Borland's `TFrame::handleEvent()` pattern (tframe.cc:214-219)

- **Window**: Added resize drag logic and size constraints
  - Tracks resize offset from bottom-right corner during drag
  - Applies minimum size limits (16 wide, 6 tall) matching Borland's `minWinSize`
  - Updates frame and interior bounds during resize
  - Prevents resizing smaller than minimum dimensions

### Technical Details
This implements Borland Turbo Vision's window resizing architecture. The Frame detects resize corner clicks and sets the `SF_RESIZING` flag. The Window handles mouse move events during resize, calculating new size while respecting minimum size constraints from `sizeLimits()`. Child views are automatically repositioned through the `set_bounds()` cascade, and efficient redrawing uses the union rect pattern to minimize screen updates.

## [0.1.5] - 2025-11-03

### Added
- **Double-click Detection**: Implemented proper double-click detection for mouse events
  - Added timing and position tracking to Terminal (`last_click_time`, `last_click_pos`)
  - Detects double-clicks within 500ms at the same position
  - `MouseEvent.double_click` field now properly set by `Terminal::convert_mouse_event()`
  - Matches expected desktop UI behavior for quick successive clicks

### Changed
- **ListBox**: Updated to trigger selection command on double-click instead of repeated single clicks
  - Double-clicking an item in ListBox now immediately triggers the `on_select_command`
  - Single clicks select items without triggering the command
  - Matches Borland's `TListViewer` pattern: `if (event.mouse.doubleClick) selectItem(focused)`

- **FileDialog**: Automatically benefits from ListBox double-click support
  - Double-clicking files now opens them immediately (no need to click OK button)
  - Double-clicking folders navigates into them
  - Improves user experience with modern expected behavior

### Technical Details
This implements double-click detection based on Borland Turbo Vision's `MouseEventType.doubleClick` field. The implementation tracks click timing using `Instant` and checks that consecutive clicks occur within 500ms at the same position. This pattern matches modern desktop UI conventions while maintaining compatibility with Borland's event-driven architecture.

## [0.1.4] - 2025-11-02

### Changed
- **Refactored Modal Execution Architecture**: Completely redesigned how modal dialogs work to match Borland Turbo Vision's architecture
  - Moved event loop from `Dialog` to `Group` level (matching Borland's `TGroup::execute()`)
  - `Group` now has `execute()`, `end_modal()`, and `get_end_state()` methods
  - `Dialog::execute()` now implements its own event loop that calls `Dialog::handle_event()` for proper polymorphic behavior
  - Dialog handles its own drawing because it's not on the desktop
  - Fixed modal dialog hang bugs related to event loop and end state checking
  - This change eliminates window movement trails and provides correct modal behavior

### Added
- **Architectural Documentation**: Created `local-only/ARCHITECTURAL-FINDINGS.md` documenting:
  - How Borland's event loop architecture works (studied original C++ source)
  - Differences between C++ inheritance and Rust composition patterns
  - Why the event loop belongs in Group, not Dialog
  - Bug fixes and design decisions
  - Comparison of Borland's TGroup::execute() with the Rust implementation

### Fixed
- **Modal Dialog Trails**: Fixed issue where moving modal dialogs left visual trails on screen
- **Dialog Hang Bug #1**: Fixed infinite loop where `end_state` check was inside event handling block
- **Dialog Hang Bug #2**: Fixed polymorphism issue where `Group::handle_event()` was called instead of `Dialog::handle_event()`
- **Application::get_event()**: Now properly draws desktop before returning events, preventing trails

### Technical Details
This release implements Borland Turbo Vision's proven architecture for modal execution. The key insight from studying the original Borland C++ source code (in `local-only/borland-tvision/`) is that **the event loop belongs in TGroup**, not in individual dialog types. In Borland:

```cpp
// TGroup::execute() - the ONE event loop (tgroup.cc:182-195)
ushort TGroup::execute() {
    do {
        endState = 0;
        do {
            TEvent e;
            getEvent(e);        // Get event from owner chain
            handleEvent(e);     // Virtual dispatch to TDialog::handleEvent
        } while(endState == 0);
    } while(!valid(endState));
    return endState;
}
```

Our Rust implementation adapts this pattern:
- `Group` has `execute()` with event loop and `end_state` field
- `Dialog::execute()` implements the loop pattern but calls `Dialog::handle_event()` for polymorphism
- `Dialog::handle_event()` calls `window.end_modal()` when commands occur
- Drawing happens in the loop because dialogs aren't on the desktop

See `local-only/ARCHITECTURAL-FINDINGS.md` for complete analysis.

## [0.1.3] - 2025-11-02

### Added
- **Scroll Wheel Support**: Mouse wheel scrolling now works in ListBox, Memo, and TextView components
  - Wheel up scrolls content upward (moves selection/cursor up)
  - Wheel down scrolls content downward (moves selection/cursor down)
  - Only responds when mouse is within the component's bounds
  - Implemented by adding `MouseWheelUp` and `MouseWheelDown` event types to the event system
  - Terminal now converts crossterm's `ScrollUp` and `ScrollDown` events to internal event types

- **Window Closing Support**: Non-modal windows can now be properly closed
  - Click close button on non-modal windows to remove them from desktop
  - Modal dialogs: close button converts `CM_CLOSE` to `CM_CANCEL`
  - Non-modal windows: close button sets `SF_CLOSED` flag, removed by Desktop on next frame
  - Matches Borland's `TWindow::handleEvent()` behavior (twindow.cc lines 124-138)
  - Added `SF_CLOSED` flag (0x1000) to mark windows for removal
  - Desktop automatically removes closed windows after event handling

### Fixed
- **TextView Indicator**: Indicator now updates properly when scrolling with mouse wheel or keyboard

### Technical Details
**Scroll Wheel**: This implements modern mouse wheel support that wasn't present in the original Borland Turbo Vision (which predated mouse wheels). The implementation follows the framework's event-driven architecture:
- Added event type constants `EV_MOUSE_WHEEL_UP` (0x0010) and `EV_MOUSE_WHEEL_DOWN` (0x0020)
- Updated `EV_MOUSE` mask to 0x003F to include wheel events
- Each scrollable component checks mouse position before handling wheel events
- Wheel events are cleared after handling to prevent propagation

**Window Closing**: Adapts Borland's architecture for Rust's ownership model:
- Borland uses `CLY_destroy(this)` to remove views from owner
- Rust uses `SF_CLOSED` flag since views can't remove themselves from parent Vec
- `Window::handle_event()` sets flag on `CM_CLOSE` (non-modal) or converts to `CM_CANCEL` (modal)
- `Desktop::remove_closed_windows()` removes flagged windows after event handling
- `Group::remove()` handles child removal and focus tracking

## [0.1.2] - 2025-11-02

### Added
- **Z-Order Management**: Non-modal windows can now be brought to the front by clicking on them, matching Borland Turbo Vision's `TGroup::selectView()` behavior.
- **Modal Window Support**: Modal dialogs (like `Dialog::execute()`) now properly block interaction with background windows. When a modal dialog is present, clicking background windows has no effect.
- **Menu Borders and Shadows**: Dropdown menus now display with single-line borders and shadows, matching Borland's TMenuBox styling:
  - Single-line box drawing characters (`┌─┐`, `│`, `└─┘`, `├─┤`)
  - 2x1 shadow (2 cells wide on right, 1 cell tall on bottom)
  - Verified against original Borland Turbo Vision source code
- **Window Overlap Test**: New `window_modal_overlap_test` example demonstrating z-order management with three overlapping non-modal windows.

### Fixed
- **Mouse Event Z-Order**: Fixed mouse event handling to search in reverse z-order (top-most view first), preventing background views from capturing events intended for foreground windows.
- **Upward Dragging**: Fixed issue where windows could not be dragged upward. Windows can now be dragged in all directions by sending mouse events to dragging windows even when the mouse moves outside their bounds.

### Changed
- **Group::bring_to_front()**: Added method to reorder children in z-order, automatically updating focused index.
- **Desktop Event Handling**: Desktop now manages z-order changes on mouse clicks and enforces modal blocking when modal windows are present.
- **Dialog Modal Flag**: `Dialog::execute()` now automatically sets and clears the `SF_MODAL` flag, making all executed dialogs modal by default.

### Technical Details
This release implements Borland Turbo Vision's window management architecture:
- **Z-Order**: Children vector index represents z-order (higher index = on top)
- **Modal Scope**: Top-most window with `SF_MODAL` flag captures all events
- **Border Drawing**: Uses Borland's `frameChars` pattern for consistent styling
- **Shadow Rendering**: Matches Borland's `shadowSize = {2, 1}` and rendering algorithm

## [0.1.1] - 2025-11-02

### Fixed
- **Window dragging trails**: Fixed visual corruption when dragging windows. Modal dialogs now properly redraw the desktop background on each frame, matching Borland Turbo Vision's `TProgram::getEvent()` pattern where the entire screen is redrawn before polling for events.

### Changed
- **Desktop architecture**: Desktop now uses a `Background` view as its first child (matching Borland's `TDeskTop` with `TBackground`), ensuring proper z-order rendering.
- **FileDialog execution**: `FileDialog::execute()` now takes an `Application` reference and redraws the desktop before drawing the dialog, following Borland's modal dialog pattern.

### Technical Details
The fix addresses a fundamental architectural issue where modal dialogs had their own event loops that only redrawed themselves, not the desktop background. This caused visible trails when windows moved. The solution follows Borland Turbo Vision's pattern where `getEvent()` triggers a full screen redraw before returning events to modal views.

## [0.1.0] - 2025-11-02

### Added

#### Core System
- Event-driven architecture with keyboard and command-based event routing
- Complete drawing system with color support (16-color palette with attribute system)
- Geometry primitives with absolute and relative positioning
- Focus management with Tab navigation and keyboard shortcuts
- Modal dialog execution system
- Cross-platform terminal I/O abstraction layer built on crossterm

#### UI Components
- **Dialog**: Dialog boxes with frames and close buttons
- **Button**: Interactive buttons with keyboard shortcuts and mouse support
- **StaticText**: Text labels with centered text support
- **InputLine**: Single-line text input fields
- **Menu**: Menu bar with dropdown menus and mouse support
- **StatusLine**: Status bar with clickable items
- **Desktop**: Desktop manager for window management
- **ScrollBar**: Vertical and horizontal scrollbars with mouse support
- **Scroller**: Base class for scrollable views
- **Indicator**: Position/status display widget
- **TextView**: Scrollable text viewer with line numbers
- **CheckBox**: Checkbox controls with mouse support
- **RadioButton**: Radio button groups with mouse support
- **ListBox**: List selection widget with mouse and keyboard navigation
- **Memo**: Multi-line text editor with basic editing capabilities
- **FileDialog**: Full-featured file selection dialog with directory navigation

#### Input & Navigation
- Full keyboard support with arrow keys, Tab, Enter, Escape
- Mouse support including:
  - Button clicks and hover effects
  - Menu interaction
  - Status bar clicks
  - Dialog close buttons
  - ListBox item selection
  - Scrollbar interaction
- Keyboard shortcuts for quick access

#### Application Framework
- Application class with event loop
- Terminal initialization and cleanup
- Resource management

### Documentation
- Comprehensive README with quick start guide
- Module overview documentation
- Example programs demonstrating framework usage

### Known Limitations
- Full text editor with search/replace not yet implemented (basic editing available in Memo)

[0.1.3]: https://github.com/aovestdipaperino/turbo-vision-4-rust/releases/tag/v0.1.3
[0.1.2]: https://github.com/aovestdipaperino/turbo-vision-4-rust/releases/tag/v0.1.2
[0.1.1]: https://github.com/aovestdipaperino/turbo-vision-4-rust/releases/tag/v0.1.1
[0.1.0]: https://github.com/aovestdipaperino/turbo-vision-4-rust/releases/tag/v0.1.0
