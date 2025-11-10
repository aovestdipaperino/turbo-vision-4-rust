# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.10.1] - 2025-11-10

### Added
- **Runtime Palette Customization**
  - New `Application::set_palette()` method for changing application palette at runtime
  - Automatically triggers redraw when palette actually changes
  - Change detection avoids unnecessary redraws
  - Simplifies theme switching API - one method call instead of two
  - Example usage: `app.set_palette(Some(dark_palette))` - redraw is automatic
  - Thread-local storage for custom palettes via `palettes::set_custom_palette()`

- **Palette Themes Demo** (examples/palette_themes_demo.rs - 214 lines)
  - Interactive demonstration of 4 different color themes
  - Default theme: Classic Borland Turbo Vision colors
  - Dark theme: Dark backgrounds with bright text for low-light environments
  - High-Contrast theme: Black/white for maximum visibility and accessibility
  - Solarized theme: Earth tones inspired by the Solarized color scheme
  - Shows how to create custom palettes with 63-byte color arrays
  - Demonstrates automatic redraw on palette change

### Fixed
- **Dialog Command Handling** (src/views/dialog.rs)
  - Fixed: Dialogs now properly handle custom button commands
  - Previously only CM_OK, CM_YES, CM_NO, CM_CANCEL closed dialogs
  - Now all commands (including custom ones) call `end_modal()`
  - Enables theme switching buttons and other custom commands to work correctly

- **Syntax Highlighting Import** (src/views/syntax.rs)
  - Fixed unused import warning for TvColor
  - Made TvColor import conditional with `#[cfg(test)]`
  - Only used in test code, not production

### Changed
- **Documentation Updates**
  - README.md: Added "Runtime Customization" bullet to palette features
  - README.md: New "Custom Palettes and Theming" section with code example
  - README.md: Updated tokei statistics (108 files, 31,215 lines, 23,546 code)
  - README.md: Updated test count (198 tests, up from 194)
  - docs/PALETTE_SYSTEM.md: New "Runtime Palette Customization" section (137 lines)
  - docs/PALETTE_SYSTEM.md: Documents `Application::set_palette()` API with examples
  - docs/PALETTE_SYSTEM.md: Explains custom palette format (63 bytes, fg<<4|bg encoding)
  - docs/PALETTE_SYSTEM.md: Palette layout reference and theme creation examples

### Technical Details
**Color Mapping Flow**:
1. User calls `app.set_palette(Some(palette))`
2. Method compares new palette with current palette
3. If different, calls `palettes::set_custom_palette(palette)`
4. Sets `needs_redraw` flag to trigger full redraw
5. Next frame, all views remap colors through new palette

**Implementation Benefits**:
- Simple API: One method call instead of two (set + redraw)
- Efficient: Only redraws when palette actually changes
- Safe: No redundant redraws on same palette
- Automatic: Users don't need to remember `needs_redraw()`

**Custom Palette Format**:
- 63-byte array where each byte encodes: `(foreground << 4) | background`
- Color values: 0=Black, 1=Blue, 2=Green, 3=Cyan, 4=Red, 5=Magenta, 6=Brown, 7=LightGray, 8=DarkGray, 9=LightBlue, A=LightGreen, B=LightCyan, C=LightRed, D=LightMagenta, E=Yellow, F=White
- Layout: 1-8 Desktop, 9-15 Menu/StatusLine, 16-23 Cyan Window, 24-31 Gray Window, 32-63 Dialog/Controls

## [0.10.0] - 2025-11-09

### Fixed
- **Palette Remapping System**
  - Fixed dialog control palette remapping to match Borland Turbo Vision behavior
  - Labels now display correct white-on-grey instead of red-on-grey
  - Menu selected items show black-on-green instead of white-on-grey
  - Menu shortcuts display red-on-grey as in original Borland implementation
  - Button shadows render correctly with proper foreground/background swap
  - Added `owner_type` field to Button, Label, StaticText, and InputLine
  - All dialog controls now default to `OwnerType::Dialog` for proper palette remapping
  - Views with `OwnerType::None` (MenuBar, StatusLine) use direct app palette
  - Views with `OwnerType::Dialog` remap indices 1-31 through dialog palette

### Added
- **Palette Regression Tests**
  - Added 9 comprehensive palette regression tests in `tests/palette_regression_tests.rs`
  - Tests verify Borland-accurate colors for Button, Label, StaticText, InputLine, ScrollBar, MenuBar, and Dialog
  - Tests ensure color stability across changes
  - All tests pass with visually correct colors

### Changed
- Moved palette regression tests from `src/core/` to `tests/` directory for better organization
- Updated `map_color()` to respect `OwnerType` for context-aware palette remapping
- Fixed CP_MENU_BAR palette to match Borland's original values `[2, 5, 3, 4]`
- Removed ScrollBar's custom `map_color()` implementation (now uses default View trait implementation)
- Replaced magic palette indices in StatusLine with named constants

### Removed
- Deleted obsolete example files: `dialog_example.rs`, `history.rs`, `key_test.rs`, `menu_status_data.rs`, `quick_start.rs`, `status_line_demo.rs`
- Moved `menu_status_data.rs` to `tests/` directory

## [0.9.2] - 2025-11-08

### Added
- **Semi-Transparent Shadows**
  - Shadows now darken underlying content instead of drawing opaque backgrounds
  - Matches Borland Turbo Vision's original VGA-based shadow behavior
  - Added `TvColor::to_rgb()` for RGB component extraction
  - Added `TvColor::from_rgb()` for closest color matching via Euclidean distance
  - Added `Attr::darken(factor)` method for color darkening (default 50%)
  - Added `Terminal::read_cell()` to read existing buffer content
  - Completely rewrote `draw_shadow()` to use read-modify-write pattern
  - Preserves underlying characters while darkening colors
  - Cross-platform implementation using RGB blending instead of VGA bit manipulation

### Changed
- Shadow rendering now reads terminal buffer before drawing
- Shadow cells show darkened version of underlying content (semi-transparent effect)
- Updated Rust-vs-Borland comparison document with new shadow implementation details

### Technical Details
- Darkening uses 50% factor (configurable constant)
- Color matching finds nearest color in 16-color palette using RGB distance
- Falls back to default shadow color for out-of-bounds positions
- No performance impact - shadow rendering is still O(n) where n = shadow size

## [0.3.0] - 2025-11-06

### Added
- **Real-Time Input Validation System**
  - Complete birthdate validation in biorhythm example with three validation layers
  - RangeValidator for field-level input filtering
  - Cross-field date validation (leap years, month lengths, future dates)
  - Command set integration for dynamic button enable/disable
  - Custom event loop pattern for real-time validation feedback
  - Validation runs after every keystroke with immediate UI updates

- **Command Set Broadcasting Pattern**
  - `CM_COMMAND_SET_CHANGED` broadcast system for global command state changes
  - Buttons automatically update disabled state via command set queries
  - Declarative command management instead of direct widget manipulation
  - Matches Borland Turbo Vision's command enable/disable architecture

- **Enhanced Biorhythm Calculator**
  - Startup dialog for birthdate input (no default random chart)
  - Clean exit on cancel (no orphaned windows)
  - Date prefill across dialog invocations
  - Centered windows and dialogs accounting for shadow size
  - Three-layer validation: RangeValidator, complete date check, command updates
  - Enter key support via event reprocessing

- **Dialog Event Reprocessing**
  - Fixed Enter key handling in dialogs with default buttons
  - Event conversion (KB_ENTER → CM_OK) now properly reprocessed
  - Matches Borland's `putEvent()` pattern for converted events
  - Ensures modal dialogs close correctly when Enter pressed in InputLine

- **Window Centering with Shadow Calculations**
  - Proper centering accounting for shadow size (2 cols width, 1 row height)
  - Menu bar and status line offset calculations for main windows
  - Dialog centering for full-screen placement
  - Visual balance maintained across different terminal sizes

- **Comprehensive Documentation**
  - `docs/BIORHYTHM-TUTORIAL.md` - Narrative blog-style tutorial (822 lines)
  - Real-world examples and personal discovery stories
  - Common patterns and gotchas with DO/DON'T comparisons
  - Manual test cases for validation scenarios
  - Quick reference section for key patterns

### Changed
- **Biorhythm Example Architecture**
  - Moved from random initial chart to dialog-first startup flow
  - Window creation deferred until after successful date validation
  - Custom event loop replaces standard `dialog.execute()` for validation needs
  - Command set pattern used instead of direct button manipulation

### Fixed
- **Enter Key in Modal Dialogs**
  - Dialog event loop now reprocesses converted command events
  - KB_ENTER → CM_OK conversion properly triggers `end_modal()`
  - Matches Borland's event re-queuing behavior

### Removed
- **Unused Downcasting Infrastructure**
  - Removed `as_any_mut()` from View trait (not needed with command set pattern)
  - Removed Button's `as_any_mut()` implementation
  - Removed `std::any::Any` imports
  - Cleaner API without unnecessary complexity

### Technical Details

**Real-Time Validation Architecture:**

The validation system uses three coordinated layers:
1. **RangeValidator** - Character-level filtering during typing (1-31 for day, 1-12 for month)
2. **Complete Date Validation** - Cross-field checks (Feb 31, leap years, future dates)
3. **Command Set Updates** - Global command enable/disable with broadcast propagation

The custom event loop pattern enables validation after every event:
```rust
loop {
    draw_and_flush();
    if let Some(event) = poll_event() {
        dialog.handle_event(&mut event);
        if event.what == EventType::Command {
            dialog.handle_event(&mut event);  // Reprocess converted events
        }
        validate_and_update_command_state();  // After every event
        broadcast_if_changed();
    }
    if dialog.get_end_state() != 0 { break; }
}
```

**Command Set Pattern vs Direct Manipulation:**

Instead of fragile child index access:
```rust
// OLD: Direct manipulation (removed)
dialog.child_at_mut(8).downcast_mut::<Button>().set_disabled(true);
```

Use declarative command state:
```rust
// NEW: Command set pattern
command_set::disable_command(CM_OK);
broadcast(CM_COMMAND_SET_CHANGED);
```

Benefits: Scales to multiple buttons, no fragile indices, separates validation from UI structure.

**Shadow-Aware Centering:**

Windows have shadows (2 cols right, 1 row bottom) that must be included in centering:
```rust
let x = (screen_width - (window_width + 2)) / 2;  // +2 for shadow
let y = 1 + ((screen_height - 2) - (window_height + 1)) / 2;  // +1 for shadow, 1+ for menu
```

This ensures visual balance - ignoring shadows makes windows appear off-center.

**Event Reprocessing Pattern:**

When dialogs convert keyboard events to commands, the converted event must be reprocessed:
```rust
dialog.handle_event(&mut event);  // First pass: KB_ENTER → CM_OK
if event.what == EventType::Command {
    dialog.handle_event(&mut event);  // Second pass: Process CM_OK
}
```

Without this, Enter key appears to do nothing in modal dialogs.

## [0.2.11] - 2025-11-04

### Fixed
- **StatusLine Drawing and Hit Detection** - Fixed highlighting extending into separator
  - StatusLine now draws leading and trailing spaces around text (matches Borland tstatusl.cc:143-145)
  - Selection highlight includes spaces before and after text, but not the separator
  - Separator "│ " always drawn in normal color, never highlighted
  - Hit detection properly includes leading space and text with trailing space
  - Matches Borland TStatusLine drawing and hit detection behavior exactly

### Technical Details
The StatusLine highlighting bug was caused by not matching Borland's exact drawing pattern. In Borland TStatusLine::drawSelect (tstatusl.cc:143-145), each status item is drawn as:
1. Space before text (in selection color when selected)
2. The text itself (with proper shortcut highlighting)
3. Space after text (in selection color when selected)
4. Separator (always in normal color)

Previously, we were drawing the text directly without surrounding spaces, and the separator was being drawn with the selection color. This caused the selection highlight to extend into the separator. The fix now exactly replicates Borland's drawing sequence.

## [0.2.10] - 2025-11-04

### Added
- **FileEditor Component** (src/views/file_editor.rs)
  - Proper implementation of Borland's TFileEditor pattern
  - File name tracking and modified flag management
  - `valid(app, command)` method for save prompts on close
  - Load/Save/SaveAs operations with proper file management
  - Wraps Editor component with file-specific functionality
  - Ready for future proper architecture implementation

- **Window and Desktop Helper Methods**
  - `Window::get_editor_text_if_present()` - Extract current editor text
  - `Window::is_editor_modified()` - Check if editor has unsaved changes
  - `Window::clear_editor_modified()` - Clear modified flag after save
  - `Desktop::get_first_window_as_window()` - Get immutable window reference
  - `Desktop::get_first_window_as_window_mut()` - Get mutable window reference
  - Pragmatic unsafe downcasting helpers for editor demo use case

- **Standard Library Dialog Functions** (src/views/msgbox.rs)
  - `message_box_ok()` - Simple information message with OK button
  - `message_box_error()` - Error message with OK button
  - `message_box_warning()` - Warning message with OK button
  - `confirmation_box()` - Yes/No/Cancel confirmation dialog
  - `confirmation_box_yes_no()` - Yes/No confirmation dialog
  - `confirmation_box_ok_cancel()` - OK/Cancel confirmation dialog
  - `search_box()` - Search dialog returning Option<String>
  - `search_replace_box()` - Find/replace dialog returning Option<(String, String)>
  - `goto_line_box()` - Go to line dialog returning Option<usize> with validation
  - Convenience wrappers around existing `message_box()` function
  - Eliminates need for manual dialog construction in common cases
  - Example: `examples/dialogs_demo.rs` demonstrating all dialog types

- **Rust Text Editor Demo** (demo/rust_editor.rs)
  - Full-featured text editor application with Rust syntax highlighting
  - File operations: New, Open, Save, Save As (using FileDialog)
  - Menu bar: File, Edit, Tools menus with keyboard shortcuts
  - Status line: F10 Menu, Ctrl+S Save, Ctrl+F Find
  - Close menu item (Ctrl+W) for closing editor window
  - Close button (■) in window frame
  - Smart dirty flag tracking - only prompts when actually modified
  - Save prompts before destructive operations (Close, New, Quit)
  - Actual save on "Yes" in confirmation dialog
  - Search and Replace dialogs using standard library functions (search_box, search_replace_box, goto_line_box)
  - Rust analyzer integration (placeholder for future LSP integration)
  - About dialog showing "Lonbard Turbo Rust" on startup
  - Empty desktop on startup - user must choose File → New or File → Open
  - Comprehensive demonstration of Editor, FileDialog, and standard dialogs
  - Documentation: demo/README.md with features, shortcuts, and usage guide

### Changed
- **Rust Editor Cleanup**
  - Replaced local dialog implementations with standard library functions
  - Removed 120+ lines of duplicate dialog code (show_search_dialog, show_replace_dialog, show_goto_line_dialog)
  - Simplified show_about_dialog() to use message_box_ok()
  - Removed unused imports (Button, Dialog, InputLine, Label, Rc, RefCell)

- **msgbox.rs Dialog Layout**
  - Moved dialog text and buttons one row higher for better appearance
  - Improved visual spacing in confirmation dialogs

- **msgbox.rs Command Constants**
  - Removed duplicate CM_YES and CM_NO definitions
  - Now imports CM_YES and CM_NO from core::command module
  - Maintains consistency across entire framework

- **Window CM_CLOSE Event Handling**
  - Non-modal windows no longer auto-close on CM_CLOSE
  - CM_CLOSE event propagates to application level for validation
  - Matches Borland's TWindow::close() → valid(cmClose) pattern
  - Applications can intercept and validate before allowing close

- **Rust Editor Save Operations**
  - Fixed critical bug: now saves actual editor content, not stale state
  - Simplified EditorState to only track filename
  - Save operations retrieve current text from editor window
  - Clear modified flag after successful save
  - CM_SAVE no longer recreates window (performance optimization)
  - CMD_SAVE_AS only recreates window on success (to update title)

### Fixed
- **StatusLine Event Handling** - Critical bug where StatusLine was completely non-functional
  - Changed from OF_POST_PROCESS to OF_PRE_PROCESS to match Borland behavior (tstatusl.cc:33)
  - Added status_line.handle_event() call in rust_editor event loop
  - StatusLine now properly handles mouse clicks and keyboard shortcuts
  - Items now generate commands when clicked or shortcuts pressed
- **StatusLine Hit Detection** - Fixed "one off to the right" highlighting issue
  - First item now includes leading space (position 0) in hit area
  - All items include first separator space after text (matches Borland's inc=2)
  - Subsequent items don't include previous separator in their hit area
  - Hit detection now matches Borland TStatusLine behavior (tstatusl.cc:204)
- **Editor Content Not Saved** - Critical bug where saves would write initial content instead of current edits
- **Close Button Not Prompting** - Frame's close button now properly triggers save confirmation
- **Always Prompting on Close** - Now only prompts when editor is actually modified
- **Save on Confirmation** - "Yes" button in save dialog now actually saves the file

### Technical Details
The FileEditor component provides the proper Borland TFileEditor pattern with encapsulated file management and validation. The Window/Desktop helpers enable pragmatic downcasting for the editor demo while maintaining type safety. The rust_editor now properly synchronizes editor content with file operations.

**StatusLine Event Processing**: The StatusLine bug was caused by using OF_POST_PROCESS instead of OF_PRE_PROCESS. In Borland's TStatusLine (tstatusl.cc:33), the status line sets `options |= ofPreProcess` to ensure it gets first chance at events. This allows it to intercept function keys and mouse clicks before they reach the focused view. The fix also required adding the status_line.handle_event() call in the event loop's pre-process phase, matching Borland's TGroup::handleEvent() three-phase architecture.

The standard library dialog functions provide a cleaner API for common dialog patterns. Instead of manually constructing Dialog with StaticText and Button components, applications can now use simple function calls:

**Before** (47 lines):
```rust
let mut dialog = Dialog::new(bounds, "Save Changes?");
let text = StaticText::new_centered(...);
dialog.add(Box::new(text));
let yes_button = Button::new(..., CM_YES, true);
dialog.add(Box::new(yes_button));
// ... more buttons ...
let result = dialog.execute(app);
```

**After** (1 line):
```rust
let result = confirmation_box(app, "Save changes?");
```

The new input dialog functions follow the same simple pattern:
```rust
// Search dialog
if let Some(search_text) = search_box(&mut app, "Search") {
    // User entered search text
}

// Find and replace dialog
if let Some((find, replace)) = search_replace_box(&mut app, "Replace") {
    // User entered both find and replace text
}

// Go to line dialog with validation
if let Some(line_num) = goto_line_box(&mut app, "Go to Line") {
    // User entered valid line number
}
```

The rust_editor demo showcases a complete application built with Turbo Vision for Rust, demonstrating best practices for:
- Application structure with event loops
- Desktop/Window management
- Menu systems with cascading submenus
- File I/O with FileDialog integration
- Modal dialog patterns
- Syntax highlighting with Editor component
- Status line with keyboard shortcuts

This editor serves as a reference implementation for building complete TUI applications.

## [0.2.9] - 2025-11-04

### Fixed
- **MenuBox Mouse Interaction** (CRITICAL BUG FIX - Borland Compatibility)
  - **Root Cause**: MenuBox executed commands on MouseDown instead of MouseUp, inconsistent with Borland Turbo Vision
  - **Impact**: Menu items executed before mouse was fully released, preventing proper drag-selection
  - **Fix**: Following Borland tmenuvie.cc:215-222, MouseDown now only tracks selection, MouseUp executes commands
  - **Result**: Menu behavior now matches original Turbo Vision exactly

- **MenuBox ESC/ESC ESC Handling**
  - Both KB_ESC and KB_ESC_ESC now properly close popup menus
  - Returns command 0 to signal cancellation matching Borland behavior (tmenuvie.cc:264-268)

- **Submenu Auto-Popup Removed** (Borland Compatibility)
  - **Issue**: Submenus were appearing automatically on hover or right arrow navigation
  - **Fix**: Following Borland tmenuvie.cc:333-349, submenus now only show on explicit action:
    - Press Enter on submenu item
    - Click (MouseUp) on submenu item
  - KB_RIGHT now only navigates to next top-level menu, doesn't open submenus
  - Matches original Turbo Vision behavior perfectly

- **Validator Demo Dialog Height**
  - Increased dialog height from 30 to 34 lines to properly display all fields and buttons

### Added
- **MenuBar Cascading Submenu Support**
  - Added `show_cascading_submenu()` method to display nested submenus
  - Added `check_cascading_submenu()` public method for event loop integration
  - MenuBox positioned to right of parent dropdown menu
  - Proper keyboard (Enter) and mouse (Click) activation
  - Full support for multi-level menu hierarchies

- **Extended Menu Example** (examples/menu.rs)
  - File menu now includes Recent Files submenu (3 sample files, Clear Recent option)
  - Edit menu added with Cut/Copy/Paste and Preferences submenu
  - Preferences submenu contains General, Appearance, and Keyboard Shortcuts
  - Right-click popup menu on desktop with New File, Open File, Properties
  - Comprehensive demonstration of all menu features
  - Status line shows "Right-Click Popup" hint

- **MenuBar MouseUp Event Handling**
  - Added proper MouseUp event handling matching Borland behavior
  - Executes commands only when mouse released on selected item
  - Handles submenu activation on click-release

### Changed
- **Validator Demo Unified**
  - Removed initial menu selection dialog
  - All validators (Filter, Range, Picture) now shown in single comprehensive dialog
  - Organized into clear sections with dynamic layout
  - Filter & Range Validators section with 4 different validator types
  - Picture Mask Validators section with phone, date, and product code examples

### Technical Details
- **Borland Compatibility**: All menu changes verified against local-only/borland-tvision source code
- **Files Modified**:
  - src/views/menu_box.rs (mouse handling, ESC handling)
  - src/views/menu_bar.rs (cascading menus, MouseUp support, removed auto-popup)
  - examples/menu.rs (extended with submenus and popup menu)
  - examples/validator_demo.rs (unified dialog, removed menu)

## [0.2.8] - 2025-11-03

### Fixed
- **Keyboard Modifiers Lost in Event System** (CRITICAL BUG FIX)
  - **Root Cause**: `Terminal::poll_event()` was creating Event with `Event::keyboard(key_code)` which lost modifiers from crossterm's KeyEvent
  - **Impact**: ALL keyboard modifiers (Shift, Ctrl, Alt) were being stripped, making Shift+Arrow selection completely non-functional
  - **Discovery**: Through debug testing, found crossterm correctly sent SHIFT but Editor received KeyModifiers(0x0)
  - **Fix**: Changed Terminal to preserve `key.modifiers` when creating Event structure
  - **Result**: Shift+Arrow keys now work perfectly with visible cyan selection highlighting

- **Editor Selection Visibility**
  - Added `is_position_selected()` helper method to check if character is in selection
  - Modified `draw()` to apply EDITOR_SELECTED color (black on cyan) to selected text
  - Selection highlighting works with both plain text and syntax-highlighted code
  - Multi-line selections fully supported

- **Window ESC ESC Handling**
  - Window now handles both KB_ESC and KB_ESC_ESC to close modal windows
  - Matches expected Turbo Vision behavior for double-ESC

### Added
- **examples/key_test.rs** - Diagnostic tool to test keyboard input directly from crossterm
  - Shows raw KeyCode and KeyModifiers for debugging
  - Useful for verifying terminal keyboard behavior

### Changed
- **Test Suite**: 178 tests (all passing)
- **Code Size**: 16,030 lines total, 12,239 lines of code

## [0.2.7] - 2025-11-03

### Fixed
- **Editor Text Selection with Shift+Arrow Keys** (CRITICAL BUG FIX)
  - **Root Cause**: Event structure didn't track keyboard modifiers (Shift, Ctrl, Alt). Editor had hardcoded `shift_pressed = false` with TODO comment
  - **Impact**: Shift+Arrow keys, Shift+Home, Shift+End didn't create text selections
  - **Fix**: Added `key_modifiers` field to Event structure, updated Editor to check for SHIFT modifier
  - **Features Now Working**:
    - Shift+Arrow keys create text selection
    - Shift+Home selects from cursor to start of line
    - Shift+End selects from cursor to end of line
    - Shift+PgUp/PgDn select full pages
    - Moving without Shift clears selection (expected behavior)

- **Button Broadcast Handling** (CRITICAL BUG FIX)
  - **Root Cause**: Disabled buttons were checking disabled state before processing broadcasts, causing them to return early and never receive CM_COMMAND_SET_CHANGED broadcasts
  - **Impact**: Buttons that started disabled (e.g., Cut, Copy, Paste when clipboard empty) would stay disabled forever, breaking the command set system
  - **Fix**: Moved broadcast handling to the top of `Button::handle_event()`, before disabled check
  - **Verification**: Confirmed implementation matches Borland's original behavior (tbutton.cc:196, tview.cc:486, tbutton.cc:255-262)
  - **Documentation**: Added detailed comments referencing Borland source code line numbers
  - Fixed command_set_demo - Enable/Disable buttons now properly update button states

- **Dialog Event Handling**
  - Fixed Dialog to accept ANY command as end-modal signal, not just CM_OK/CM_CANCEL/CM_YES/CM_NO
  - Matches Borland behavior where any command reaching the dialog ends the modal loop
  - Fixed editor_demo and validator_demo menu dialogs with custom command IDs

- **Editor Demo Modal Windows**
  - Wrapped all editor demos in modal Windows so they can be used interactively
  - Added ESC key handling to Window for modal windows (calls end_modal on ESC press)
  - Fixed editor_demo to use Window with SF_MODAL flag for all four demo modes

### Added
- **Regression Tests for Button Broadcasts** (7 new tests)
  - test_disabled_button_receives_broadcast_and_becomes_enabled (main regression test)
  - test_enabled_button_receives_broadcast_and_becomes_disabled
  - test_button_creation_with_disabled_command
  - test_button_creation_with_enabled_command
  - test_disabled_button_ignores_keyboard_events
  - test_disabled_button_ignores_mouse_clicks
  - test_broadcast_does_not_clear_event

### Changed
- **Test Suite**: 178 tests (was 171) - all passing
- **Code Size**: 16,030 lines total, 12,239 lines of code (was 15,845 / 12,134)

## [0.2.6] - 2025-11-03

### Added
- **Syntax Highlighting System** (~450 lines, 7 tests)
  - **SyntaxHighlighter trait** (src/views/syntax.rs)
    - Extensible architecture for language-specific highlighting
    - Token-based coloring system (Keywords, Strings, Comments, Numbers, etc.)
    - Line-by-line highlighting with efficient token generation
    - Methods: `language()`, `highlight_line()`, multi-line context support
  - **TokenType enum** - 11 token types with default color mappings
    - Keywords (Yellow), Strings (LightRed), Comments (LightCyan)
    - Numbers (LightMagenta), Operators (White), Types (LightGreen)
    - Functions (Cyan), Preprocessor (LightCyan), etc.
  - **RustHighlighter** - Built-in Rust syntax highlighting
    - Recognizes Rust keywords (fn, let, if, for, match, etc.)
    - String and character literals with escape sequences
    - Line comments (//) and block comments (/* */)
    - Numeric literals (decimal, hex, float)
    - Type names (i32, String, custom types)
    - Operators and special characters
  - **PlainTextHighlighter** - No-op highlighter for plain text
  - **Editor Integration**
    - `set_highlighter()` - Attach syntax highlighter to Editor
    - `clear_highlighter()` - Remove highlighting
    - `has_highlighter()` - Check if highlighting is enabled
    - Automatic per-token color rendering in draw method
    - Preserves all existing Editor functionality (search/replace, undo/redo, etc.)

- **TPXPictureValidator** (Picture Mask Validator) (~360 lines, 11 tests)
  - **PictureValidator** (src/views/picture_validator.rs - 255 lines, 8 tests)
    - Validates and formats input according to picture masks
    - Matches Borland's TPXPictureValidator from validate.h
    - Mask characters:
      - `#` - Digit (0-9)
      - `@` - Alpha (A-Z, a-z)
      - `!` - Any character
      - `*` - Optional section marker
      - Literals - Must match exactly (e.g., `/`, `-`, `(`, `)`)
    - Methods: `new()`, `format()`, `set_auto_format()`
    - Auto-formatting mode inserts literals automatically as user types
    - Example masks:
      - `"(###) ###-####"` - Phone number: (555) 123-4567
      - `"##/##/####"` - Date: 12/25/2023
      - `"@@@@-####"` - Product code: ABCD-1234
    - Implements Validator trait for InputLine integration
  - **Helper function**: `picture_validator()` - Creates ValidatorRef

### Examples
- **editor_demo.rs** - Comprehensive editor demonstration (290 lines)
  - Menu-driven interface with 4 demonstrations:
    1. Basic editing (undo/redo/clipboard operations)
    2. Search and replace functionality
    3. Syntax highlighting (Rust code with colored tokens)
    4. File I/O operations (load/save)
  - Consolidates previous examples: file_editor.rs, full_editor.rs, syntax_highlighting.rs
  - Shows all Editor features in one interactive demo
- **validator_demo.rs** - All validator types demonstration (320 lines)
  - Menu-driven interface with 2 demonstrations:
    1. FilterValidator and RangeValidator (character filtering, numeric ranges, hex numbers)
    2. PictureValidator (phone numbers, dates, product codes with format masks)
  - Consolidates previous examples: validator_demo.rs, picture_validator.rs
  - Shows all validation patterns with interactive examples

### Changed
- **Examples reorganization** - Reduced from 19 to 16 examples by consolidation
  - Removed: file_editor.rs, full_editor.rs, syntax_highlighting.rs (→ editor_demo.rs)
  - Removed: picture_validator.rs (→ validator_demo.rs)
  - Updated examples/README.md with new structure and descriptions

### Technical Details
**Syntax Highlighting** implements a token-based coloring system that works efficiently with the Editor's line-by-line rendering. Each line is parsed into tokens (keyword, string, comment, etc.) with start/end positions. The Editor's draw method iterates through tokens and applies colors accordingly. The system is extensible - new languages can be added by implementing the SyntaxHighlighter trait.

**Design Patterns**:
- Hook-based architecture for language extensions
- Token type abstraction for color mapping customization
- Line-by-line processing for efficiency
- Optional multi-line state tracking for block comments
- Integrates seamlessly with existing Editor features

**TPXPictureValidator** provides input formatting and validation using Borland's picture mask pattern. Unlike character filtering (FilterValidator) or range validation (RangeValidator), picture masks define the exact format of input including literal characters. The validator can auto-format input by inserting literals (like parentheses, slashes, dashes) as the user types, or validate completed input against the mask pattern.

**Design Patterns**:
- Matches Borland's TPXPictureValidator architecture
- Integrates with InputLine via Validator trait
- Supports both auto-format and validation-only modes
- Optional sections with `*` marker (partially implemented)
- Real-time validation during typing

Reference: Borland Turbo Vision tvalidat.cc, validate.h (picture validators)

### Test Coverage
- **Syntax Highlighting**: 7 new tests
  - Token type colors, plain text, Rust keywords, strings, comments, numbers, types
- **Picture Validator**: 11 new tests
  - Phone mask, date mask, format functions, alpha mask, optional sections, partial input
- **Total Tests**: 171 tests passing (up from 154)

## [0.2.5] - 2025-11-03

### Added
- **Help System** (Phase 9 - 867 lines, 22 tests)
  - **HelpFile** (src/views/help_file.rs - 302 lines, 7 tests)
    - Parses markdown files into help topics with # Title {#topic-id} format
    - Cross-reference support via [Text](#topic-id) markdown links
    - Methods: `get_topic()`, `get_default_topic()`, `get_topic_ids()`, `reload()`
    - Human-readable format replacing Borland's binary TPH files
  - **HelpViewer** (src/views/help_viewer.rs - 286 lines, 4 tests)
    - Displays help topic content with scrolling support
    - Keyboard navigation: Up/Down, PgUp/PgDn, Home/End
    - Optional vertical scrollbar for long topics
    - Focus-aware coloring (HELP_NORMAL, HELP_FOCUSED)
    - Uses DrawBuffer for efficient rendering
  - **HelpWindow** (src/views/help_window.rs - 157 lines, 4 tests)
    - Modal help window wrapper around HelpViewer
    - Methods: `show_topic()`, `show_default_topic()`, `execute()`
    - ESC key closes help window
    - Delegates to Window for frame and modal behavior
  - **HelpContext** (src/views/help_context.rs - 122 lines, 7 tests)
    - Maps context IDs (u16) to help topic IDs (String)
    - Methods: `register()`, `get_topic()`, `has_context()`, `unregister()`
    - Foundation for F1 context-sensitive help support
  - Example: examples/help_system.rs demonstrating help topics and navigation
  - Sample help file: examples/help.md with 6 topics and cross-references

- **Color Palette**:
  - `HELP_NORMAL`: Black on LightGray for unfocused help text
  - `HELP_FOCUSED`: Black on White for focused help text

### Technical Details
The Help System implements Borland's context-sensitive help architecture using modern markdown format instead of proprietary binary TPH files. Key advantages:

**Markdown Format**:
- Human-readable and easy to author
- Version control friendly (plain text diffs)
- No special tools required for editing
- Cross-platform compatible
- Can be generated from other documentation

**Architecture**:
- HelpFile parses markdown on load, building a HashMap of topics
- Topics identified by {#topic-id} in heading: # Welcome {#welcome}
- Cross-references via standard markdown links: [See also](#other-topic)
- HelpViewer provides scrollable display with keyboard navigation
- HelpWindow wraps viewer in a modal window for display
- HelpContext enables F1-style context-sensitive help

**Design Patterns**:
- Matches Borland's THelpFile, THelpViewer, THelpWindow patterns
- Uses Rc<RefCell<HelpFile>> for shared help file access
- Modal execution via Window's execute() method
- Keyboard-driven navigation matching Borland's behavior

Reference: Borland Turbo Vision help.h and help system architecture

## [0.2.3] - 2025-11-03

### Added
- **TEditWindow** (src/views/edit_window.rs - 169 lines, 3 tests)
  - Window wrapper around Editor for ready-to-use editor windows
  - Delegates file operations: `load_file()`, `save_file()`, `save_as()`, `get_filename()`
  - Provides editor access methods: `editor()`, `editor_mut()`, `is_modified()`
  - Automatically adjusts editor bounds when window is resized
  - Implements View trait with proper event routing to both Window and Editor
  - Matches Borland's TEditWindow pattern from teditor.h

- **TLookupValidator** (src/views/lookup_validator.rs - 255 lines, 8 tests)
  - Validates input against a list of valid values
  - Supports case-sensitive mode via `new()` and case-insensitive via `new_case_insensitive()`
  - Helper methods: `add_value()`, `remove_value()`, `contains()`, `set_case_sensitive()`
  - Implements Validator trait for InputLine integration
  - Allows all characters during typing, validates on completion
  - Matches Borland's TLookupValidator pattern from validate.h

- **OS Clipboard Integration** (src/core/clipboard.rs - enhanced with arboard 3.3)
  - Added system clipboard integration via arboard crate
  - Fallback strategy: attempts OS clipboard first, falls back to in-memory
  - Cross-platform support (macOS, Linux, Windows)
  - Functions: `set_clipboard()`, `get_clipboard()`, `has_clipboard_content()`, `clear_clipboard()`
  - Editor can now copy/paste to/from system clipboard (Ctrl+C, Ctrl+X, Ctrl+V)
  - Graceful degradation on platforms without clipboard support

### Changed
- **Documentation**: Updated MISSING_FEATURES.md progress tracking
  - Marked TFileCollection and TDirCollection as obsolete (use Vec<FileEntry/DirEntry>)
  - Updated summary: 29 missing components (down from 35), 648 hours remaining
  - HIGH Priority: COMPLETE (0 hours remaining)
  - Added Phase 7+ improvements section documenting recent work
  - Updated statistics: 134 tests passing (up from 126)

- **Cargo.toml**: Added arboard 3.3 dependency for OS clipboard support

### Technical Details
**TEditWindow** provides a complete editor window solution by composing a Window with an Editor. It matches Borland's TEditWindow pattern where the editor fills the window interior and the window provides the frame and title bar. The implementation properly handles View trait delegation, routing draw and event calls to both the Window (for frame) and Editor (for content).

**TLookupValidator** implements Borland's validation pattern for restricting input to predefined values. Unlike FilterValidator (character-by-character) or RangeValidator (numeric), LookupValidator validates the complete string against a list. This is useful for dropdowns, enum values, or any constrained input set.

**OS Clipboard** integration uses the arboard crate to access the system clipboard across platforms. The fallback strategy ensures the application works even when OS clipboard access fails, maintaining the in-memory clipboard as a reliable backup.

Reference: Borland's TEditWindow (teditor.h), TLookupValidator (validate.h), and clipboard integration patterns.

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
