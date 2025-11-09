# Button Palette System - Code Reference

This document contains exact code sections from the implementation for quick reference.

## 1. Button Structure Definition

**File:** `/Users/enzo/Code/turbo-vision/src/views/button.rs:1-30`

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

impl Button {
    pub fn new(bounds: Rect, title: &str, command: CommandId, is_default: bool) -> Self {
        use crate::core::command_set;
        use crate::core::state::OF_POST_PROCESS;

        let mut state = 0;
        if !command_set::command_enabled(command) {
            state |= SF_DISABLED;
        }

        Self {
            bounds,
            title: title.to_string(),
            command,
            is_default,
            is_broadcast: false,
            state,
            options: OF_POST_PROCESS,
            owner: None,
        }
    }
}
```

## 2. Button Palette Definition

**File:** `/Users/enzo/Code/turbo-vision/src/core/palette.rs:CP_BUTTON`

```rust
// Button palette - maps button colors to parent (dialog) palette
#[rustfmt::skip]
pub const CP_BUTTON: &[u8] = &[
    13, 13, 14, 14, 16, 15, 15, 9,  // 1-8: (4=disabled), shadow maps to dialog 9
];
```

### Interpretation
```
Index 1 → 13  : Normal text
Index 2 → 13  : Default text
Index 3 → 14  : Selected/Focused text
Index 4 → 14  : Disabled text (index 4 in palette)
Index 5 → 16  : Reserved
Index 6 → 15  : Reserved
Index 7 → 15  : Shortcut text
Index 8 → 9   : Shadow (maps to dialog index 9)
```

## 3. Button Draw - State-Based Color Selection

**File:** `/Users/enzo/Code/turbo-vision/src/views/button.rs:draw() method, lines ~100-140`

```rust
fn draw(&mut self, terminal: &mut Terminal) {
    use std::io::Write;
    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("calc.log")
        .ok();

    if let Some(ref mut log) = log {
        writeln!(log, "Button '{}' draw START, owner={:?}", self.title, self.owner).ok();
    }

    let width = self.bounds.width() as usize;
    let height = self.bounds.height() as usize;

    let is_disabled = self.is_disabled();
    let is_focused = self.is_focused();

    // Button color indices (from CP_BUTTON palette):
    // 1: Normal text
    // 2: Default text
    // 3: Selected (focused) text
    // 4: Disabled text
    // 7: Shortcut text
    // 8: Shadow
    let button_attr = if is_disabled {
        if let Some(ref mut log) = log {
            writeln!(log, "  Calling map_color(4)...").ok();
        }
        let result = self.map_color(4);  // Disabled
        if let Some(ref mut log) = log {
            writeln!(log, "  map_color(4) OK").ok();
        }
        result
    } else if is_focused {
        if let Some(ref mut log) = log {
            writeln!(log, "  Calling map_color(3)...").ok();
        }
        let result = self.map_color(3);  // Selected/focused
        if let Some(ref mut log) = log {
            writeln!(log, "  map_color(3) OK").ok();
        }
        result
    } else if self.is_default {
        if let Some(ref mut log) = log {
            writeln!(log, "  Calling map_color(2)...").ok();
        }
        let result = self.map_color(2);  // Default but not focused
        if let Some(ref mut log) = log {
            writeln!(log, "  map_color(2) OK").ok();
        }
        result
    } else {
        if let Some(ref mut log) = log {
            writeln!(log, "  Calling map_color(1)...").ok();
        }
        let result = self.map_color(1);  // Normal
        if let Some(ref mut log) = log {
            writeln!(log, "  map_color(1) OK").ok();
        }
        result
    };

    if let Some(ref mut log) = log {
        writeln!(log, "  Calling map_color(8) for shadow...").ok();
    }
    let shadow_attr = self.map_color(8);  // Shadow
    if let Some(ref mut log) = log {
        writeln!(log, "  map_color(8) OK").ok();
    }

    // Shortcut attributes
    let shortcut_attr = if is_disabled {
        self.map_color(4)  // Disabled shortcut same as disabled text
    } else {
        self.map_color(7)  // Shortcut color
    };
    
    // ... rest of draw method
}
```

## 4. Critical: Broadcast Event Handling

**File:** `/Users/enzo/Code/turbo-vision/src/views/button.rs:handle_event() method, lines ~180-210`

```rust
fn handle_event(&mut self, event: &mut Event) {
    // Handle broadcasts FIRST, even if button is disabled
    //
    // IMPORTANT: This matches Borland's TButton::handleEvent() behavior:
    // - tbutton.cc:196 calls TView::handleEvent() first
    // - TView::handleEvent() (tview.cc:486) only checks sfDisabled for evMouseDown, NOT broadcasts
    // - tbutton.cc:235-263 processes evBroadcast in switch statement
    // - tbutton.cc:255-262 handles cmCommandSetChanged regardless of disabled state
    //
    // This is critical: disabled buttons MUST receive CM_COMMAND_SET_CHANGED broadcasts
    // so they can become enabled when their command becomes enabled in the global command set.
    if event.what == EventType::Broadcast {
        use crate::core::command::CM_COMMAND_SET_CHANGED;
        use crate::core::command_set;

        if event.command == CM_COMMAND_SET_CHANGED {
            // Query global command set (thread-local static, like Borland)
            let should_be_enabled = command_set::command_enabled(self.command);
            let is_currently_disabled = self.is_disabled();

            // Update disabled state if it changed
            // Matches Borland: tbutton.cc:256-260
            if should_be_enabled && is_currently_disabled {
                // Command was disabled, now enabled
                self.set_disabled(false);
            } else if !should_be_enabled && !is_currently_disabled {
                // Command was enabled, now disabled
                self.set_disabled(true);
            }

            // Event is not cleared - other views may need it
            // Matches Borland: broadcasts are not cleared in the button handler
        }
        return; // Broadcasts don't fall through to regular event handling
    }

    // Disabled buttons don't handle any other events (mouse, keyboard)
    // Matches Borland: TView::handleEvent() checks sfDisabled for evMouseDown (tview.cc:486)
    // and TButton's switch cases for evMouseDown/evKeyDown won't execute if disabled
    if self.is_disabled() {
        return;
    }

    match event.what {
        EventType::Keyboard => {
            // Only handle keyboard events if focused
            if !self.is_focused() {
                return;
            }
            if event.key_code == KB_ENTER || event.key_code == ' ' as u16 {
                if self.is_broadcast {
                    *event = Event::broadcast(self.command);
                } else {
                    *event = Event::command(self.command);
                }
            }
        }
        EventType::MouseDown => {
            // Check if click is within button bounds
            let mouse_pos = event.mouse.pos;
            if event.mouse.buttons & MB_LEFT_BUTTON != 0
                && mouse_pos.x >= self.bounds.a.x
                && mouse_pos.x < self.bounds.b.x
                && mouse_pos.y >= self.bounds.a.y
                && mouse_pos.y < self.bounds.b.y - 1  // Exclude shadow line
            {
                // Button clicked - generate command or broadcast
                if self.is_broadcast {
                    *event = Event::broadcast(self.command);
                } else {
                    *event = Event::command(self.command);
                }
            }
        }
        _ => {}
    }
}
```

## 5. Button Get Palette Method

**File:** `/Users/enzo/Code/turbo-vision/src/views/button.rs:get_palette() method`

```rust
fn get_palette(&self) -> Option<crate::core::palette::Palette> {
    use crate::core::palette::{Palette, palettes};
    Some(Palette::from_slice(palettes::CP_BUTTON))
}
```

## 6. View Trait: Map Color Implementation

**File:** `/Users/enzo/Code/turbo-vision/src/views/view.rs:map_color() method, lines ~301-365`

```rust
fn map_color(&self, color_index: u8) -> crate::core::palette::Attr {
    use crate::core::palette::{palettes, Attr, Palette};
    use std::io::Write;

    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("calc.log")
        .ok();

    if let Some(ref mut log) = log {
        writeln!(log, "      map_color({}) START", color_index).ok();
    }

    const ERROR_ATTR: u8 = 0x0F; // White on Black

    if color_index == 0 {
        return Attr::from_u8(ERROR_ATTR);
    }

    let mut color = color_index;

    // First, remap through this view's palette
    if let Some(palette) = self.get_palette() {
        if !palette.is_empty() {
            color = palette.get(color as usize);
            if let Some(ref mut log) = log {
                writeln!(
                    log,
                    "      Remapped {} -> {} via own palette",
                    color_index, color
                )
                .ok();
            }
            if color == 0 {
                return Attr::from_u8(ERROR_ATTR);
            }
        }
    }

    // NOTE: We skip the owner chain traversal to avoid unsafe pointer dereference.
    // Instead, we apply a standard palette chain: View -> Dialog -> Application
    //
    // Borland Turbo Vision palette layout (from program.h):
    //    1      = TBackground
    //    2-7    = TMenuView and TStatusLine (direct to app)
    //    8-15   = TWindow(Blue)
    //    16-23  = TWindow(Cyan)
    //    24-31  = TWindow(Gray)
    //    32-63  = TDialog (remapped through dialog palette)
    //
    // Only apply dialog palette remapping for indices in dialog range (32-63)
    if color >= 32 && color < 64 {
        let dialog_palette = Palette::from_slice(palettes::CP_GRAY_DIALOG);
        let remapped = dialog_palette.get((color - 31) as usize); // Dialog palette is 1-indexed, starting at 32
        if remapped > 0 {
            if let Some(ref mut log) = log {
                writeln!(
                    log,
                    "      Remapped {} -> {} via dialog palette",
                    color, remapped
                )
                .ok();
            }
            color = remapped;
        }
    }

    if let Some(ref mut log) = log {
        writeln!(log, "      Using CP_APP_COLOR[{}]", color).ok();
    }

    // Reached root (Application) - color is now an index into app palette
    // Use the application color palette to get the final attribute
    let app_palette = Palette::from_slice(palettes::CP_APP_COLOR);
    let final_color = app_palette.get(color as usize);
    if final_color == 0 {
        return Attr::from_u8(ERROR_ATTR);
    }
    Attr::from_u8(final_color)
}
```

## 7. Menu Bar Palette Constants (New)

**File:** `/Users/enzo/Code/turbo-vision/src/views/menu_bar.rs:lines ~16-19`

```rust
// MenuBar palette indices (matches Borland TMenuView)
const MENU_NORMAL: u8 = 1; // Normal item text
const MENU_SELECTED: u8 = 2; // Selected item text
const MENU_DISABLED: u8 = 3; // Disabled item text
const MENU_SHORTCUT: u8 = 4; // Shortcut/accelerator text
```

### Usage in draw_dropdown
**File:** `/Users/enzo/Code/turbo-vision/src/views/menu_bar.rs:draw_dropdown() method, lines ~159-162`

```rust
let normal_attr = self.map_color(MENU_NORMAL);
let selected_attr = self.map_color(MENU_SELECTED);
let disabled_attr = self.map_color(MENU_DISABLED);
let shortcut_attr = self.map_color(MENU_SHORTCUT);
```

## 8. Menu Box Palette Constants & Fix

**File:** `/Users/enzo/Code/turbo-vision/src/views/menu_box.rs:lines ~14-17`

```rust
// MenuBox palette indices (same as MenuBar - matches Borland TMenuView)
const MENU_NORMAL: u8 = 1; // Normal item text
const MENU_SELECTED: u8 = 2; // Selected item text
const MENU_DISABLED: u8 = 3; // Disabled item text
const MENU_SHORTCUT: u8 = 4; // Shortcut/accelerator text
```

### Bug Fix: Shortcut Color
**File:** `/Users/enzo/Code/turbo-vision/src/views/menu_box.rs:draw() method, line ~241`

```rust
// BEFORE (incorrect):
buf.put_char(shortcut_x + i, ch, color);

// AFTER (correct):
buf.put_char(shortcut_x + i, ch, shortcut_attr);
```

## 9. Menu Palette Definition

**File:** `/Users/enzo/Code/turbo-vision/src/core/palette.rs:CP_MENU_BAR`

```rust
// MenuBar palette (gray background, matching desktop colors)
#[rustfmt::skip]
pub const CP_MENU_BAR: &[u8] = &[
    2, 39, 3, 4,  // 1-4: Normal (Black/LightGray), Selected (White/Green), Disabled (DarkGray/LightGray), Shortcut (Red/LightGray)
];
```

## 10. Application Color Palette (Root)

**File:** `/Users/enzo/Code/turbo-vision/src/core/palette.rs:CP_APP_COLOR`

```rust
#[rustfmt::skip]
pub const CP_APP_COLOR: &[u8] = &[
    0x71, 0x70, 0x78, 0x74, 0x20, 0x28, 0x24, 0x17, // 1-8: Desktop colors
    0x1F, 0x1A, 0x31, 0x31, 0x1E, 0x71, 0x1F,       // 9-15: Menu colors
    0x37, 0x3F, 0x3A, 0x13, 0x13, 0x3E, 0x21,       // 16-22: More menu
    0x70, 0x7F, 0x7A, 0x13, 0x13, 0x70, 0x7F,       // 23-29: Dialog frame
    0x7A, 0x13, 0x13, 0x70, 0x70, 0x7F, 0x7E,       // 30-36: Dialog interior
    0x20, 0x2B, 0x2F, 0x87, 0x2E, 0x70,             // 37-42: Dialog controls
    0x20, 0x2A, 0x2F, 0x1F, 0x2E, 0x70,             // 43-48: Button (GREEN BACKGROUND!)
    0x20, 0x72, 0x31, 0x31, 0x30, 0x2F,             // 49-54: Cluster
    0x3E, 0x31,                                      // 55-56: Input line
    0x13, 0x13, 0x30, 0x3E, 0x13,                   // 57-61: History
    0x30, 0x3F, 0x3E, 0x70, 0x2F,                   // 62-66: List viewer
    0x37, 0x3F, 0x3A, 0x20, 0x2E, 0x30,             // 67-72: Info pane
    0x3F, 0x3E, 0x1F, 0x2F, 0x1A, 0x20,             // 73-78: Cluster (more)
    0x72, 0x31, 0x31, 0x30, 0x2F, 0x3E,             // 79-84: Editor
    0x31,                                            // 85: Reserved
];
```

## 11. Color Attribute Encoding

**File:** `/Users/enzo/Code/turbo-vision/src/core/palette.rs:Attr struct`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attr {
    pub fg: TvColor,  // Foreground (0-15)
    pub bg: TvColor,  // Background (0-15)
}

impl Attr {
    pub const fn new(fg: TvColor, bg: TvColor) -> Self {
        Self { fg, bg }
    }

    pub fn from_u8(byte: u8) -> Self {
        Self {
            fg: TvColor::from_u8(byte & 0x0F),
            bg: TvColor::from_u8((byte >> 4) & 0x0F),
        }
    }

    pub fn to_u8(self) -> u8 {
        (self.fg as u8) | ((self.bg as u8) << 4)
    }
}
```

## 12. TvColor Enum

**File:** `/Users/enzo/Code/turbo-vision/src/core/palette.rs:TvColor enum`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TvColor {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    Yellow = 14,
    White = 15,
}
```

## 13. Regression Test: Broadcast Handling

**File:** `/Users/enzo/Code/turbo-vision/src/views/button.rs:tests module, lines ~430-480`

```rust
#[test]
fn test_disabled_button_receives_broadcast_and_becomes_enabled() {
    // REGRESSION TEST: Disabled buttons must receive broadcasts to become enabled
    // This tests the fix for the bug where disabled buttons returned early
    // and never received CM_COMMAND_SET_CHANGED broadcasts

    const TEST_CMD: u16 = 502;

    // Start with command disabled
    command_set::disable_command(TEST_CMD);

    let mut button = Button::new(
        Rect::new(0, 0, 10, 2),
        "Test",
        TEST_CMD,
        false
    );

    // Verify button starts disabled
    assert!(button.is_disabled(), "Button should start disabled");

    // Enable the command in the global command set
    command_set::enable_command(TEST_CMD);

    // Send broadcast to button
    let mut event = Event::broadcast(CM_COMMAND_SET_CHANGED);
    button.handle_event(&mut event);

    // Verify button is now enabled
    assert!(!button.is_disabled(), "Button should be enabled after receiving broadcast");
}
```

## 14. Palette Resolution Example

**Step-by-step walkthrough of button.map_color(1):**

```
START: button.map_color(1)

Step 1: Check this view's palette
  - Button.get_palette() returns Some(CP_BUTTON)
  - CP_BUTTON[1] = 13
  - Color now = 13
  Log: "Remapped 1 -> 13 via own palette"

Step 2: Check if dialog palette needed
  - Is 13 >= 32 && < 64? NO
  - Skip dialog palette
  
Step 3: Look up in app palette
  - CP_APP_COLOR[13] = 0x1E
  
Step 4: Decode color attribute
  - 0x1E = 0b00011110
  - Foreground: 0x0E = 14 = TvColor::Yellow
  - Background: 0x01 = 1 = TvColor::Blue
  - Result: Attr { fg: Yellow, bg: Blue }

END: Return Attr { Yellow on Blue }

Note: 0x1E is incorrect for buttons (should be green background)
The correct sequence would use index 13 which maps through dialog.
```

## 15. Owner Tracking

**File:** `/Users/enzo/Code/turbo-vision/src/views/button.rs:set_owner() and get_owner()`

```rust
fn set_owner(&mut self, owner: *const dyn View) {
    self.owner = Some(owner);
}

fn get_owner(&self) -> Option<*const dyn View> {
    self.owner
}
```

**Usage in Group (parent adds child):**
```rust
// In Group::add_child():
child.set_owner(self as *const dyn View);  // Set parent pointer
```

## Key Takeaways

1. **Palette Chain:** View → Dialog → App
2. **Button Palette:** 8-element array mapping button states
3. **Critical Fix:** Broadcasts processed before disabled check
4. **Green Background:** Button uses green background (0x2X encoding)
5. **Index Ranges:** 32-63 triggers dialog palette remapping
6. **Owner Tracking:** Non-owning pointer for palette chain (future use)
7. **Logging:** Debug output to calc.log for color mapping trace
8. **Constants:** Menu components use named constants (not magic numbers)
