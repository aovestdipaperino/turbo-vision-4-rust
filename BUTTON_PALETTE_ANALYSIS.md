# Button Implementation and Palette Integration Analysis

## Executive Summary

The button implementation has been significantly updated to properly support the Borland Turbo Vision palette system. Buttons now correctly map logical color indices through a palette hierarchy to obtain final display attributes. Recent changes include palette constant definitions and improved code organization.

---

## 1. Button View Implementation

### Location
`/Users/enzo/Code/turbo-vision/src/views/button.rs`

### Key Components

#### Button Structure
```rust
pub struct Button {
    bounds: Rect,
    title: String,
    command: CommandId,
    is_default: bool,
    is_broadcast: bool,
    state: StateFlags,
    options: u16,
    owner: Option<*const dyn View>,
}
```

#### Button Palette (CP_BUTTON)
```rust
pub const CP_BUTTON: &[u8] = &[
    13, 13, 14, 14, 16, 15, 15, 9,  // 1-8: (4=disabled), shadow maps to dialog 9
];
```

**Index Meanings:**
- Index 1: Normal text (maps to dialog index 13)
- Index 2: Default text (maps to dialog index 13) 
- Index 3: Selected/focused text (maps to dialog index 14)
- Index 4: Disabled text (maps to dialog index 14)
- Index 5: Reserved
- Index 6: Reserved
- Index 7: Shortcut text (maps to dialog index 15)
- Index 8: Shadow (maps to dialog index 9)

### Drawing Logic

The button draw method implements a state-based color selection:

```rust
fn draw(&mut self, terminal: &mut Terminal) {
    let is_disabled = self.is_disabled();
    let is_focused = self.is_focused();

    // Button color indices (from CP_BUTTON palette):
    let button_attr = if is_disabled {
        self.map_color(4)      // Disabled
    } else if is_focused {
        self.map_color(3)      // Selected/focused
    } else if self.is_default {
        self.map_color(2)      // Default but not focused
    } else {
        self.map_color(1)      // Normal
    };

    // Shadow uses index 8
    let shadow_attr = self.map_color(8);

    // Shortcut uses index 7
    let shortcut_attr = if is_disabled {
        self.map_color(4)      // Disabled shortcut
    } else {
        self.map_color(7)      // Shortcut color
    };
}
```

### Palette Mapping Indices Used

| Index | State | Meaning |
|-------|-------|---------|
| 1 | Normal | Standard button appearance |
| 2 | Default | Button marked as default (press Enter) |
| 3 | Focused | Button has keyboard focus |
| 4 | Disabled | Command not enabled in global set |
| 7 | Shortcut | Accelerator key character |
| 8 | Shadow | Right/bottom shadow effect |

### Event Handling

#### Broadcast Handling (Critical Fix)
The button correctly handles `CM_COMMAND_SET_CHANGED` broadcasts:

1. **Broadcasts are processed first**, before any disabled check
2. **Disabled buttons MUST receive broadcasts** to update their state when commands become enabled
3. **Broadcasts are NOT cleared** - they propagate to other views

```rust
if event.what == EventType::Broadcast {
    if event.command == CM_COMMAND_SET_CHANGED {
        let should_be_enabled = command_set::command_enabled(self.command);
        let is_currently_disabled = self.is_disabled();

        if should_be_enabled && is_currently_disabled {
            self.set_disabled(false);
        } else if !should_be_enabled && !is_currently_disabled {
            self.set_disabled(true);
        }
    }
    return;  // Broadcasts don't fall through
}
```

#### User Input Handling
- Disabled buttons ignore all keyboard and mouse events
- Enter key or spacebar activates when focused
- Mouse clicks within bounds activate the button
- Broadcasts can be sent instead of commands (for dialog propagation)

---

## 2. Palette System Architecture

### Overview
The system uses Borland Turbo Vision's indirect palette model where each view has a palette that maps logical indices to parent indices.

### Color Resolution Chain

```
Button.map_color(1)
    ↓
CP_BUTTON[1] = 13
    ↓
CP_GRAY_DIALOG[13] = 36 (if in dialog range 32-63)
    ↓
CP_APP_COLOR[36] = 0x20 (Black on Green)
    ↓
Attr { fg: Black, bg: Green }
```

### Application Palette (CP_APP_COLOR)

Root palette containing actual color attributes (u8 encoded):
- Bytes 1-8: Desktop colors
- Bytes 9-15: Menu colors
- Bytes 16-22: Additional menu colors
- Bytes 23-29: Dialog frame colors
- Bytes 30-36: Dialog interior colors
- **Bytes 37-42: Dialog controls (including buttons with GREEN background)**
- Bytes 43-48: Button colors
- Bytes 49-61: Other controls
- Bytes 62-85: Editor and misc colors

**Key fact:** Button background is GREEN (0x20 = LightGreen on Black encoded as 0x2X)

### Dialog Palette (CP_GRAY_DIALOG / CP_BLUE_DIALOG)

Maps dialog-range indices (32-63) to application palette indices:
- CP_GRAY_DIALOG: Dialog on gray/light background
- CP_BLUE_DIALOG: Dialog on blue background

### View Palette (CP_BUTTON)

Maps button logical indices (1-8) to dialog palette indices:

```rust
pub const CP_BUTTON: &[u8] = &[
    13, 13, 14, 14, 16, 15, 15, 9,
];
```

---

## 3. Recent Changes (Palette Owner Branch)

### File: `src/views/button.rs`

**Status:** Modified but no diff shown (likely debugging additions)

Logging added to track color mapping:
```rust
writeln!(log, "Button '{}' draw START, owner={:?}", self.title, self.owner).ok();
writeln!(log, "  Calling map_color(4)...").ok();
let result = self.map_color(4);
writeln!(log, "  map_color(4) OK").ok();
```

### File: `src/views/menu_bar.rs`

**Status:** Modified

**Changes:**
1. Added palette constant definitions at top of file
   ```rust
   const MENU_NORMAL: u8 = 1;
   const MENU_SELECTED: u8 = 2;
   const MENU_DISABLED: u8 = 3;
   const MENU_SHORTCUT: u8 = 4;
   ```

2. Updated palette indices to use named constants instead of magic numbers
   ```rust
   // Before:
   let normal_attr = self.map_color(1);
   
   // After:
   let normal_attr = self.map_color(MENU_NORMAL);
   ```

3. Code formatting improvements (import reorganization, line breaking)

### File: `src/views/menu_box.rs`

**Status:** Modified

**Changes:**
1. Added same palette constants as menu_bar
2. Updated to use named constants throughout
3. Fixed shortcut attribute color (line 241):
   ```rust
   // Before: (incorrect - used color instead of shortcut_attr)
   buf.put_char(shortcut_x + i, ch, color);
   
   // After: (correct)
   buf.put_char(shortcut_x + i, ch, shortcut_attr);
   ```
4. Code formatting improvements

### File: `src/views/view.rs`

**Status:** Modified

**Changes:**
1. Import reorganization (alphabetical sorting)
2. Code formatting improvements:
   - Multi-line method definitions
   - Proper line wrapping
3. Enhanced map_color() documentation:
   ```rust
   // Added comprehensive documentation about palette hierarchy:
   // Borland Turbo Vision palette layout (from program.h):
   //    1      = TBackground
   //    2-7    = TMenuView and TStatusLine (direct to app)
   //    8-15   = TWindow(Blue)
   //    16-23  = TWindow(Cyan)
   //    24-31  = TWindow(Gray)
   //    32-63  = TDialog (remapped through dialog palette)
   ```
4. No functional changes to palette mapping logic

---

## 4. Menu Components and Palettes

### MenuBar (`src/views/menu_bar.rs`)

**Palette:** CP_MENU_BAR
```rust
pub const CP_MENU_BAR: &[u8] = &[
    2, 39, 3, 4,  // 1-4: Normal, Selected, Disabled, Shortcut
];
```

**Usage:**
- Index 1: Normal menu text (Black on LightGray)
- Index 2: Selected menu text (White on Green)
- Index 3: Disabled menu text (DarkGray on LightGray)
- Index 4: Shortcut/accelerator (Red on LightGray)

**Features:**
- Dropdown menus appear when menu is clicked
- Keyboard navigation with arrow keys
- Cascading submenus supported
- Hot key support (Alt+F, etc.)

### MenuBox (`src/views/menu_box.rs`)

**Palette:** CP_MENU_BAR (same as MenuBar)

**Usage:**
- Popup menu container
- Modal execution for menu selection
- Same color indices as MenuBar

**Features:**
- Vertical menu with borders and shadows
- Supports regular items, submenus, and separators
- Mouse and keyboard interaction
- Returns selected command on completion

---

## 5. View Trait and Palette Integration

### Key Methods

#### `get_palette() -> Option<Palette>`
Returns this view's palette for color remapping. Default implementation returns None.

#### `map_color(color_index: u8) -> Attr`
Resolves a logical color index to final attribute through palette hierarchy.

**Process:**
1. Check if index is 0 (error color) - return white on black
2. Apply this view's palette (if present) to remap index
3. If remapped index is in dialog range (32-63), apply dialog palette
4. Look up final color in application palette (CP_APP_COLOR)
5. Return Attr with foreground and background colors

**Implementation Details:**
- Avoids unsafe pointer dereference by using standard palette chain
- View → Dialog (if applicable) → Application
- Logging support for debugging (calc.log)

### Color Attribute Types

```rust
pub struct Attr {
    pub fg: TvColor,  // Foreground color (0-15)
    pub bg: TvColor,  // Background color (0-15)
}

pub enum TvColor {
    Black = 0,
    Blue = 1,
    Green = 2,
    // ... 16 colors total ...
    White = 15,
}
```

---

## 6. Button Owner Tracking

### Purpose
Buttons track their owner (parent view) to support palette chain resolution.

### Implementation

```rust
owner: Option<*const dyn View>

fn set_owner(&mut self, owner: *const dyn View) {
    self.owner = Some(owner);
}

fn get_owner(&self) -> Option<*const dyn View> {
    self.owner
}
```

### Usage Notes
- Stored as raw const pointer (non-owning reference)
- Set by parent Group when button is added
- Used for palette chain traversal (potential future enhancement)
- Currently primarily for debugging/logging

---

## 7. Palette Definition Hierarchy

### Application Root (CP_APP_COLOR)
- Contains actual Attr values encoded as u8
- Indices 1-85 mapped to colors
- Root of all palette chains

### Dialog Layer (CP_GRAY_DIALOG / CP_BLUE_DIALOG)
- Maps dialog indices (32-63) to application indices
- Allows different dialog palettes for different backgrounds
- 32 entries (for indices 32-63)

### Component Layer (CP_BUTTON, CP_MENU_BAR, etc.)
- Maps component logical indices to dialog/app indices
- Usually 4-8 entries per component
- Enables component-specific color schemes

### Map Example: Button Index 1

```
CP_BUTTON[1]           = 13 (maps to dialog)
CP_GRAY_DIALOG[13]     = 36 (maps to app, adjusting for dialog range)
CP_APP_COLOR[36]       = 0x20 (LightGreen on Black)
Result: Green button background
```

---

## 8. Testing

### Test Suite in button.rs

Comprehensive tests covering:

1. **Button state on creation**
   - `test_button_creation_with_disabled_command`
   - `test_button_creation_with_enabled_command`

2. **Broadcast handling**
   - `test_disabled_button_receives_broadcast_and_becomes_enabled` (REGRESSION TEST)
   - `test_enabled_button_receives_broadcast_and_becomes_disabled`
   - `test_broadcast_does_not_clear_event`

3. **User input handling**
   - `test_disabled_button_ignores_keyboard_events`
   - `test_disabled_button_ignores_mouse_clicks`

4. **Builder pattern**
   - `test_button_builder`
   - `test_button_builder_default_is_false`
   - `test_button_builder_panics_without_bounds`
   - `test_button_builder_panics_without_title`
   - `test_button_builder_panics_without_command`

### Regression Tests Enabled
The critical regression test verifies that disabled buttons receive broadcasts:
```rust
#[test]
fn test_disabled_button_receives_broadcast_and_becomes_enabled() {
    // REGRESSION TEST: Disabled buttons must receive broadcasts...
    // Bug: disabled buttons returned early and never received CM_COMMAND_SET_CHANGED
    // Fix: Process broadcasts FIRST, before checking disabled state
}
```

---

## 9. Key Design Patterns

### State-Driven Color Selection
Buttons determine color based on state flags:
- Disabled state → Index 4
- Focused state → Index 3
- Default state → Index 2
- Normal state → Index 1

### Palette Chain
Indirect palette system allows:
- Component-specific color schemes (CP_BUTTON)
- Dialog context awareness (CP_GRAY_DIALOG)
- Root color definitions (CP_APP_COLOR)

### Broadcast-First Event Handling
Critical pattern ensures:
- Disabled buttons can become enabled
- Broadcasts don't block on disabled state
- Events propagate correctly through view hierarchy

### Owner Chain (Preparation)
Though not fully traversed in current impl:
- Supports future owner chain palette resolution
- Logs owner information for debugging
- Set by parent Group

---

## 10. Recent Commits

**Branch:** palette-owner

1. **a0df6b7** - Add comprehensive palette system documentation
2. **c2d3feb** - Merge branch 'palette-owner' 
3. **9b74935** - Remove unsafe pointer casting, use safe palette chain for color mapping
4. **ec1b3ba** - Merge main→palette-owner
5. **68899be** - Palette indirect implementation

**Key Focus:** Safe palette chain implementation without unsafe pointer traversal.

---

## 11. Files Affected by Recent Changes

| File | Status | Changes |
|------|--------|---------|
| button.rs | Modified | Logging added for debugging |
| menu_bar.rs | Modified | Palette constants, code cleanup |
| menu_box.rs | Modified | Palette constants, shortcut fix, code cleanup |
| view.rs | Modified | Documentation, formatting |
| calc.log | Created | Debug output for palette mapping |

---

## 12. Summary of Implementation Details

### Strengths
1. Safe palette chain implementation (no unsafe pointer traversal)
2. Comprehensive button state handling
3. Correct broadcast handling for command state changes
4. Well-documented palette system
5. Extensive test coverage including regression tests
6. Green button background maintained per Borland standard

### Areas for Enhancement
1. Full owner chain traversal could be implemented (currently uses fixed palette chain)
2. Additional views could implement custom palettes
3. Color customization at runtime (currently hardcoded)
4. More detailed palette documentation in code comments

### Critical Features
1. Disabled buttons receive broadcasts (fixed regression)
2. Palette indices mapped through 3-layer hierarchy
3. Shadow rendering with semi-transparency
4. Menu items support accelerator keys
5. Default button highlighting support

---

## Appendix: Palette Index Reference

### Button Palette Indices
- **1:** Normal (Black on Green)
- **2:** Default (LightGreen on Green)
- **3:** Selected/Focused (White on Green)
- **4:** Disabled (DarkGray on Green)
- **7:** Shortcut (Yellow on Green)
- **8:** Shadow (LightGray on DarkGray)

### Menu Palette Indices
- **1:** Normal (Black on LightGray)
- **2:** Selected (White on Green)
- **3:** Disabled (DarkGray on LightGray)
- **4:** Shortcut (Red on LightGray)

### Color Constants
- Background Green: 0x20 (maps to TvColor::Green with dark foreground)
- Normal text: Black on LightGray (0x70)
- Selected: White on Green (0x2F)
- Disabled: DarkGray text
- Shadow: LightGray on DarkGray (0x87)
