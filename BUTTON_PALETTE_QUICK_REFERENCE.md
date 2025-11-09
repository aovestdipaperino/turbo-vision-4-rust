# Button and Palette System - Quick Reference Guide

## File Locations

| Component | Path |
|-----------|------|
| Button | `/Users/enzo/Code/turbo-vision/src/views/button.rs` |
| MenuBar | `/Users/enzo/Code/turbo-vision/src/views/menu_bar.rs` |
| MenuBox | `/Users/enzo/Code/turbo-vision/src/views/menu_box.rs` |
| Palette Defs | `/Users/enzo/Code/turbo-vision/src/core/palette.rs` |
| View Trait | `/Users/enzo/Code/turbo-vision/src/views/view.rs` |

## Button Color Mapping Quick Lookup

### Drawing State → Palette Index

```
Button State                 Palette Index   Final Color
─────────────────────────────────────────────────────────
Disabled                     4               DarkGray on Green
Focused (has keyboard focus) 3               White on Green
Default (Enter activates)    2               LightGreen on Green
Normal (unfocused)           1               Black on Green
Shortcut letter              7               Yellow on Green
Right shadow                 8               LightGray on DarkGray
```

## Palette Constants (Newly Added)

### MenuBar & MenuBox (src/views/menu_bar.rs and src/views/menu_box.rs)
```rust
const MENU_NORMAL: u8 = 1;      // Normal menu text
const MENU_SELECTED: u8 = 2;    // Selected menu item
const MENU_DISABLED: u8 = 3;    // Disabled menu item
const MENU_SHORTCUT: u8 = 4;    // Shortcut key
```

## Map Color Flow

```
Step 1: View calls map_color(index)
        Example: button.map_color(1) for normal text

Step 2: Apply view's palette
        CP_BUTTON[1] = 13

Step 3: Apply dialog palette (if index >= 32)
        CP_GRAY_DIALOG[13] = 36

Step 4: Apply app palette (final lookup)
        CP_APP_COLOR[36] = 0x20 (Attr: Black fg, Green bg)

Result: Attr { fg: Black, bg: Green }
```

## Critical Code Sections

### Button Draw Method (Palette Index Selection)
**File:** `button.rs`, lines ~80-120

```rust
let button_attr = if is_disabled {
    self.map_color(4)      // Disabled
} else if is_focused {
    self.map_color(3)      // Selected/focused
} else if self.is_default {
    self.map_color(2)      // Default
} else {
    self.map_color(1)      // Normal
};
```

### Broadcast Event Handling (Critical!)
**File:** `button.rs`, lines ~180-210

```rust
if event.what == EventType::Broadcast {
    if event.command == CM_COMMAND_SET_CHANGED {
        let should_be_enabled = command_set::command_enabled(self.command);
        if should_be_enabled && is_currently_disabled {
            self.set_disabled(false);
        }
    }
    return;  // IMPORTANT: Don't consume broadcast
}
```

### Map Color Implementation
**File:** `view.rs`, lines ~300-365

Walks palette chain:
1. View palette (if present)
2. Dialog palette (for indices 32-63)
3. App palette (final lookup)

## Recent Changes Summary

| File | Change Type | What Changed |
|------|-------------|--------------|
| `button.rs` | Logging | Added debug logging to calc.log |
| `menu_bar.rs` | Constants | Added MENU_* constants |
| `menu_box.rs` | Constants + Fix | Added MENU_* constants, fixed shortcut color |
| `view.rs` | Docs | Enhanced palette documentation |

## Debug Logging

When running button operations, check `calc.log` for:

```
Button 'OK' draw START, owner=...
  Calling map_color(1)...
    Remapped 1 -> 13 via own palette
    Remapped 13 -> 36 via dialog palette
    Using CP_APP_COLOR[36]
  map_color(1) OK
  Calling map_color(8) for shadow...
  ...
```

## Palette Definition Quick Lookup

### Button Palette (CP_BUTTON)
```rust
Index  Maps To  Description
─────  ────────  ─────────────────────
1      13        Normal
2      13        Default
3      14        Focused/Selected
4      14        Disabled
5      16        Reserved
6      15        Reserved
7      15        Shortcut
8      9         Shadow
```

### Menu Palette (CP_MENU_BAR)
```rust
Index  Maps To  Color Scheme
─────  ────────  ──────────────────────
1      2         Black on LightGray
2      39        White on Green
3      3         DarkGray on LightGray
4      4         Red on LightGray
```

## Button Features

### State Flags
- `SF_DISABLED`: Command not enabled in global set
- `SF_FOCUSED`: Button has keyboard focus
- Button tracks `is_default` separately (not a flag)

### Event Handling
- **Keyboard:** Enter or Space activates when focused
- **Mouse:** Click within bounds activates
- **Broadcast:** Received even when disabled (allows state updates)
- **Commands:** Can be broadcast or direct command events

### Key Methods
```rust
pub fn new(bounds, title, command, is_default) -> Button
pub fn set_disabled(disabled: bool)
pub fn is_disabled() -> bool
pub fn set_broadcast(broadcast: bool)
pub fn set_selectable(selectable: bool)
```

## Owner Tracking

```rust
// Set by parent Group when button is added
button.set_owner(parent_ptr);

// Used in palette chain (potential future enhancement)
if let Some(owner) = button.get_owner() {
    // Could traverse owner chain for palette resolution
}
```

## Testing Key Points

### Regression Test: Broadcast Handling
**Location:** `button.rs`, test `test_disabled_button_receives_broadcast_and_becomes_enabled`

**What it tests:**
1. Button created disabled when command is disabled
2. Command is then enabled in global set
3. CM_COMMAND_SET_CHANGED broadcast is sent
4. Button receives broadcast and becomes enabled

**Why it matters:** Earlier bug had disabled buttons returning early, preventing broadcast reception.

### Color Mapping Tests
No direct tests for map_color (checked indirectly through draw), but you can verify by:
1. Enabling debug logging
2. Drawing button in different states
3. Checking calc.log for correct palette index sequence

## Common Modifications

### Add Custom Palette to New Component
```rust
fn get_palette(&self) -> Option<Palette> {
    use crate::core::palette::{Palette, palettes};
    Some(Palette::from_slice(palettes::CP_BUTTON))
}
```

### Use Palette Constant Instead of Magic Number
```rust
// Before:
let normal_attr = self.map_color(1);

// After:
const MY_NORMAL: u8 = 1;
let normal_attr = self.map_color(MY_NORMAL);
```

### Add New State Color
```rust
// In draw():
let new_state_attr = if some_condition {
    self.map_color(5)  // Add index 5 to palette
} else {
    self.map_color(1)  // Default
};
```

## Palette Index Ranges

| Range | Purpose | Example |
|-------|---------|---------|
| 1-7 | Desktop/Global | App menu, status |
| 8-15 | Window (Blue) | Window backgrounds |
| 16-23 | Window (Cyan) | Window backgrounds |
| 24-31 | Window (Gray) | Window backgrounds |
| 32-63 | Dialog Contents | Buttons, inputs, etc. |
| 64-85 | Misc | Editors, help |

**Key Point:** Indices 32-63 trigger dialog palette remapping!

## Color Attribute Byte Encoding

```
Attribute Byte: 0xBF (example)
                ││ └─ Foreground (0-15)
                └──── Background (0-15)

Examples:
0x20 = Black fg (0), Green bg (2) = Button background
0x2F = White fg (15), Green bg (2) = Focused button
0x87 = LightGray fg (7), DarkGray bg (8) = Shadow
```

## Important Links

### In Code
- Button state handling: `button.rs:40-75`
- Draw palette selection: `button.rs:80-140`
- Event handling: `button.rs:145-265`
- Broadcast handling: `button.rs:185-210` (CRITICAL)
- Palette resolution: `view.rs:305-360`

### In Commits
- Latest palette docs: `a0df6b7`
- Safe palette chain: `9b74935`
- Initial palette impl: `68899be`

## Troubleshooting

### Button Not Responding to Input
1. Check `is_disabled()` - may need to enable command
2. Check `can_focus()` - needs to return true
3. For broadcasts, verify event isn't being cleared

### Button Color Wrong
1. Check palette constant definition
2. Check map_color() debug log
3. Verify palette index in get_palette()
4. Check dialog palette range (32-63)

### Owner Pointer Issues
1. Set by Group.add() - don't set manually
2. Currently non-owning (safe)
3. Used for logging/debugging
4. Don't dereference in current impl

## Acronyms

| Acronym | Meaning |
|---------|---------|
| CP_ | Color Palette prefix |
| SF_ | State Flag prefix |
| OF_ | Option Flag prefix |
| CM_ | Command prefix |
| MB_ | Mouse Button prefix |
| KB_ | Keyboard prefix |
| Attr | Text Attribute (color pair) |
| TvColor | Turbo Vision Color (16 colors) |

## Navigation by Task

### "I need to change button colors"
1. Open `src/core/palette.rs`
2. Find `CP_BUTTON` definition
3. Modify the indices or create new palette
4. Update `button.rs:get_palette()` if needed

### "I need to debug color mapping"
1. Run button draw operation
2. Check `calc.log` for trace
3. Follow the index remapping sequence
4. Verify each palette step

### "I need to add a new button state"
1. Open `button.rs`
2. Add new index to palette or use existing
3. Update draw() state check
4. Update get_palette() if using new palette
5. Add test case

### "I need to handle menu colors differently"
1. Check `menu_bar.rs` or `menu_box.rs`
2. Note they use `CP_MENU_BAR` palette
3. Modify constants or palette definition
4. Both files now use named constants (not magic numbers)
