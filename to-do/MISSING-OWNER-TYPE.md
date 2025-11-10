# Missing owner_type Implementation

## Issue

Many View implementations are missing the `owner_type` field and corresponding `get_owner_type()` / `set_owner_type()` methods. This breaks the palette chain and can cause incorrect colors.

## Root Cause

The Editor view was showing a green background instead of the correct Yellow-on-Blue editor colors because:

1. Editor was missing `owner_type` field
2. `get_owner_type()` defaulted to `OwnerType::None`
3. Window palette remapping was skipped
4. Color mapping went: Editor color 1 → CP_EDITOR[1]=6 → CP_APP_COLOR[6]=0x28 (Gray on Green) ❌

## Fix Applied

Fixed `src/views/editor.rs` by:
1. Adding `owner_type: super::view::OwnerType` field to struct
2. Initializing it in constructor: `owner_type: super::view::OwnerType::None`
3. Implementing the trait methods:
```rust
fn get_owner_type(&self) -> super::view::OwnerType {
    self.owner_type
}

fn set_owner_type(&mut self, owner_type: super::view::OwnerType) {
    self.owner_type = owner_type;
}
```

## Views That Need Fixing

### High Priority (Common Controls with Palettes)

These are frequently used in dialogs and windows and will show incorrect colors:

- [ ] `checkbox.rs` - Uses CP_CLUSTER palette
- [ ] `radiobutton.rs` - Uses CP_CLUSTER palette
- [ ] `listbox.rs` - Uses CP_LISTBOX palette
- [ ] `memo.rs` - Uses CP_MEMO palette
- [ ] `history.rs` - Uses CP_HISTORY palette
- [ ] `history_viewer.rs` - Uses CP_HISTORY_VIEWER palette

### Medium Priority (Specialized Viewers)

These are used in help systems and specialized views:

- [ ] `text_viewer.rs` - Scrollable text display
- [ ] `help_viewer.rs` - Uses CP_HELP_VIEWER palette
- [ ] `help_index.rs` - Help index list
- [ ] `help_toc.rs` - Help table of contents
- [ ] `scroller.rs` - Base class for scrollable views
- [ ] `indicator.rs` - Uses CP_INDICATOR palette
- [ ] `paramtext.rs` - Parametrized text display

### Lower Priority (List Derivatives)

These inherit from or are similar to ListBox:

- [ ] `dir_listbox.rs` - Directory listing
- [ ] `file_list.rs` - File listing
- [ ] `sorted_listbox.rs` - Sorted list
- [ ] `outline.rs` - Outline view

### File/Dialog Wrappers

These are composite views that may need forwarding:

- [ ] `chdir_dialog.rs` - Change directory dialog
- [ ] `color_dialog.rs` - Color picker dialog
- [ ] `file_dialog.rs` - File open/save dialog
- [ ] `file_editor.rs` - File editor wrapper
- [ ] `terminal_widget.rs` - Terminal widget
- [ ] `color_selector.rs` - Color selection widget

## Views Already Fixed

- [x] `editor.rs` - Fixed 2025-11-10
- [x] `button.rs` - Already had it
- [x] `input_line.rs` - Already had it
- [x] `label.rs` - Already had it
- [x] `static_text.rs` - Already had it
- [x] `scrollbar.rs` - Already had it
- [x] `edit_window.rs` - Has forwarding wrapper
- [x] `help_window.rs` - Has forwarding wrapper

## Views That Don't Need It

These are either top-level views or special cases that use direct app palette:

- `window.rs` - Parent container
- `dialog.rs` - Parent container
- `desktop.rs` - Top-level
- `menu_bar.rs` - Top-level, uses direct app palette
- `status_line.rs` - Top-level, uses direct app palette
- `menu_box.rs` - Part of menu system, uses direct palette
- `frame.rs` - Part of window frame
- `background.rs` - Desktop background
- `group.rs` - Container (children have owner_type set when added)

## Implementation Pattern

For each view that needs fixing:

1. Add field to struct:
```rust
pub struct ViewName {
    // ... existing fields ...
    owner: Option<*const dyn View>,
    owner_type: super::view::OwnerType,  // ADD THIS
}
```

2. Initialize in constructor(s):
```rust
pub fn new(...) -> Self {
    Self {
        // ... existing fields ...
        owner: None,
        owner_type: super::view::OwnerType::None,  // ADD THIS
    }
}
```

3. Implement trait methods in `impl View for ViewName`:
```rust
fn get_owner_type(&self) -> super::view::OwnerType {
    self.owner_type
}

fn set_owner_type(&mut self, owner_type: super::view::OwnerType) {
    self.owner_type = owner_type;
}
```

## Testing

After fixing, verify colors are correct:
- Dialogs should use Gray palette (gray background, cyan highlights)
- Window controls should use Blue palette (blue background, cyan/yellow highlights)
- Test with dialogs and windows to ensure correct color remapping

## Related Files

- `src/views/view.rs` - Contains `OwnerType` enum and `map_color()` implementation
- `src/views/window.rs` - Sets `OwnerType::Window` in `Window::add()`
- `src/views/dialog.rs` - Should set `OwnerType::Dialog` in `Dialog::add()`
- `src/views/group.rs` - Sets owner pointer but not owner_type (parent does that)
- `src/core/palette.rs` - Contains all palette definitions (CP_CLUSTER, CP_LISTBOX, etc.)
