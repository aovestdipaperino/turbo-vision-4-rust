# Palette Flexibility & Window Management Fixes

Addresses GitHub issues #96, #97, #98, #101, #103 filed by Hans-Christian Esperer.

## Scope

Two groups of changes:

1. **Palette/Color Inflexibility** (#96, #98, #101): Public window palette selection, palette-integrated syntax highlighting, and documenting the ListBox-in-Dialog constraint.
2. **Window Management/Focus** (#103, #97): Public bring-to-front API and HelpWindow bug fix + UX improvements.

---

## Group 1: Palette/Color Inflexibility

### #96 -- Window palette selection not public

**Problem**: `Window::new_with_palette()` is private. Users can only create Blue (`new`), Cyan (`new_for_help`), or Gray/Dialog (`new_for_dialog`) windows.

**Fix**: Add a public constructor that accepts `WindowPaletteType`:

```rust
// New public API on Window
pub fn new_with_type(bounds: Rect, title: &str, palette_type: WindowPaletteType) -> Self
```

This delegates to the existing `new_with_palette()` with sensible defaults:
- `WindowPaletteType::Blue` -> `FramePaletteType::Editor`, resizable
- `WindowPaletteType::Cyan` -> `FramePaletteType::HelpWindow`, resizable
- `WindowPaletteType::Gray` -> `FramePaletteType::Dialog`, resizable
- `WindowPaletteType::Dialog` -> `FramePaletteType::Dialog`, not resizable

`WindowPaletteType` is already public. No new types needed.

**Files changed**: `src/views/window.rs`

### #98 -- Syntax highlighting colors hardcoded to blue background

**Problem**: `TokenType::default_color()` returns hardcoded `Attr` values (all with `TvColor::Blue` background). The editor draw method at `editor.rs:1239` calls `token.token_type.default_color()` directly, bypassing the palette chain. Syntax-highlighted text always has a blue background regardless of the window's palette.

**Fix**: Integrate syntax colors into the palette system.

#### New palette indices

Add 11 syntax color palette indices to `src/core/palette.rs`:

```rust
// Syntax token palette indices (editor-relative, map through window palette)
pub const SYNTAX_NORMAL_IDX: u8 = 9;
pub const SYNTAX_KEYWORD_IDX: u8 = 10;
pub const SYNTAX_STRING_IDX: u8 = 11;
pub const SYNTAX_COMMENT_IDX: u8 = 12;
pub const SYNTAX_NUMBER_IDX: u8 = 13;
pub const SYNTAX_OPERATOR_IDX: u8 = 14;
pub const SYNTAX_IDENTIFIER_IDX: u8 = 15;
pub const SYNTAX_TYPE_IDX: u8 = 16;
pub const SYNTAX_PREPROCESSOR_IDX: u8 = 17;
pub const SYNTAX_FUNCTION_IDX: u8 = 18;
pub const SYNTAX_SPECIAL_IDX: u8 = 19;
```

#### Extended window palettes

Extend `CP_BLUE_WINDOW`, `CP_CYAN_WINDOW`, `CP_GRAY_WINDOW` from 8 to 19 entries. Entries 9-19 map to new app palette positions 64-74.

```rust
// CP_BLUE_WINDOW extended (indices 1-19)
pub const CP_BLUE_WINDOW: &[u8] = &[
    8, 9, 10, 11, 12, 13, 14, 15,          // 1-8: Original window entries
    64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, // 9-19: Syntax colors
];

// CP_CYAN_WINDOW extended (indices 1-19)
pub const CP_CYAN_WINDOW: &[u8] = &[
    16, 17, 18, 19, 20, 21, 22, 23,        // 1-8: Original window entries
    75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, // 9-19: Syntax colors (cyan bg)
];

// CP_GRAY_WINDOW extended (indices 1-19)
pub const CP_GRAY_WINDOW: &[u8] = &[
    24, 25, 26, 27, 28, 29, 30, 31,        // 1-8: Original window entries
    86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, // 9-19: Syntax colors (gray bg)
];
```

#### Extended app palette

Extend `CP_APP_COLOR` from 63 to 96 entries. Entries 64-96 are syntax colors for each window background:

- 64-74: Blue background syntax colors (current hardcoded values, moved here)
- 75-85: Cyan background syntax colors (same foregrounds, cyan background)
- 86-96: Gray background syntax colors (same foregrounds, gray background)

#### TokenType palette index method

Add a method on `TokenType` to return the palette index:

```rust
impl TokenType {
    pub fn palette_index(&self) -> u8 {
        match self {
            TokenType::Normal => SYNTAX_NORMAL_IDX,
            TokenType::Keyword => SYNTAX_KEYWORD_IDX,
            // ... etc
        }
    }
}
```

#### Editor draw change

`editor.rs:1239` changes from:

```rust
token.token_type.default_color(),
```

to:

```rust
self.map_color(token.token_type.palette_index()),
```

`TokenType::default_color()` remains for backward compatibility but is no longer called by the editor.

**Files changed**: `src/core/palette.rs`, `src/views/syntax.rs`, `src/views/editor.rs`

### #101 -- ListBox palette indices assume Dialog context

**Problem**: `CP_LISTBOX` maps to dialog-relative indices 26-28. When placed in a Window (8-entry palette), these indices are out of bounds.

**Decision**: Document as by-design. In the original Borland Turbo Vision, `TListViewer` (`cpListViewer = "\x1A\x1A\x1B\x1C\x1D"`) was strictly a Dialog control. Window palettes were 8 entries; dialog palettes were 32 entries. Controls like TListBox, TButton, TInputLine, TLabel, TCluster, and TMemo all use dialog-relative indices above 8. This is an architectural constraint of the palette system, not a bug.

**Action**: Close issue #101 with explanation. Users who need list-like behavior in a Window should use a Dialog styled to look like a window, or use TScroller-based views.

---

## Group 2: Window Management/Focus

### #103 -- No public bring-to-front method

**Problem**: `Group::bring_to_front(index)` exists but Desktop exposes no way to bring a specific window to front by its ViewId. Users must destroy and recreate windows as a workaround.

**Fix**: Add a public method on Desktop:

```rust
impl Desktop {
    /// Bring a specific window to the front of the Z-order.
    /// Returns true if the window was found and moved, false if not found.
    pub fn bring_to_front(&mut self, view_id: ViewId) -> bool
}
```

Implementation:
1. Search `self.children` (skipping background at index 0) for a child whose ViewId matches.
2. If found, call `self.children.bring_to_front(index)`.
3. Set focus on the newly-fronted window, unfocus the previously-top window.
4. Return true/false.

**Pre-existing bug**: `Group::bring_to_front()` and `Group::send_to_back()` reorder `children` but do not reorder the parallel `view_ids` vec. This desyncs ViewId-to-child mapping after any z-order change. Fix both methods to also reorder `view_ids` in the same way. This is required for `Desktop::bring_to_front(ViewId)` to work correctly, and also fixes silent bugs in existing `select_next()`/`select_prev()` calls.

**Files changed**: `src/views/group.rs`, `src/views/desktop.rs`

### #97 -- HelpWindow usability issues

Four sub-items, one bug fix and three UX improvements.

#### 97a: Missing options/set_options delegation (bug)

**Problem**: `HelpWindow`'s `View` impl does not delegate `options()` or `set_options()` to `self.window`. The default `View::options()` returns `0`, so `OF_TOP_SELECT` is never set, and the Desktop's click-to-focus logic skips the HelpWindow.

**Fix**: Add two method delegations to the `impl View for HelpWindow` block:

```rust
fn options(&self) -> u16 {
    self.window.options()
}

fn set_options(&mut self, options: u16) {
    self.window.set_options(options);
}
```

**Files changed**: `src/views/help_window.rs`

#### 97b: Single-click follows hyperlinks

**Problem**: Currently single-click only selects a link; double-click follows it. This is unintuitive -- users expect single-click to follow a link (web-like behavior).

**Fix**: In `HelpViewer::handle_event`, when a single-click lands on a cross-ref, select it AND do not clear the event so `HelpWindow` can follow the link (same path as the current double-click logic). The double-click path stays for backward compat but is now redundant.

Specifically in `help_viewer.rs` around line 451: change from clearing the event on single-click to letting it propagate (like the double-click case currently does).

In `help_window.rs`, the `MouseDown` handler already checks for the selected target. Extend it to handle single-click (not just double-click):

```rust
EventType::MouseDown => {
    if event.mouse.buttons & MB_LEFT_BUTTON != 0 {
        // Let viewer handle selection first
        self.window.handle_event(event);
        // If a link was clicked, follow it
        let target = self.viewer.borrow().get_selected_target().map(|s| s.to_string());
        if let Some(target) = target {
            self.switch_to_topic(&target);
            event.clear();
            return;
        }
    }
}
```

**Files changed**: `src/views/help_viewer.rs`, `src/views/help_window.rs`

#### 97c: Up/Down arrow keys cycle visible hyperlinks

**Problem**: Only Tab/Shift+Tab cycle links. The reporter requests Up/Down for link cycling with automatic scrolling.

**Fix**: In `HelpViewer::handle_event`, add logic for `KB_UP` and `KB_DOWN`:

- If cross-refs exist on screen, Up/Down select the previous/next visible link instead of scrolling.
- When the first visible link is selected and Up is pressed, scroll up to reveal more links.
- When the last visible link is selected and Down is pressed, scroll down to reveal more links.
- When no links are visible, Up/Down scroll normally (current behavior).

New helper method:

```rust
/// Find the next/previous visible cross-reference relative to current selection
fn find_visible_cross_ref(&self, forward: bool) -> Option<usize>
```

**Files changed**: `src/views/help_viewer.rs`

#### 97d: Backspace restores scroll position and selected link

**Problem**: `go_back()` navigates to the previous topic but doesn't restore where the user was scrolled to or which link was selected.

**Fix**: Change the history entries from `String` (topic ID) to a struct:

```rust
struct HistoryEntry {
    topic_id: String,
    delta: Point,    // Scroll position
    selected: usize, // Selected cross-ref (1-based)
}
```

In `HelpWindow`:
- `switch_to_topic()`: Before switching, capture current `(delta, selected)` from the viewer and push a `HistoryEntry`.
- `go_back()`: After showing the topic, restore `delta` and `selected` on the viewer.
- `go_forward()`: Same restoration logic.

New methods on `HelpViewer`:

```rust
pub fn get_scroll_state(&self) -> (Point, usize)  // (delta, selected)
pub fn set_scroll_state(&mut self, delta: Point, selected: usize)
```

**Files changed**: `src/views/help_window.rs`, `src/views/help_viewer.rs`

---

## Out of Scope

- FileDialog resizability (#99)
- README screenshots (#102)
- Mouse wheel scrolling in HelpViewer (not requested, already exists in other views)
- Any changes to Dialog, ListBox, Button, or other dialog-only controls

## Testing Strategy

- Existing palette chain tests cover color resolution. Add tests for extended palette (indices 9-19 resolve correctly through blue/cyan/gray window palettes).
- Add test for `Desktop::bring_to_front(ViewId)` -- add two windows, bring the bottom one to front, verify z-order.
- Add test for `HelpWindow::options()` returns `OF_SELECTABLE | OF_TOP_SELECT | OF_TILEABLE`.
- History entry restoration: add test that `go_back()` restores `delta` and `selected`.
