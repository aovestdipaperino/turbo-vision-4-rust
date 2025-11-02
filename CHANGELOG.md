# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
